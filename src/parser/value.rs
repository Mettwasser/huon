use std::collections::HashMap;

/// Cloning is fairly cheap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HuonValue<'a> {
    String(&'a str),
    Int(i64),
    Boolean(bool),
    Object(HashMap<&'a str, HuonValue<'a>>),
}
