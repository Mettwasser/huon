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
}

impl<'a> Parser<'a> {
    pub fn new(input: Vec<Token<'a>>) -> Self {
        Self { input, cursor: 0 }
    }

    pub fn parse(&mut self) -> Result<'a, HashMap<&'a str, HuonValue<'a>>> {
        self.parse_object(0)
    }

    fn parse_object(
        &mut self,
        expected_indent: usize,
    ) -> Result<'a, HashMap<&'a str, HuonValue<'a>>> {
        let mut map = HashMap::new();

        while let Ok(token) = self.peek() {
            let mut had_whitespace_check = false;
            // Check if this line is at the expected indentation
            if let Token::WhiteSpace(n) = token {
                let indent = n / 4;
                match indent.cmp(&expected_indent) {
                    Ordering::Less => {
                        // We have dedented: end of this nested object.
                        return Ok(map);
                    }
                    Ordering::Greater => {
                        return Err(ParserError::InvalidToken(token));
                    }
                    Ordering::Equal => {
                        // Indentation matches expected, so consume it.
                        self.advance()?;
                    }
                }
                had_whitespace_check = true;
            }

            if let Token::NewLine = token {
                // Consume the newline and continue to the next line.
                self.advance()?;

                if let (Token::Identifier(_), true, false) =
                    (self.peek()?, expected_indent > 0, had_whitespace_check)
                {
                    return Ok(map);
                }
            }

            // Check if this line is at the expected indentation
            if let Token::WhiteSpace(n) = self.peek()? {
                let indent = n / 4;
                match indent.cmp(&expected_indent) {
                    Ordering::Less => {
                        // We have dedented: end of this nested object.
                        return Ok(map);
                    }
                    Ordering::Greater => {
                        return Err(ParserError::InvalidToken(token));
                    }
                    Ordering::Equal => {
                        // Indentation matches expected, so consume it.
                        self.advance()?;
                    }
                }
            }

            // Now, expect an identifier.
            let key = match self.advance()? {
                Token::Identifier(s) => s,
                token => return Err(ParserError::InvalidToken(token)),
            };

            // Here we check if the value is inline or nested.
            let value = match self.peek()? {
                // Inline value: may be introduced by a separator (or a single space).
                Token::WhiteSpace(1) => {
                    self.advance()?; // consume the inline whitespace
                    self.parse_value()? // parse the literal value
                }
                // Nested object: line break indicates new block
                Token::NewLine => {
                    self.advance()?; // consume newline
                    // The next token must be indentation strictly greater than current expected_indent.
                    match self.peek()? {
                        Token::WhiteSpace(n) if (n / 4) > expected_indent => {
                            // Consume the indentation and then recursively parse.
                            self.advance()?;
                            HuonValue::Object(self.parse_object(n / 4)?) // wrap the resulting map into a HuonValue::Object(...)
                        }
                        token => return Err(ParserError::InvalidToken(token)),
                    }
                }
                token => return Err(ParserError::InvalidToken(token)),
            };

            // Insert into the map.
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
