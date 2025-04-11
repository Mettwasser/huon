use {
    crate::tokenizer::token::Token,
    std::{cmp::Ordering, collections::HashMap},
    value::HuonValue,
};

pub mod indentation;
pub mod value;

type Result<'a, T> = std::result::Result<T, ParserError<'a>>;

#[derive(Debug, thiserror::Error)]
pub enum ParserError<'a> {
    #[error("EOF")]
    Eof,

    #[error("Invalid token: {_0:?}")]
    InvalidToken(Token<'a>),

    #[error("Couldn't convert from: {_0:?}")]
    InvalidHuonValue(Token<'a>),
}

pub struct Parser<'a> {
    input: Vec<Token<'a>>,
    cursor: usize,
    collapse: usize,
}

impl<'a> Parser<'a> {
    pub fn new(input: Vec<Token<'a>>) -> Self {
        Self {
            input,
            cursor: 0,
            collapse: 0,
        }
    }

    pub fn parse(&mut self) -> Result<'a, HashMap<&'a str, HuonValue<'a>>> {
        self.parse_object(0)
    }

    /// A helper func to check if a token is whitespace with the expected indentation.
    /// If found, it consumes the token and returns true.
    /// Otherwise, it returns false, or an error if the indentation is greater.
    fn check_indentation(&mut self, token: Token<'a>, expected_indent: usize) -> Result<'a, bool> {
        if let Token::WhiteSpace(n) = token {
            let indent = n / 4;
            match indent.cmp(&expected_indent) {
                Ordering::Less => return Ok(false),
                Ordering::Greater => return Err(ParserError::InvalidToken(token)),
                Ordering::Equal => {
                    self.advance()?;
                    return Ok(true);
                }
            }
        }
        Ok(false)
    }

    fn parse_object(
        &mut self,
        expected_indent: usize,
    ) -> Result<'a, HashMap<&'a str, HuonValue<'a>>> {
        let mut map = HashMap::new();

        while let Ok(token) = self.peek() {
            // this is to "notify" recursive calls that they should return and
            // dedent 1 level
            if self.collapse > 0 {
                self.collapse -= 1;
                return Ok(map);
            }

            // First check the line's starting whitespace.
            let had_whitespace_check = self.check_indentation(token, expected_indent)?;

            // If the token is a NewLine, consume it and check for a dedented identifier.
            if let Token::NewLine = token {
                self.advance()?; // consume the newline

                // Check if the following token indicates an identifier with a dedented (or no) preceding whitespace.
                // This implements your special "collapse" behavior.
                if let (Token::Identifier(_), true, false) =
                    (self.peek()?, expected_indent > 0, had_whitespace_check)
                {
                    // we subtract 1 because we will do the first return immediately
                    self.collapse = expected_indent - 1;
                    return Ok(map);
                }
            }

            // Check indentation again before attempting to parse a key.
            let next_token = self.peek()?;
            self.check_indentation(next_token, expected_indent)?;

            // Expect an identifier key.
            let key = match self.advance()? {
                Token::Identifier(s) => s,
                token => return Err(ParserError::InvalidToken(token)),
            };

            // Now decide whether the value is inline or a nested block.
            let value = match self.peek()? {
                // Inline value: indicated by a single whitespace.
                Token::WhiteSpace(1) => {
                    self.advance()?; // consume the inline whitespace
                    self.parse_value()?
                }
                // Nested object: indicated by a newline.
                Token::NewLine => {
                    self.advance()?; // consume the newline
                    // The next token must be whitespace with an indentation strictly greater than expected.
                    match self.peek()? {
                        Token::WhiteSpace(n) if (n / 4) > expected_indent => {
                            self.advance()?; // consume the nested whitespace
                            HuonValue::Object(self.parse_object(n / 4)?)
                        }
                        token => return Err(ParserError::InvalidToken(token)),
                    }
                }
                token => return Err(ParserError::InvalidToken(token)),
            };

            map.insert(key, value);
        }
        Ok(map)
    }

    fn parse_value(&mut self) -> Result<'a, HuonValue<'a>> {
        // This function will try to parse a literal value.
        match self.advance()? {
            Token::Str(s) => Ok(HuonValue::String(s)),
            Token::Int(i) => Ok(HuonValue::Int(i)),
            Token::Boolean(b) => Ok(HuonValue::Boolean(b)),
            token => Err(ParserError::InvalidToken(token)),
        }
    }

    fn peek(&self) -> Result<'a, Token<'a>> {
        self.input.get(self.cursor).cloned().ok_or(ParserError::Eof)
    }

    fn advance(&mut self) -> Result<'a, Token<'a>> {
        let token = self.peek()?;
        self.cursor += 1;
        Ok(token)
    }
}

#[cfg(test)]
mod tests {
    use {super::*, crate::tokenizer::Tokenizer};

    #[test]
    fn test_parser() {
        let mut tokenizer = Tokenizer::new(include_str!("../../test.huon"));
        let tokens = tokenizer.scan().unwrap();

        let mut parser = Parser::new(tokens);
        let result = parser.parse().unwrap();

        dbg!(result);
    }
}
