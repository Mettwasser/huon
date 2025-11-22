pub mod de;
mod error;
pub mod parser;
pub mod ser;
pub mod tokenizer;

#[cfg(test)]
pub mod test_model;

#[cfg(test)]
pub mod test_list_model;

pub use error::{Error, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ListCommaStyle {
    /// No commas at all
    None,

    /// Basic, in between every entry
    Basic,

    /// In between every entry and at the end
    Trailing,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EncoderOptions {
    pub indent: u8,
    pub list_comma_style: ListCommaStyle,
}

impl Default for EncoderOptions {
    fn default() -> Self {
        Self {
            list_comma_style: ListCommaStyle::None,
            indent: 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DecoderOptions {
    pub indent: u8,
}

impl Default for DecoderOptions {
    fn default() -> Self {
        Self { indent: 4 }
    }
}
