use std::fmt::Display;
use std::result::Result as StdResult;

use serde::de;

use crate::tokenizer::TokenizerError;
use crate::tokenizer::token::Token;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error<'a> {
    #[error("{}", _0)]
    Custom(String),

    #[error("EOF")]
    Eof,

    Tokenizer(#[from] TokenizerError),

    #[error("Invalid token: {:?}", _0)]
    InvalidToken(Token<'a>),
}

impl de::Error for Error<'_> {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Self::Custom(msg.to_string())
    }
}

pub type Result<'a, T> = StdResult<T, Error<'a>>;
