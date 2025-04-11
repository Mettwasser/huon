#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Token<'a> {
    Identifier(&'a str),
    Str(&'a str),
    Int(i64),
    NewLine,
    WhiteSpace(usize),
    Boolean(bool),
}
