pub mod token;
use token::Token;

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum TokenizerError {
    /// End of file
    #[error("End of file")]
    EOF,

    #[error("The identifier '{_0}' is not valid")]
    InvalidIdentifier(String),

    #[error("Found an unexpected character: {_0}")]
    UnexpectedCharacter(char),
}

type Result<T> = std::result::Result<T, TokenizerError>;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Tokenizer<'a> {
    /// The original input to operate on
    input: &'a str,

    /// The "pointer" to where the lexer is at
    cursor: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            ..Default::default()
        }
    }
}

impl<'a> Tokenizer<'a> {
    pub fn scan(input: &'a str) -> Result<Vec<Token<'a>>> {
        let mut tokenizer = Self::new(input);

        let mut buffer = Vec::new();
        loop {
            match tokenizer.next_token() {
                Ok(token) => buffer.push(token),
                Err(TokenizerError::EOF) => break,
                Err(err) => return Err(err),
            };
        }

        Ok(buffer)
    }

    fn next_token(&mut self) -> Result<Token<'a>> {
        let token_start_idx = self.cursor;
        match self.advance()? {
            '"' => self.read_string(),

            char if char.is_ascii_digit() => self.read_number(token_start_idx),

            char if is_valid_identifier_char(char) => {
                let raw_ident = self.read_identifier(token_start_idx)?;

                match self.peek() {
                    Ok(':') => {
                        self.advance()?;
                        return Ok(Token::Identifier(raw_ident));
                    }
                    Err(TokenizerError::EOF) => (),
                    Err(err) => return Err(err),
                    _ => (),
                }

                parse_bool(raw_ident)
                    .map(Token::Boolean)
                    .ok_or_else(|| TokenizerError::InvalidIdentifier(raw_ident.to_owned()))
            }

            '\n' => Ok(Token::NewLine),
            '\r' => match self.peek()? {
                '\n' => {
                    self.advance()?;
                    Ok(Token::NewLine)
                }
                c => Err(TokenizerError::UnexpectedCharacter(c)),
            },

            ' ' => self.read_whitespace(token_start_idx),

            c => Err(TokenizerError::UnexpectedCharacter(c)),
        }
    }

    /// Peeks the next char.
    fn peek(&self) -> Result<char> {
        self.input
            .chars()
            .nth(self.cursor)
            .ok_or(TokenizerError::EOF)
    }

    /// Reads the current char, advancing the lexer's position.
    fn advance(&mut self) -> Result<char> {
        let next_char = self.input.chars().nth(self.cursor);

        self.cursor += 1;
        next_char.ok_or(TokenizerError::EOF)
    }

    fn read_identifier(&mut self, start_idx: usize) -> Result<&'a str> {
        self.advance_until(|char| !is_valid_identifier_char(char))?;

        Ok(&self.input[start_idx..self.cursor])
    }

    /// Reads a [TokenType::Str].
    ///
    /// Expects the cursor to be AFTER the initial `"`
    ///
    /// For example:
    /// ```txt
    ///     "Some string!"
    ///      ^
    ///      Expects cursor to be here (before S)!
    /// ```
    fn read_string(&mut self) -> Result<Token<'a>> {
        // Snapshot current pos
        let start_idx = self.cursor;

        self.advance_until(|char| char == '"')?;

        // Skip the trailing `"`
        self.advance()?;

        Ok(Token::Str(&self.input[start_idx..(self.cursor - 1)]))
    }

    fn read_number(&mut self, start_idx: usize) -> Result<Token<'a>> {
        self.advance_until(|char| !char.is_ascii_digit())?;

        Ok(Token::Int(
            self.input[start_idx..self.cursor].parse().unwrap(),
        ))
    }

    fn read_whitespace(&mut self, start_idx: usize) -> Result<Token<'a>> {
        self.advance_until(|char| char != ' ')?;

        Ok(Token::WhiteSpace(self.cursor - start_idx))
    }

    #[inline]
    fn advance_until<F>(&mut self, guard: F) -> Result<()>
    where
        F: Fn(char) -> bool,
    {
        loop {
            let char = self.peek();

            if char.is_err() || guard(char.unwrap()) {
                break;
            }

            self.advance()?;
        }

        Ok(())
    }
}

fn is_valid_identifier_char(char: char) -> bool {
    char.is_ascii_alphabetic() || ['_'].contains(&char)
}

fn parse_bool(input: &str) -> Option<bool> {
    match input {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use crate::tokenizer::TokenizerError;
    use crate::tokenizer::token::Token;

    use super::Result;
    use super::Tokenizer;

    #[test]
    fn read_string() -> Result<()> {
        let input = r#""Hi""#;
        let mut lexer = Tokenizer::new(input);
        lexer.advance()?;
        let s = lexer.read_string()?;

        assert_eq!(s, Token::Str("Hi"));
        assert_eq!(lexer.advance(), Err(TokenizerError::EOF));

        Ok(())
    }

    #[test]
    fn read_number() -> Result<()> {
        let input = r#"number: 69420"#;
        let mut lexer = Tokenizer::new(input);
        lexer.cursor = 8;
        let s = lexer.read_number(8)?;

        assert_eq!(s, Token::Int(69420));
        assert_eq!(lexer.advance(), Err(TokenizerError::EOF));

        Ok(())
    }

    #[test]
    fn advance_and_peek() -> Result<()> {
        let input = r#"abc"#;
        let mut lexer = Tokenizer::new(input);

        assert_eq!(lexer.peek()?, 'a');
        assert_eq!(lexer.cursor, 0);

        assert_eq!(lexer.advance()?, 'a');
        assert_eq!(lexer.peek()?, 'b');
        assert_eq!(lexer.cursor, 1);

        assert_eq!(lexer.advance()?, 'b');
        assert_eq!(lexer.peek()?, 'c');
        assert_eq!(lexer.cursor, 2);

        assert_eq!(lexer.advance()?, 'c');
        assert_eq!(lexer.peek(), Err(TokenizerError::EOF));
        assert_eq!(lexer.cursor, 3);

        Ok(())
    }
}
