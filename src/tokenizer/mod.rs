use std::iter::Peekable;
use std::num::{ParseFloatError, ParseIntError};
use std::str::CharIndices;

use token::Token;

pub mod token;

#[derive(Debug, thiserror::Error, Clone, PartialEq)]
pub enum TokenizerError {
    /// End of file
    #[error("End of file")]
    EOF,

    #[error("The identifier '{_0}' is not valid")]
    InvalidIdentifier(String),

    #[error("Found an unexpected character: {_0}")]
    UnexpectedCharacter(char),

    #[error("Failed to parse a float: {_0}")]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Failed to parse an int: {_0}")]
    ParseIntError(#[from] ParseIntError),
}

type Result<T> = std::result::Result<T, TokenizerError>;

#[derive(Debug, Clone)]
pub struct Tokenizer<'a> {
    input: &'a str,
    char_indices: Peekable<CharIndices<'a>>,
}

impl<'a> Tokenizer<'a> {
    #[must_use]
    pub fn new(input: &'a str) -> Self {
        Self {
            input,
            char_indices: input.char_indices().peekable(),
        }
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = Result<Token<'a>>;

    fn next(&mut self) -> Option<Self::Item> {
        let (token_start_idx, char) = self.char_indices.next()?;

        let token_result = match char {
            '"' => self.read_string(),

            char if char.is_ascii_digit() || char == '-' => self.read_number(token_start_idx),

            char if is_valid_identifier_char(char) => {
                let raw_ident = self.read_identifier(token_start_idx);

                if let Some((_, ':')) = self.char_indices.peek() {
                    self.char_indices.next();
                    return Some(Ok(Token::Identifier(raw_ident)));
                }

                parse_keyword(raw_ident).ok_or(TokenizerError::UnexpectedCharacter(char))
            }

            '[' => Ok(Token::ListStart),

            ']' => Ok(Token::ListEnd),

            ',' => Ok(Token::Separator),

            '\n' => Ok(Token::NewLine),

            '\r' => match self.char_indices.peek() {
                Some((_, '\n')) => {
                    self.char_indices.next();
                    Ok(Token::NewLine)
                }
                Some((_, c)) => Err(TokenizerError::UnexpectedCharacter(*c)),
                None => Ok(Token::NewLine),
            },

            ' ' => self.read_whitespace(),

            c => Err(TokenizerError::UnexpectedCharacter(c)),
        };

        Some(token_result)
    }
}

impl<'a> Tokenizer<'a> {
    fn read_identifier(&mut self, start_idx: usize) -> &'a str {
        loop {
            match self.char_indices.peek() {
                Some((_, char)) if is_valid_identifier_char(*char) => {
                    self.char_indices.next();
                }
                Some((end_idx, _)) => return &self.input[start_idx..*end_idx],
                None => return &self.input[start_idx..],
            }
        }
    }

    fn read_string(&mut self) -> Result<Token<'a>> {
        let start_idx = match self.char_indices.peek() {
            Some((idx, _)) => *idx,
            None => return Err(TokenizerError::EOF),
        };

        loop {
            match self.char_indices.peek() {
                Some((_, '"')) => {
                    let (end_idx, _) = self.char_indices.next().unwrap(); // advance past the closing quote
                    return Ok(Token::Str(&self.input[start_idx..end_idx]));
                }
                Some(_) => {
                    self.char_indices.next();
                }
                None => return Err(TokenizerError::EOF),
            }
        }
    }

    fn read_number(&mut self, start_idx: usize) -> Result<Token<'a>> {
        let mut is_float = false;

        loop {
            match self.char_indices.peek() {
                Some((_, char)) if char.is_ascii_digit() => {
                    self.char_indices.next();
                }
                Some((_, '.')) => {
                    is_float = true;
                    self.char_indices.next();
                }
                Some((end_idx, _)) => {
                    let num_str = &self.input[start_idx..*end_idx];
                    return if is_float {
                        Ok(num_str.parse().map(Token::Float)?)
                    } else {
                        Ok(num_str.parse().map(Token::Int)?)
                    };
                }
                None => {
                    let num_str = &self.input[start_idx..];
                    return if is_float {
                        Ok(num_str.parse().map(Token::Float)?)
                    } else {
                        Ok(num_str.parse().map(Token::Int)?)
                    };
                }
            }
        }
    }

    fn read_whitespace(&mut self) -> Result<Token<'a>> {
        let mut count = 1;
        loop {
            match self.char_indices.peek() {
                Some((_, ' ')) => {
                    count += 1;
                    self.char_indices.next();
                }
                _ => return Ok(Token::WhiteSpace(count)),
            }
        }
    }
}

