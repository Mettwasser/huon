use {
    crate::{
        tokenizer::{token::Token, Tokenizer, TokenizerError},
        DecoderOptions,
    },
    std::{cmp::Ordering, collections::HashMap, iter::Peekable},
    value::HuonValue,
};

pub mod value;

type Result<'a, T> = std::result::Result<T, ParserError<'a>>;

#[derive(Debug, thiserror::Error, PartialEq)]
pub enum ParserError<'a> {
    #[error("EOF")]
    Eof,

    #[error("Invalid token: {_0:?}")]
    InvalidToken(Token<'a>),

    #[error("Couldn't convert from: {_0:?}")]
    InvalidHuonValue(Token<'a>),

    #[error(transparent)]
    TokenizerError(#[from] TokenizerError),
}

pub type ValueMap<'a> = HashMap<&'a str, HuonValue<'a>>;

pub struct Parser<'a> {
    tokenizer: Peekable<Tokenizer<'a>>,
    collapse: usize,
    options: DecoderOptions,
}

impl<'a> Parser<'a> {
    #[must_use]
    pub fn new(tokenizer: Tokenizer<'a>, options: DecoderOptions) -> Self {
        Self {
            tokenizer: tokenizer.peekable(),
            collapse: 0,
            options,
        }
    }

    pub fn parse(tokenizer: Tokenizer<'a>, options: DecoderOptions) -> Result<'a, ValueMap<'a>> {
        let mut parser = Self::new(tokenizer, options);
        parser.parse_object(0)
    }

    /// A helper func to check if a token is whitespace with the expected indentation.
    /// If found, it consumes the token and returns true.
    /// Otherwise, it returns false, or an error if the indentation is greater.
    fn check_indentation(&mut self, token: Token<'a>, expected_indent: usize) -> Result<'a, bool> {
        if let Token::WhiteSpace(n) = token {
            let indent = n / self.options.indent as usize;
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

        while let Some(Ok(token)) = self.peek() {
            if self.collapse > 0 {
                self.collapse -= 1;
                return Ok(map);
            }

            if let Token::NewLine = token {
                self.advance()?;

                let next_token = match self.peek() {
                    Some(token) => token?,
                    None => break,
                };

                match next_token {
                    Token::WhiteSpace(n)
                        if expected_indent > 0
                            && (n / self.options.indent as usize) < expected_indent =>
                    {
                        self.collapse = expected_indent - (n / self.options.indent as usize) - 1;
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

            let value = match self.peek().unwrap()? {
                Token::WhiteSpace(1) => {
                    self.advance()?; // consume whitespace

                    if self.peek().unwrap()? == Token::ListStart {
                        HuonValue::List(self.parse_list()?)
                    } else {
                        self.parse_value()?
                    }
                }

                Token::NewLine => {
                    self.advance()?;

                    match self.peek().unwrap()? {
                        Token::WhiteSpace(n)
                            if (n / self.options.indent as usize) > expected_indent =>
                        {
                            self.advance()?;
                            HuonValue::Object(self.parse_object(n / self.options.indent as usize)?)
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

    fn parse_list(&mut self) -> Result<'a, Vec<HuonValue<'a>>> {
        let mut list = Vec::new();

        self.advance()?; // consume ListStart

        while let Some(Ok(token)) = self.peek() {
            match token {
                Token::ListEnd => {
                    self.advance()?; // consume ListEnd
                    break;
                }

                Token::WhiteSpace(_) | Token::NewLine => {
                    self.advance()?; // consume whitespace
                }

                Token::Separator => {
                    self.advance()?; // consume Separator
                }

                _ => {
                    let value = self.parse_value()?;
                    list.push(value);
                }
            }
        }

        Ok(list)
    }

    fn peek(&mut self) -> Option<Result<'a, Token<'a>>> {
        self.tokenizer.peek().map(|res| res.clone().map_err(Into::into))
    }

    fn advance(&mut self) -> Result<'a, Token<'a>> {
        self.tokenizer.next().unwrap().map_err(Into::into)
    }
}

pub fn parse(
    input: &str,
    options: DecoderOptions,
) -> std::result::Result<ValueMap<'_>, ParserError<'_>> {
    let tokenizer = crate::tokenizer::Tokenizer::new(input);

    Parser::parse(tokenizer, options)
}

#[cfg(test)]
mod tests {
    use indoc::indoc;

    use super::*;

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
    fn test_parser_list_newline() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let map = parse(
            indoc! {"numbers: [
                        -3.5
                        2.5
                        1.1
                    ]"},
            DecoderOptions::default(),
        )?;

        let expected = map! {
            "numbers" => HuonValue::List(vec![
                HuonValue::Float(-3.5),
                HuonValue::Float(2.5),
                HuonValue::Float(1.1),
            ])
        };

        assert_eq!(map, expected);

        Ok(())
    }

    #[test]
    fn test_parser_list_spaced() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let map = parse("numbers: [-3.5 2.5 1.1]", DecoderOptions::default())?;

        let expected = map! {
            "numbers" => HuonValue::List(vec![
                HuonValue::Float(-3.5),
                HuonValue::Float(2.5),
                HuonValue::Float(1.1),
            ])
        };

        assert_eq!(map, expected);

        Ok(())
    }

    #[test]
    fn test_parser() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let map = parse(include_str!("../../test.huon"), DecoderOptions::default())?;

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
    fn fail_int_before_ident() {
        let err =
            parse("1job1: \"swe\"", DecoderOptions::default()).unwrap_err();

        assert_eq!(err, ParserError::InvalidToken(Token::Int(1)));
    }
}
