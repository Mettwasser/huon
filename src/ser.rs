use serde::ser::{self, Serialize, SerializeMap, Serializer};
use std::fmt::Display;
use std::io;

#[derive(Debug, thiserror::Error)]
pub enum HuonSerializeError {
    #[error(transparent)]
    Io(io::Error),
    #[error("{_0}")]
    Custom(String),
}

impl ser::Error for HuonSerializeError {
    fn custom<T: Display>(msg: T) -> Self {
        HuonSerializeError::Custom(msg.to_string())
    }
}

impl From<io::Error> for HuonSerializeError {
    fn from(e: io::Error) -> Self {
        HuonSerializeError::Io(e)
    }
}

pub struct HuonSerializer<W: io::Write> {
    writer: W,
    indent_level: usize,
    is_key: bool,
    is_root: bool,
    key_pending: bool,
}

impl<W: io::Write> HuonSerializer<W> {
    pub fn new(writer: W) -> Self {
        HuonSerializer {
            writer,
            indent_level: 0,
            is_key: false,
            is_root: true,
            key_pending: false,
        }
    }

    fn write_indent(&mut self) -> Result<(), HuonSerializeError> {
        write!(self.writer, "{}", "    ".repeat(self.indent_level))?;
        Ok(())
    }

    fn write_non_map_value_separator(&mut self) -> Result<(), HuonSerializeError> {
        if self.key_pending {
            self.writer.write_all(b": ")?;
            self.key_pending = false;
        }
        Ok(())
    }

    fn write_map_value_separator(&mut self) -> Result<(), HuonSerializeError> {
        if self.key_pending {
            self.writer.write_all(b":")?;
            self.key_pending = false;
        }
        Ok(())
    }
}

impl<'a, W: io::Write> Serializer for &'a mut HuonSerializer<W> {
    type Ok = ();

    type Error = HuonSerializeError;

    type SerializeMap = HuonMapSerializer<'a, W>;

    type SerializeStruct = Self::SerializeMap;

    type SerializeSeq = ser::Impossible<(), HuonSerializeError>;

    type SerializeTuple = ser::Impossible<(), HuonSerializeError>;

    type SerializeTupleStruct = ser::Impossible<(), HuonSerializeError>;

    type SerializeTupleVariant = ser::Impossible<(), HuonSerializeError>;

    type SerializeStructVariant = ser::Impossible<(), HuonSerializeError>;

    fn serialize_bool(self, v: bool) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_i8(self, v: i8) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_i16(self, v: i16) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_i32(self, v: i32) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_i64(self, v: i64) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_u8(self, _v: u8) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Unsigned integers are not supported in Huon".to_string(),
        ))
    }

    fn serialize_u16(self, _v: u16) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Unsigned integers are not supported in Huon".to_string(),
        ))
    }

    fn serialize_u32(self, _v: u32) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Unsigned integers are not supported in Huon".to_string(),
        ))
    }

    fn serialize_u64(self, _v: u64) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Unsigned integers are not supported in Huon".to_string(),
        ))
    }

    fn serialize_f32(self, v: f32) -> Result<Self::Ok, Self::Error> {
        self.serialize_f64(v as f64)
    }

    fn serialize_f64(self, v: f64) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_char(self, v: char) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{v}")?;
        Ok(())
    }

    fn serialize_str(self, v: &str) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        if self.is_key {
            write!(self.writer, "{v}")?;
        } else {
            write!(self.writer, "\"{v}\"")?;
        }
        Ok(())
    }

    fn serialize_bytes(self, _v: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Byte arrays are not supported in Huon".to_string(),
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "null")?;
        Ok(())
    }

    fn serialize_some<T>(self, value: &T) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Unit is not supported in huon".to_string(),
        ))
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Unit structs are not supported in huon".to_string(),
        ))
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        self.write_non_map_value_separator()?;
        write!(self.writer, "{variant_index}")?;
        Ok(())
    }

    fn serialize_newtype_struct<T>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Sequences are not supported in huon".to_string(),
        ))
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Tuples are not supported in huon".to_string(),
        ))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Tuple structs are not supported in huon".to_string(),
        ))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Tuple variants are not supported in huon".to_string(),
        ))
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        self.write_map_value_separator()?;

        if !self.is_root {
            self.writer.write_all(b"\n")?;
            self.indent_level += 1;
        } else {
            self.is_root = false;
        }

        Ok(Self::SerializeMap::new(self))
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        self.serialize_map(Some(len))
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(HuonSerializeError::Custom(
            "Serializing struct variants is not supported".to_string(),
        ))
    }
}

pub struct HuonMapSerializer<'a, W: io::Write> {
    ser: &'a mut HuonSerializer<W>,
    first: bool,
}

impl<'a, W: io::Write> HuonMapSerializer<'a, W> {
    pub fn new(ser: &'a mut HuonSerializer<W>) -> HuonMapSerializer<'a, W> {
        HuonMapSerializer { ser, first: true }
    }
}

impl<'a, W: io::Write> SerializeMap for HuonMapSerializer<'a, W> {
    type Ok = ();
    type Error = HuonSerializeError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        if !self.first {
            self.ser.writer.write_all(b"\n")?;
        }
        self.first = false;

        self.ser.write_indent()?;

        self.ser.is_key = true;
        key.serialize(&mut *self.ser)?;
        self.ser.is_key = false;

        self.ser.key_pending = true;
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        value.serialize(&mut *self.ser)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.ser.indent_level > 0 {
            self.ser.indent_level -= 1;
        }
        Ok(())
    }
}

impl<'a, W: io::Write> ser::SerializeStruct for HuonMapSerializer<'a, W> {
    type Ok = ();
    type Error = HuonSerializeError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        self.serialize_key(key)?;
        self.serialize_value(value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeMap::end(self)
    }
}

pub fn to_string<T>(value: &T) -> Result<String, HuonSerializeError>
where
    T: ?Sized + Serialize,
{
    let mut vec = Vec::new();
    let mut serializer = HuonSerializer {
        writer: &mut vec,
        indent_level: 0,
        is_key: false,
        is_root: true,
        key_pending: false,
    };

    value.serialize(&mut serializer)?;

    String::from_utf8(vec).map_err(|e| HuonSerializeError::Custom(e.to_string()))
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use crate::test_model::{Job, JobCategory, JobInfo, NewType, PayRate, Person};

    use super::*;

    #[test]
    fn test_serialize_struct() {
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

        let s = to_string(&expected_person).unwrap();

        let expected = include_str!("../test.huon");

        assert_eq!(s, expected);
    }
}
