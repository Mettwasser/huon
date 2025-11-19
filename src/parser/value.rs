use std::{collections::HashMap, ops::Index};

/// Cloning is fairly cheap.
#[derive(Debug, Clone, PartialEq)]
pub enum HuonValue<'a> {
    // String types
    String(&'a str),

    // Numeric types
    Int(i64),
    Float(f64),

    // Bool types
    Boolean(bool),

    // Null types
    Null,

    List(Vec<HuonValue<'a>>),

    // Composite types
    Object(HashMap<&'a str, HuonValue<'a>>),
}

impl<'a> Index<&'_ str> for HuonValue<'a> {
    type Output = HuonValue<'a>;

    fn index(&self, index: &'_ str) -> &Self::Output {
        match self {
            HuonValue::Object(map) => &map[index],
            _ => panic!("Not an object"),
        }
    }
}
