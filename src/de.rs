// use {
//     crate::{
//         Error, Result,
//         tokenizer::{Tokenizer, TokenizerError, token::Token},
//     },
//     serde::{de, forward_to_deserialize_any},
//     std::{result::Result as StdResult, str::FromStr},
// };

// pub struct Deserializer<'a, 'de> {
//     tokens: &'a [Token<'de>],
//     cursor: usize,
//     is_root: bool,
//     indentation_level: usize,
// }

// impl<'de> Deserializer<'_, 'de> {
//     fn peek(&mut self) -> Result<'de, Token<'de>> {
//         self.tokens
//             .get(self.cursor)
//             .cloned()
//             .ok_or(TokenizerError::EOF.into())
//     }

//     fn advance(&mut self) -> Result<'de, Token<'de>> {
//         let token = self.peek()?;
//         self.cursor += 1;
//         Ok(token)
//     }

//     /// Advances to the next symbol
//     fn advance_to_symbol(&mut self) -> Result<'de, Token<'de>> {
//         loop {
//             let token = self.advance()?;
//             match token {
//                 Token::WhiteSpace(n) if n % 4 == 0 => {
//                     self.indentation_level = n / 4;
//                     continue;
//                 }
//                 Token::NewLine => {
//                     if let Token::WhiteSpace(n) = self.peek()? {
//                         let maybe_new_indentation = n / 4;

//                     }
//                 }
//                 Token::Boolean(_) | Token::Identifier(_) | Token::Int(_) | Token::Str(_) => {
//                     return Ok(token);
//                 }
//             }
//         }
//     }
// }

// impl<'de> de::Deserializer<'de> for &mut Deserializer<'_, 'de> {
//     type Error = crate::Error<'de>;

//     fn deserialize_any<V>(self, visitor: V) -> Result<'de, V::Value>
//     where
//         V: de::Visitor<'de>,
//     {
//         if self.is_root {
//             self.is_root = false;
//             return self.deserialize_map(visitor);
//         }

//         match self.peek()? {
//             Token::Identifier(s) => visitor.visit_borrowed_str(s),
//             Token::Str(s) => visitor.visit_str(s),
//             Token::Int(i) => visitor.visit_i64(i),
//             Token::Boolean(b) => visitor.visit_bool(b),
//             token => Err(Error::InvalidToken(token)),
//         }
//     }

//     fn deserialize_map<V>(self, visitor: V) -> StdResult<V::Value, Self::Error>
//     where
//         V: de::Visitor<'de>,
//     {
//         let mut map = visitor.visit_map(MapAccess::new(self))?;
//     }

//     forward_to_deserialize_any! {
//         bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 f32 f64 char str string
//         bytes byte_buf option unit unit_struct newtype_struct seq tuple
//         tuple_struct struct enum identifier ignored_any
//     }
// }
// struct MapAccess<'a, 'b: 'a, 'de> {
//     de: &'b mut Deserializer<'a, 'de>,
//     // The indentation level that this map is parsing.
//     current_indent: usize,
// }

// impl<'a, 'b, 'de: 'a> de::MapAccess<'de> for MapAccess<'a, 'b, 'de> {
//     type Error = Error<'de>;

//     /// Attempt to deserialize the next key.
//     fn next_key_seed<K>(&mut self, seed: K) -> Result<'de, Option<K::Value>>
//     where
//         K: de::DeserializeSeed<'de>,
//     {
//         // Peek at the next non-formatting token.
//         let token = self.de.advance_to_symbol()?;

//         // If the indentation is less than the map's expected level, we consider it the end of this mapping.
//         if indent < self.current_indent {
//             return Ok(None);
//         }

//         // Otherwise, treat the token as a key.
//         // Before deserializing the key, update the current indentation if necessary.
//         self.de.current_indent = indent;
//         let key = seed.deserialize(&mut *self.de)?;
//         Ok(Some(key))
//     }

//     /// Deserialize the value corresponding to the previously deserialized key.
//     fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
//     where
//         V: de::DeserializeSeed<'de>,
//     {
//         // After a key, we expect a value. In many cases you might have a colon separator,
//         // which you would verify here. For simplicity we assume the value follows immediately.
//         seed.deserialize(&mut *self.de)
//     }
// }

// pub fn from_str<'de, T>(s: &'de str) -> Result<'de, T>
// where
//     T: de::Deserialize<'de>,
// {
//     let mut tokenizer = Tokenizer::new(s);
//     let tokens = tokenizer.scan()?;

//     let mut deserializer = Deserializer {
//         tokens: &tokens,
//         cursor: 0,
//         is_root: true,
//         indentation_level: 0,
//     };

//     T::deserialize(&mut deserializer)
// }
