use std::collections::hash_map;

use crate::{
    parser::{Parser, ValueMap, value::HuonValue},
    tokenizer::Tokenizer,
};
use serde::{
    Deserialize, Deserializer,
    de::{self, Visitor},
    forward_to_deserialize_any,
};

pub struct HuonDeserializer<'de> {
    value: HuonValue<'de>,
}

impl<'de> Deserializer<'de> for HuonDeserializer<'de> {
    type Error = serde::de::value::Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            HuonValue::Boolean(b) => visitor.visit_bool(b),
            HuonValue::Int(i) => visitor.visit_i64(i),
            // `s` is `&'de str`, `visit_borrowed_str` expects `&'de str`. This is correct.
            HuonValue::String(s) => visitor.visit_borrowed_str(s),
            // `map` is `&'de ValueMap<'de>`.
            HuonValue::Object(map) => visitor.visit_map(MapDeserializer::new(map)),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            HuonValue::Boolean(b) => visitor.visit_bool(b),
            _ => Err(de::Error::custom("Expected bool")),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            HuonValue::Int(i) => visitor.visit_i64(i),
            _ => Err(de::Error::custom("Expected i64")),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            HuonValue::String(s) => visitor.visit_borrowed_str(s),
            _ => Err(de::Error::custom("Expected string")),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            HuonValue::String(s) => visitor.visit_borrowed_str(s),
            _ => Err(de::Error::custom("Expected str")),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            // `map` is `&'de ValueMap<'de>`, so `MapDeserializer::new(map)` is correct.
            HuonValue::Object(map) => visitor.visit_map(MapDeserializer::new(map)),
            _ => Err(de::Error::custom("Expected map")),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_newtype_struct(self)
    }

    forward_to_deserialize_any! {
        i8 i16 i32 u8 u16 u32 u64 f32 f64 char bytes byte_buf option unit unit_struct
        seq tuple tuple_struct enum identifier ignored_any
    }
}

struct MapDeserializer<'de> {
    iter: hash_map::IntoIter<&'de str, HuonValue<'de>>,
    next_value: Option<HuonValue<'de>>,
}

impl<'de> MapDeserializer<'de> {
    fn new(map: ValueMap<'de>) -> Self {
        Self {
            iter: map.into_iter(),
            next_value: None,
        }
    }
}

// The MapAccess impl is for 'de
impl<'de> de::MapAccess<'de> for MapDeserializer<'de> {
    type Error = de::value::Error;

    fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>, Self::Error>
    where
        K: de::DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some((key, value)) => {
                // store the owned value for the subsequent `next_value_seed` call
                self.next_value = Some(value);
                // key is `&'de str` (owned pointer), pass it directly
                let key_deserializer = de::IntoDeserializer::into_deserializer(key);
                seed.deserialize(key_deserializer).map(Some)
            }
            None => Ok(None),
        }
    }

    fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value, Self::Error>
    where
        V: de::DeserializeSeed<'de>,
    {
        match self.next_value.take() {
            Some(value) => {
                // `value` is `HuonValue<'de>` (owned). Create a deserializer that owns it.
                let value_deserializer = HuonDeserializer { value };
                seed.deserialize(value_deserializer)
            }
            None => Err(de::Error::custom(
                "Called next_value_seed before next_key_seed",
            )),
        }
    }
}

#[derive(Debug)]
pub enum HuonDeserializeError<'de> {
    SerdeError(serde::de::value::Error),
    ParserError(crate::parser::ParserError<'de>),
    TokenizerError(crate::tokenizer::TokenizerError),
}

pub fn from_str<'de, T>(s: &'de str) -> Result<T, HuonDeserializeError<'de>>
where
    T: Deserialize<'de>,
{
    let tokens = Tokenizer::scan(s).map_err(|err| HuonDeserializeError::TokenizerError(err))?;

    let parsed = Parser::parse(tokens).map_err(|err| HuonDeserializeError::ParserError(err))?;

    let value_tree = HuonValue::Object(parsed);

    let deserializer = HuonDeserializer { value: value_tree };

    Ok(T::deserialize(deserializer).map_err(|err| HuonDeserializeError::SerdeError(err))?)
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use super::*;

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct NewType<'a>(&'a str);

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct JobCategory<'a> {
        #[serde(borrow)]
        name: NewType<'a>,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct PayRate<'a> {
        iteration: &'a str,
        date: &'a str,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct JobInfo<'a> {
        pay: i64,
        #[serde(borrow)]
        payrate: PayRate<'a>,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Job<'a> {
        category: JobCategory<'a>,
        info: JobInfo<'a>,
        name: &'a str,
    }

    #[derive(Debug, Deserialize, PartialEq, Eq)]
    struct Person<'a> {
        name: &'a str,
        age: i64,
        job: Job<'a>,
    }

    #[test]
    fn test_deserialization() {
        let input = include_str!("../simple.huon").to_owned();

        let person: Person = from_str(&input).expect("Deserialization failed");

        let expected_person = Person {
            name: "John",
            age: 32,
            job: Job {
                category: JobCategory {
                    name: NewType("IT"),
                },
                info: JobInfo {
                    pay: 4200,
                    payrate: PayRate {
                        iteration: "monthly",
                        date: "Last Friday of every month",
                    },
                },
                name: "Software Engineer",
            },
        };

        assert_eq!(person, expected_person);
    }
}
