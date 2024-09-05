use crate::error::{Error, ErrorCode, Result};
use crate::value_no_obj::Map;
use crate::value_no_obj::{to_value, ValueNoObj};
use alloc::borrow::ToOwned;
use alloc::string::{String, ToString};
use alloc::vec::Vec;
use core::fmt::Display;
use core::result;
use serde::de;
use serde::de::Unexpected;
use serde::ser::{Impossible, Serialize};

impl Serialize for ValueNoObj {
    #[inline]
    fn serialize<S>(&self, serializer: S) -> result::Result<S::Ok, S::Error>
    where
        S: ::serde::Serializer,
    {
        match self {
            ValueNoObj::Null => serializer.serialize_unit(),
            ValueNoObj::Bool(b) => serializer.serialize_bool(*b),
            ValueNoObj::Number(n) => n.serialize(serializer),
            ValueNoObj::String(s) => serializer.serialize_str(s),
            ValueNoObj::Array(v) => v.serialize(serializer),
        }
    }
}

/// Serializer whose output is a `ValueNoObj`.
///
/// This is the serializer that backs [`serde_json::to_value`][crate::to_value].
/// Unlike the main serde_json serializer which goes from some serializable
/// value of type `T` to JSON text, this one goes from `T` to
/// `serde_json::ValueNoObj`.
///
/// The `to_value` function is implementable as:
///
/// ```
/// use serde::Serialize;
/// use serde_json::{Error, ValueNoObj};
///
/// pub fn to_value<T>(input: T) -> Result<ValueNoObj, Error>
/// where
///     T: Serialize,
/// {
///     input.serialize(serde_json::value::Serializer)
/// }
/// ```
pub struct Serializer;

impl serde::Serializer for Serializer {
    type Ok = ValueNoObj;
    type Error = Error;

    type SerializeSeq = SerializeVec;
    type SerializeTuple = SerializeVec;
    type SerializeTupleStruct = SerializeVec;
    type SerializeTupleVariant = SerializeTupleVariant;
    type SerializeMap = SerializeMap;
    type SerializeStruct = SerializeMap;
    type SerializeStructVariant = SerializeStructVariant;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<ValueNoObj> {
        Ok(ValueNoObj::Bool(value))
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<ValueNoObj> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<ValueNoObj> {
        self.serialize_i64(value as i64)
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<ValueNoObj> {
        self.serialize_i64(value as i64)
    }

    fn serialize_i64(self, value: i64) -> Result<ValueNoObj> {
        Ok(ValueNoObj::Number(value.into()))
    }

    fn serialize_i128(self, value: i128) -> Result<ValueNoObj> {
        #[cfg(feature = "arbitrary_precision")]
        {
            Ok(ValueNoObj::Number(value.into()))
        }

        #[cfg(not(feature = "arbitrary_precision"))]
        {
            if let Ok(value) = u64::try_from(value) {
                Ok(ValueNoObj::Number(value.into()))
            } else if let Ok(value) = i64::try_from(value) {
                Ok(ValueNoObj::Number(value.into()))
            } else {
                Err(Error::syntax(ErrorCode::NumberOutOfRange, 0, 0))
            }
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<ValueNoObj> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<ValueNoObj> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<ValueNoObj> {
        self.serialize_u64(value as u64)
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<ValueNoObj> {
        Ok(ValueNoObj::Number(value.into()))
    }

    fn serialize_u128(self, value: u128) -> Result<ValueNoObj> {
        #[cfg(feature = "arbitrary_precision")]
        {
            Ok(ValueNoObj::Number(value.into()))
        }

        #[cfg(not(feature = "arbitrary_precision"))]
        {
            if let Ok(value) = u64::try_from(value) {
                Ok(ValueNoObj::Number(value.into()))
            } else {
                Err(Error::syntax(ErrorCode::NumberOutOfRange, 0, 0))
            }
        }
    }

    #[inline]
    fn serialize_f32(self, float: f32) -> Result<ValueNoObj> {
        Ok(ValueNoObj::from(float))
    }

    #[inline]
    fn serialize_f64(self, float: f64) -> Result<ValueNoObj> {
        Ok(ValueNoObj::from(float))
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<ValueNoObj> {
        let mut s = String::new();
        s.push(value);
        Ok(ValueNoObj::String(s))
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<ValueNoObj> {
        Ok(ValueNoObj::String(value.to_owned()))
    }

    fn serialize_bytes(self, value: &[u8]) -> Result<ValueNoObj> {
        let vec = value
            .iter()
            .map(|&b| ValueNoObj::Number(b.into()))
            .collect();
        Ok(ValueNoObj::Array(vec))
    }

    #[inline]
    fn serialize_unit(self) -> Result<ValueNoObj> {
        Ok(ValueNoObj::Null)
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<ValueNoObj> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<ValueNoObj> {
        self.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<ValueNoObj>
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
        _value: &T,
    ) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::Map,
            &"`Object` isn't supported",
        ))
    }

    #[inline]
    fn serialize_none(self) -> Result<ValueNoObj> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        Ok(SerializeVec {
            vec: Vec::with_capacity(len.unwrap_or(0)),
        })
    }

    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Ok(SerializeTupleVariant {
            vec: Vec::with_capacity(len),
        })
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Ok(SerializeMap::Map {
            map: Map::new(),
            next_key: None,
        })
    }

    fn serialize_struct(self, name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        match name {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(SerializeMap::Number { out_value: None }),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(SerializeMap::RawValueNoObj { out_value: None }),
            _ => self.serialize_map(Some(len)),
        }
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(de::Error::invalid_type(
            Unexpected::NewtypeStruct,
            &"`Object` isn't supported",
        ))
    }

    fn collect_str<T>(self, value: &T) -> Result<ValueNoObj>
    where
        T: ?Sized + Display,
    {
        Ok(ValueNoObj::String(value.to_string()))
    }
}

pub struct SerializeVec {
    vec: Vec<ValueNoObj>,
}

pub struct SerializeTupleVariant {
    vec: Vec<ValueNoObj>,
}

pub enum SerializeMap {
    Map {
        map: Map<String, ValueNoObj>,
        next_key: Option<String>,
    },
    #[cfg(feature = "arbitrary_precision")]
    Number { out_value: Option<ValueNoObj> },
    #[cfg(feature = "raw_value")]
    RawValueNoObj { out_value: Option<ValueNoObj> },
}

pub struct SerializeStructVariant;

impl serde::ser::SerializeSeq for SerializeVec {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(tri!(to_value(value)));
        Ok(())
    }

    fn end(self) -> Result<ValueNoObj> {
        Ok(ValueNoObj::Array(self.vec))
    }
}

impl serde::ser::SerializeTuple for SerializeVec {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<ValueNoObj> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SerializeVec {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        serde::ser::SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<ValueNoObj> {
        serde::ser::SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleVariant for SerializeTupleVariant {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        self.vec.push(tri!(to_value(value)));
        Ok(())
    }

    fn end(self) -> Result<ValueNoObj> {
        Ok(ValueNoObj::Array(self.vec))
    }
}

impl serde::ser::SerializeMap for SerializeMap {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { next_key, .. } => {
                *next_key = Some(tri!(key.serialize(MapKeySerializer)));
                Ok(())
            }
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValueNoObj { .. } => unreachable!(),
        }
    }

    fn serialize_value<T>(&mut self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { .. } => Err(serde::de::Error::invalid_type(
                Unexpected::Map,
                &"Object aren't supported",
            )),
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValueNoObj { .. } => unreachable!(),
        }
    }

    fn end(self) -> Result<ValueNoObj> {
        match self {
            SerializeMap::Map { .. } => Err(serde::de::Error::invalid_type(
                Unexpected::Map,
                &"Object aren't supported",
            )),
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { .. } => unreachable!(),
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValueNoObj { .. } => unreachable!(),
        }
    }
}

struct MapKeySerializer;

fn key_must_be_a_string() -> Error {
    Error::syntax(ErrorCode::KeyMustBeAString, 0, 0)
}

fn float_key_must_be_finite() -> Error {
    Error::syntax(ErrorCode::FloatKeyMustBeFinite, 0, 0)
}

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = Error;

