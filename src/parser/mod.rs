use {
    crate::tokenizer::token::Token,
    std::{cmp::Ordering, collections::HashMap},
    value::HuonValue,
};

pub mod value;

type Result<'a, T> = std::result::Result<T, ParserError<'a>>;

#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum ParserError<'a> {
    #[error("EOF")]
    Eof,

    #[error("Invalid token: {_0:?}")]
    InvalidToken(Token<'a>),

    #[error("Couldn't convert from: {_0:?}")]
    InvalidHuonValue(Token<'a>),
}

pub type ValueMap<'a> = HashMap<&'a str, HuonValue<'a>>;

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

    pub fn parse(input: Vec<Token<'a>>) -> Result<'a, ValueMap<'a>> {
        let mut parser = Self::new(input);
        parser.parse_object(0)
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

    fn parse_object(&mut self, expected_indent: usize) -> Result<'a, ValueMap<'a>> {
        let mut map = HashMap::new();

        while let Ok(token) = self.peek() {
            if self.collapse > 0 {
                self.collapse -= 1;
                return Ok(map);
            }

            if let Token::NewLine = token {
                self.advance()?;

                let next_token = self.peek()?;

                match next_token {
                    Token::WhiteSpace(n) if expected_indent > 0 && (n / 4) < expected_indent => {
                        self.collapse = expected_indent - (n / 4) - 1;
                        return Ok(map);
                    }

                    Token::Identifier(_) if expected_indent == 1 => {
                        return Ok(map);
                    }

                    _ => continue,
                }
            }

            self.check_indentation(token, expected_indent)?;

            let key = match self.advance()? {
                Token::Identifier(s) => s,
                token => return Err(ParserError::InvalidToken(token)),
            };

            let value = match self.peek()? {
                Token::WhiteSpace(1) => {
                    self.advance()?;
                    self.parse_value()?
                }

                Token::NewLine => {
                    self.advance()?;

                    match self.peek()? {
                        Token::WhiteSpace(n) if (n / 4) > expected_indent => {
                            self.advance()?;
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
        Ok(match self.advance()? {
            Token::Str(s) => HuonValue::String(s),
            Token::Int(i) => HuonValue::Int(i),
            Token::Boolean(b) => HuonValue::Boolean(b),
            Token::Float(f) => HuonValue::Float(f),
            Token::Null => HuonValue::Null,
            token => return Err(ParserError::InvalidToken(token)),
        })
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

    macro_rules! map {
        ( $( $key:expr => $value:expr ),* ) => {
            {
                let mut m = HashMap::new();
                $( m.insert($key, $value); )*
                m
            }
        };
    }

    #[test]
    fn test_parser() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let tokens = Tokenizer::scan(include_str!("../../test.huon"))?;

        let map = Parser::parse(tokens)?;

        let expected = map! {
            "name" => HuonValue::String("John"),
            "job1" => HuonValue::Object(map! {
                "category" => HuonValue::Object(map! {
                    "name" => HuonValue::String("IT")
                }),
                "info" => HuonValue::Object(map! {
                    "pay" => HuonValue::Float(-4200.5),
                    "payrate" => HuonValue::Object(map! {
                        "iteration" => HuonValue::String("monthly"),
                        "date" => HuonValue::String("Last Friday of every month"),
                        "monthly_increase" => HuonValue::String("5%")
                    })
                }),
                "name" => HuonValue::String("Software Engineer")
            }),
            "age" => HuonValue::Int(32),
            "job2" => HuonValue::Object(map! {
                "category" => HuonValue::Object(map! {
                    "name" => HuonValue::String("Security")
                }),
                "info" => HuonValue::Object(map! {
                    "pay" => HuonValue::Int(3700), // treated as an int here because the parser/tokenizer does not find an integer
                    "payrate" => HuonValue::Object(map! {
                        "iteration" => HuonValue::String("weekly"),
                        "date" => HuonValue::String("Every Friday")
                    })
                }),
                "name" => HuonValue::String("Bodyguard")
            }),
            "last_name" => HuonValue::String("Doe")
        };

        assert_eq!(map, expected);

        Ok(())
    }

    #[test]
    fn fail_int_before_ident() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let tokens = Tokenizer::scan("1job1: \"swe\"")?;

        let err = Parser::parse(tokens).unwrap_err();

        assert_eq!(err, ParserError::InvalidToken(Token::Int(1)));

        Ok(())
    }
}
