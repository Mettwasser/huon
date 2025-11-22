use std::collections::{VecDeque, hash_map};

use crate::{
    DecoderOptions,
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
            HuonValue::String(s) => visitor.visit_borrowed_str(s),
            HuonValue::Float(f) => visitor.visit_f64(f),
            HuonValue::Null => visitor.visit_none(),
            HuonValue::Object(map) => visitor.visit_map(MapDeserializer::new(map)),
            HuonValue::List(list) => visitor.visit_seq(SequenceDeserializer {
                sequence: VecDeque::from(list),
            }),
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
            HuonValue::String(s) => visitor.visit_string(s.to_string()),
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

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            HuonValue::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    forward_to_deserialize_any! {
        i8 i16 i32 u8 u16 u32 u64 f32 f64 char bytes byte_buf unit unit_struct
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
                self.next_value = Some(value);
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
                let value_deserializer = HuonDeserializer { value };
                seed.deserialize(value_deserializer)
            }
            None => Err(de::Error::custom(
                "Called next_value_seed before next_key_seed",
            )),
        }
    }
}

struct SequenceDeserializer<'de> {
    sequence: VecDeque<HuonValue<'de>>,
}

impl<'de> de::SeqAccess<'de> for SequenceDeserializer<'de> {
    type Error = de::value::Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: de::DeserializeSeed<'de>,
    {
        self.sequence
            .pop_front()
            .map(|val| {
                let value_deserializer = HuonDeserializer { value: val };
                seed.deserialize(value_deserializer)
            })
            .transpose()
    }
}

#[derive(Debug)]
pub enum HuonDeserializeError<'de> {
    SerdeError(serde::de::value::Error),
    ParserError(crate::parser::ParserError<'de>),
    TokenizerError(crate::tokenizer::TokenizerError),
}

pub fn from_str<'de, T>(
    s: &'de str,
    options: DecoderOptions,
) -> Result<T, HuonDeserializeError<'de>>
where
    T: Deserialize<'de>,
{
    let tokens = Tokenizer::tokenize(s).map_err(HuonDeserializeError::TokenizerError)?;

    let parsed = Parser::parse(tokens, options).map_err(HuonDeserializeError::ParserError)?;

    let value_tree = HuonValue::Object(parsed);

    let deserializer = HuonDeserializer { value: value_tree };

    T::deserialize(deserializer).map_err(HuonDeserializeError::SerdeError)
}

#[cfg(test)]
#[allow(unused)]
mod tests {
    use indoc::indoc;
    use pretty_assertions::assert_eq;

    use crate::{
        test_list_model::{CodeInfo, TestCodes},
        test_model::{Job, JobCategory, JobInfo, NewType, PayRate, Person},
    };

    use super::*;

    #[test]
    fn test_deserialization() {
        let input = include_str!("../test.huon").to_owned();

        let person: Person =
            from_str(&input, DecoderOptions::default()).expect("Deserialization failed");

        let expected_person = Person {
            name: "John",
            last_name: "Doe",
            age: 32,
            job1: Job {
                category: JobCategory {
                    name: NewType("IT"),
                },
                info: JobInfo {
                    pay: -4200.50,
                    payrate: PayRate {
                        iteration: "monthly",
                        date: "Last Friday of every month",
                        monthly_increase: Some("5%"),
                    },
                },
                name: "Software Engineer",
            },
            job2: Job {
                category: JobCategory {
                    name: NewType("Security"),
                },
                info: JobInfo {
                    pay: 3700_f64,
                    payrate: PayRate {
                        iteration: "weekly",
                        date: "Every Friday",
                        monthly_increase: None,
                    },
                },
                name: "Bodyguard",
            },
        };

        assert_eq!(person, expected_person);
    }

    #[test]
    fn test_deserialization_new_list() {
        let input = include_str!("../test_list.huon").to_owned();

        let code_info: CodeInfo =
            from_str(&input, DecoderOptions::default()).expect("Deserialization failed");

        let expected_code_info = CodeInfo {
            test_codes: TestCodes {
                codes: vec![111.1, 333.3, 555.5],
                info: "Passwords".to_string(),
            },
            name: "General Access".to_string(),
        };

        assert_eq!(code_info, expected_code_info);
    }

    #[test]
    fn test_deserialization_list_indent_2() {
        let input = indoc! {r#"
            test_codes:
              codes: [111.1 333.3 555.5]
              info: "Passwords"
            name: "General Access""#}
        .to_owned();

        let code_info: CodeInfo =
            from_str(&input, DecoderOptions { indent: 2 }).expect("Deserialization failed");

        let expected_code_info = CodeInfo {
            test_codes: TestCodes {
                codes: vec![111.1, 333.3, 555.5],
                info: "Passwords".to_string(),
            },
            name: "General Access".to_string(),
        };

        assert_eq!(code_info, expected_code_info);
    }
}