    type SerializeSeq = Impossible<String, Error>;
    type SerializeTuple = Impossible<String, Error>;
    type SerializeTupleStruct = Impossible<String, Error>;
    type SerializeTupleVariant = Impossible<String, Error>;
    type SerializeMap = Impossible<String, Error>;
    type SerializeStruct = Impossible<String, Error>;
    type SerializeStructVariant = Impossible<String, Error>;

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<String> {
        Ok(variant.to_owned())
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_bool(self, value: bool) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i8(self, value: i8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i16(self, value: i16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i32(self, value: i32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_i64(self, value: i64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u8(self, value: u8) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u16(self, value: u16) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u32(self, value: u32) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_u64(self, value: u64) -> Result<String> {
        Ok(value.to_string())
    }

    fn serialize_f32(self, value: f32) -> Result<String> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    fn serialize_f64(self, value: f64) -> Result<String> {
        if value.is_finite() {
            Ok(ryu::Buffer::new().format_finite(value).to_owned())
        } else {
            Err(float_key_must_be_finite())
        }
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<String> {
        Ok({
            let mut s = String::new();
            s.push(value);
            s
        })
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<String> {
        Ok(value.to_owned())
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<String> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<String>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<String>
    where
        T: ?Sized + Display,
    {
        Ok(value.to_string())
    }
}

impl serde::ser::SerializeStruct for SerializeMap {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::serialize_entry(self, key, value),
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { out_value } => {
                if key == crate::number::TOKEN {
                    *out_value = Some(tri!(value.serialize(NumberValueNoObjEmitter)));
                    Ok(())
                } else {
                    Err(invalid_number())
                }
            }
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValueNoObj { out_value } => {
                if key == crate::raw::TOKEN {
                    *out_value = Some(tri!(value.serialize(RawValueNoObjEmitter)));
                    Ok(())
                } else {
                    Err(invalid_raw_value())
                }
            }
        }
    }

    fn end(self) -> Result<ValueNoObj> {
        match self {
            SerializeMap::Map { .. } => serde::ser::SerializeMap::end(self),
            #[cfg(feature = "arbitrary_precision")]
            SerializeMap::Number { out_value, .. } => {
                Ok(out_value.expect("number value was not emitted"))
            }
            #[cfg(feature = "raw_value")]
            SerializeMap::RawValueNoObj { out_value, .. } => {
                Ok(out_value.expect("raw value was not emitted"))
            }
        }
    }
}

impl serde::ser::SerializeStructVariant for SerializeStructVariant {
    type Ok = ValueNoObj;
    type Error = Error;

    fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(serde::de::Error::invalid_type(
            Unexpected::Map,
            &"Object unsupported",
        ))
    }

    fn end(self) -> Result<ValueNoObj> {
        Err(serde::de::Error::invalid_type(
            Unexpected::Map,
            &"Object unsupported",
        ))
    }
}

#[cfg(feature = "arbitrary_precision")]
struct NumberValueNoObjEmitter;

#[cfg(feature = "arbitrary_precision")]
fn invalid_number() -> Error {
    Error::syntax(ErrorCode::InvalidNumber, 0, 0)
}

#[cfg(feature = "arbitrary_precision")]
impl serde::ser::Serializer for NumberValueNoObjEmitter {
    type Ok = ValueNoObj;
    type Error = Error;

    type SerializeSeq = Impossible<ValueNoObj, Error>;
    type SerializeTuple = Impossible<ValueNoObj, Error>;
    type SerializeTupleStruct = Impossible<ValueNoObj, Error>;
    type SerializeTupleVariant = Impossible<ValueNoObj, Error>;
    type SerializeMap = Impossible<ValueNoObj, Error>;
    type SerializeStruct = Impossible<ValueNoObj, Error>;
    type SerializeStructVariant = Impossible<ValueNoObj, Error>;

    fn serialize_bool(self, _v: bool) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_i8(self, _v: i8) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_i16(self, _v: i16) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_i32(self, _v: i32) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_i64(self, _v: i64) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_u8(self, _v: u8) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_u16(self, _v: u16) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_u32(self, _v: u32) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_u64(self, _v: u64) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_f32(self, _v: f32) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_f64(self, _v: f64) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_char(self, _v: char) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_str(self, value: &str) -> Result<ValueNoObj> {
        let n = tri!(value.to_owned().parse());
        Ok(ValueNoObj::Number(n))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_none(self) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(invalid_number())
    }

    fn serialize_unit(self) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<ValueNoObj> {
        Err(invalid_number())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(invalid_number())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(invalid_number())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(invalid_number())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(invalid_number())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(invalid_number())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(invalid_number())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(invalid_number())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(invalid_number())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(invalid_number())
    }
}

#[cfg(feature = "raw_value")]
struct RawValueNoObjEmitter;

#[cfg(feature = "raw_value")]
fn invalid_raw_value() -> Error {
    Error::syntax(ErrorCode::ExpectedSomeValue, 0, 0)
}

#[cfg(feature = "raw_value")]
impl serde::ser::Serializer for RawValueNoObjEmitter {
    type Ok = ValueNoObj;
    type Error = Error;

    type SerializeSeq = Impossible<ValueNoObj, Error>;
    type SerializeTuple = Impossible<ValueNoObj, Error>;
    type SerializeTupleStruct = Impossible<ValueNoObj, Error>;
    type SerializeTupleVariant = Impossible<ValueNoObj, Error>;
    type SerializeMap = Impossible<ValueNoObj, Error>;
    type SerializeStruct = Impossible<ValueNoObj, Error>;
    type SerializeStructVariant = Impossible<ValueNoObj, Error>;

    fn serialize_bool(self, _v: bool) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_i8(self, _v: i8) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_i16(self, _v: i16) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_i32(self, _v: i32) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_i64(self, _v: i64) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_u8(self, _v: u8) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_u16(self, _v: u16) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_u32(self, _v: u32) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_u64(self, _v: u64) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_f32(self, _v: f32) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_f64(self, _v: f64) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_char(self, _v: char) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_str(self, value: &str) -> Result<ValueNoObj> {
        crate::from_str(value)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_none(self) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(invalid_raw_value())
    }

    fn serialize_unit(self) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<ValueNoObj> {
        Err(invalid_raw_value())
    }

    fn serialize_newtype_struct<T>(self, _name: &'static str, _value: &T) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(invalid_raw_value())
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<ValueNoObj>
    where
        T: ?Sized + Serialize,
    {
        Err(invalid_raw_value())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(invalid_raw_value())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(invalid_raw_value())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(invalid_raw_value())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(invalid_raw_value())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(invalid_raw_value())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(invalid_raw_value())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(invalid_raw_value())
    }

    fn collect_str<T>(self, value: &T) -> Result<Self::Ok>
    where
        T: ?Sized + Display,
    {
        self.serialize_str(&value.to_string())
    }
}
