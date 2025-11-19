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
    ListStart,
    ListEnd,
    Separator,
}

impl Token<'_> {
    pub fn is_value(&self) -> bool {
        matches!(
            self,
            Token::Identifier(_)
                | Token::Str(_)
                | Token::Int(_)
                | Token::Float(_)
                | Token::Boolean(_)
                | Token::Null
        )
    }
}
