use std::{collections::HashMap, ops::Index};

/// Cloning is fairly cheap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HuonValue<'a> {
    String(&'a str),
    Int(i64),
    Boolean(bool),
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
