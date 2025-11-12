#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub enum Token<'a> {
    Identifier(&'a str),
    Str(&'a str),
    Int(i64),
    Float(f64),
    NewLine,
    WhiteSpace(usize),
    Boolean(bool),
    Null,
}