fn is_valid_identifier_char(char: char) -> bool {
    (char.is_ascii_alphabetic() || char.is_ascii_digit()) || ['_'].contains(&char)
}

fn parse_keyword(input: &str) -> Option<Token<'_>> {
    Some(match input {
        "true" => Token::Boolean(true),
        "false" => Token::Boolean(false),
        "null" => Token::Null,
        _ => return None,
    })
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use crate::tokenizer::TokenizerError;
    use crate::tokenizer::token::Token;

    use super::Result;
    use super::Tokenizer;

    #[test]
    fn read_string() -> std::result::Result<(), TokenizerError> {
        let input = r#""Hi""#;
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(tokens, vec![Token::Str("Hi")]);

        Ok(())
    }

    #[test]
    fn identifier() -> std::result::Result<(), TokenizerError> {
        let input = "job1: \"swe\"";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("job1"),
                Token::WhiteSpace(1),
                Token::Str("swe")
            ]
        );

        Ok(())
    }

    #[test]
    fn read_number_i64() -> std::result::Result<(), TokenizerError> {
        let input = "number: 69420";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("number"),
                Token::WhiteSpace(1),
                Token::Int(69420)
            ]
        );

        Ok(())
    }

    #[test]
    fn read_number_f64() -> std::result::Result<(), TokenizerError> {
        let input = "number: 69420.187";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("number"),
                Token::WhiteSpace(1),
                Token::Float(69420.187)
            ]
        );

        Ok(())
    }

    #[test]
    fn read_number_i64_negative() -> std::result::Result<(), TokenizerError> {
        let input = "number: -69420";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("number"),
                Token::WhiteSpace(1),
                Token::Int(-69420)
            ]
        );

        Ok(())
    }

    #[test]
    fn read_number_f64_negative() -> std::result::Result<(), TokenizerError> {
        let input = "number: -69420.187";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("number"),
                Token::WhiteSpace(1),
                Token::Float(-69420.187)
            ]
        );

        Ok(())
    }

    #[test]
    fn read_list_newline() -> std::result::Result<(), TokenizerError> {
        let input = "numbers: [
    -3.5
    2.5
    1.1
]";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("numbers"),
                Token::WhiteSpace(1),
                Token::ListStart,
                Token::NewLine,
                Token::WhiteSpace(4),
                Token::Float(-3.5),
                Token::NewLine,
                Token::WhiteSpace(4),
                Token::Float(2.5),
                Token::NewLine,
                Token::WhiteSpace(4),
                Token::Float(1.1),
                Token::NewLine,
                Token::ListEnd,
            ]
        );

        Ok(())
    }

    #[test]
    fn read_list_spaced() -> std::result::Result<(), TokenizerError> {
        let input = "numbers: [-3.5 2.5 1.1]";
        let tokens: Vec<_> = Tokenizer::new(input).collect::<Result<Vec<_>>>()?;

        assert_eq!(
            tokens,
            vec![
                Token::Identifier("numbers"),
                Token::WhiteSpace(1),
                Token::ListStart,
                Token::Float(-3.5),
                Token::WhiteSpace(1),
                Token::Float(2.5),
                Token::WhiteSpace(1),
                Token::Float(1.1),
                Token::ListEnd,
            ]
        );

        Ok(())
    }

    #[test]
    fn advance_and_peek() -> std::result::Result<(), TokenizerError> {
        let input = "true false null";
        let mut tokenizer = Tokenizer::new(input).peekable();

        assert_eq!(tokenizer.peek().unwrap().clone()?, Token::Boolean(true));
        assert_eq!(tokenizer.next().unwrap()?, Token::Boolean(true));

        assert_eq!(tokenizer.peek().unwrap().clone()?, Token::WhiteSpace(1));
        assert_eq!(tokenizer.next().unwrap()?, Token::WhiteSpace(1));

        assert_eq!(tokenizer.peek().unwrap().clone()?, Token::Boolean(false));
        assert_eq!(tokenizer.next().unwrap()?, Token::Boolean(false));

        assert_eq!(tokenizer.peek().unwrap().clone()?, Token::WhiteSpace(1));
        assert_eq!(tokenizer.next().unwrap()?, Token::WhiteSpace(1));

        assert_eq!(tokenizer.peek().unwrap().clone()?, Token::Null);
        assert_eq!(tokenizer.next().unwrap()?, Token::Null);

        assert!(tokenizer.next().is_none());

        Ok(())
    }
}
