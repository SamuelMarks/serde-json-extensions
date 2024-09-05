use alloc::string::String;
#[cfg(feature = "raw_value")]
use alloc::string::ToString;
use alloc::vec::{self, Vec};
use core::fmt;
use core::slice;
use core::str::FromStr;

use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess,
    SeqAccess, Unexpected, VariantAccess, Visitor,
};

#[cfg(feature = "arbitrary_precision")]
use crate::number::NumberFromString;

use crate::error::Error;
use crate::number::Number;
use crate::value_no_obj::ValueNoObj;

impl<'de> Deserialize<'de> for ValueNoObj {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<ValueNoObj, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueNoObjVisitor;

        impl<'de> Visitor<'de> for ValueNoObjVisitor {
            type Value = ValueNoObj;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<ValueNoObj, E> {
                Ok(ValueNoObj::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<ValueNoObj, E> {
                Ok(ValueNoObj::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<ValueNoObj, E> {
                Ok(ValueNoObj::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<ValueNoObj, E> {
                Ok(Number::from_f64(value).map_or(ValueNoObj::Null, ValueNoObj::Number))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<ValueNoObj, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_string<E>(self, value: String) -> Result<ValueNoObj, E> {
                Ok(ValueNoObj::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<ValueNoObj, E> {
                Ok(ValueNoObj::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<ValueNoObj, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<ValueNoObj, E> {
                Ok(ValueNoObj::Null)
            }

            #[inline]
            fn visit_seq<V>(self, mut visitor: V) -> Result<ValueNoObj, V::Error>
            where
                V: SeqAccess<'de>,
            {
                let mut vec = Vec::new();

                while let Some(elem) = tri!(visitor.next_element()) {
                    vec.push(elem);
                }

                Ok(ValueNoObj::Array(vec))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            fn visit_map<V>(self, mut visitor: V) -> Result<ValueNoObj, V::Error>
            where
                V: MapAccess<'de>,
            {
                match tri!(visitor.next_key_seed(KeyClassifier)) {
                    #[cfg(feature = "arbitrary_precision")]
                    Some(KeyClass::Number) => {
                        let number: NumberFromString = tri!(visitor.next_value());
                        Ok(ValueNoObj::Number(number.value))
                    }
                    #[cfg(feature = "raw_value")]
                    Some(KeyClass::RawValueNoObj) => {
                        let value = tri!(visitor.next_value_seed(crate::raw::BoxedFromString));
                        crate::from_str(value.get()).map_err(de::Error::custom)
                    }
                    None => Err(serde::de::Error::invalid_type(
                        Unexpected::Map,
                        &"`Object` isn't supported",
                    )),
                }
            }
        }

        deserializer.deserialize_any(ValueNoObjVisitor)
    }
}

impl FromStr for ValueNoObj {
    type Err = Error;
    fn from_str(s: &str) -> Result<ValueNoObj, Error> {
        super::super::de::from_str(s)
    }
}

macro_rules! deserialize_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObj::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObj::Number(n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

fn visit_array<'de, V>(array: Vec<ValueNoObj>, visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqDeserializer::new(array);
    let seq = tri!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

impl<'de> serde::Deserializer<'de> for ValueNoObj {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Null => visitor.visit_unit(),
            ValueNoObj::Bool(v) => visitor.visit_bool(v),
            ValueNoObj::Number(n) => n.deserialize_any(visitor),
            #[cfg(any(feature = "std", feature = "alloc"))]
            ValueNoObj::String(v) => visitor.visit_string(v),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            ValueNoObj::String(_) => unreachable!(),
            ValueNoObj::Array(v) => visit_array(v, visitor),
        }
    }

    deserialize_number!(deserialize_i8);
    deserialize_number!(deserialize_i16);
    deserialize_number!(deserialize_i32);
    deserialize_number!(deserialize_i64);
    deserialize_number!(deserialize_i128);
    deserialize_number!(deserialize_u8);
    deserialize_number!(deserialize_u16);
    deserialize_number!(deserialize_u32);
    deserialize_number!(deserialize_u64);
    deserialize_number!(deserialize_u128);
    deserialize_number!(deserialize_f32);
    deserialize_number!(deserialize_f64);

    #[inline]
    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    #[inline]
    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            ValueNoObj::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumDeserializer { variant, value })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
        {
            if name == crate::raw::TOKEN {
                return visitor.visit_map(crate::raw::OwnedRawDeserializer {
                    raw_value: Some(self.to_string()),
                });
            }
        }

        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            ValueNoObj::String(v) => visitor.visit_string(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_byte_buf(visitor)
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            #[cfg(any(feature = "std", feature = "alloc"))]
            ValueNoObj::String(v) => visitor.visit_string(v),
            ValueNoObj::Array(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Array(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Array(v) => visit_array(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        drop(self);
        visitor.visit_unit()
    }
}

struct EnumDeserializer {
    variant: String,
    value: Option<ValueNoObj>,
}

impl<'de> EnumAccess<'de> for EnumDeserializer {
    type Error = Error;
    type Variant = VariantDeserializer;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, VariantDeserializer), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

impl<'de> IntoDeserializer<'de, Error> for ValueNoObj {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> IntoDeserializer<'de, Error> for &'de ValueNoObj {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct VariantDeserializer {
    value: Option<ValueNoObj>,
}

impl<'de> VariantAccess<'de> for VariantDeserializer {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(ValueNoObj::Array(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visit_array(v, visitor)
                }
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqDeserializer {
    iter: vec::IntoIter<ValueNoObj>,
}

impl SeqDeserializer {
    fn new(vec: Vec<ValueNoObj>) -> Self {
        SeqDeserializer {
            iter: vec.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for SeqDeserializer {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

macro_rules! deserialize_value_ref_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObj::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObj::Number(n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

fn visit_array_ref<'de, V>(array: &'de [ValueNoObj], visitor: V) -> Result<V::Value, Error>
where
    V: Visitor<'de>,
{
    let len = array.len();
    let mut deserializer = SeqRefDeserializer::new(array);
    let seq = tri!(visitor.visit_seq(&mut deserializer));
    let remaining = deserializer.iter.len();
    if remaining == 0 {
        Ok(seq)
    } else {
        Err(serde::de::Error::invalid_length(
            len,
            &"fewer elements in array",
        ))
    }
}

impl<'de> serde::Deserializer<'de> for &'de ValueNoObj {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Null => visitor.visit_unit(),
            ValueNoObj::Bool(v) => visitor.visit_bool(*v),
            ValueNoObj::Number(n) => n.deserialize_any(visitor),
            ValueNoObj::String(v) => visitor.visit_borrowed_str(v),
            ValueNoObj::Array(v) => visit_array_ref(v, visitor),
        }
    }

    deserialize_value_ref_number!(deserialize_i8);
    deserialize_value_ref_number!(deserialize_i16);
    deserialize_value_ref_number!(deserialize_i32);
    deserialize_value_ref_number!(deserialize_i64);
    deserialize_number!(deserialize_i128);
    deserialize_value_ref_number!(deserialize_u8);
    deserialize_value_ref_number!(deserialize_u16);
    deserialize_value_ref_number!(deserialize_u32);
    deserialize_value_ref_number!(deserialize_u64);
    deserialize_number!(deserialize_u128);
    deserialize_value_ref_number!(deserialize_f32);
    deserialize_value_ref_number!(deserialize_f64);

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            ValueNoObj::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_enum<V>(
        self,
        _name: &str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        let (variant, value) = match self {
            ValueNoObj::String(variant) => (variant, None),
            other => {
                return Err(serde::de::Error::invalid_type(
                    other.unexpected(),
                    &"string or map",
                ));
            }
        };

        visitor.visit_enum(EnumRefDeserializer { variant, value })
    }

    #[inline]
    fn deserialize_newtype_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        #[cfg(feature = "raw_value")]
        {
            if name == crate::raw::TOKEN {
                return visitor.visit_map(crate::raw::OwnedRawDeserializer {
                    raw_value: Some(self.to_string()),
                });
            }
        }

        let _ = name;
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            ValueNoObj::Bool(v) => visitor.visit_bool(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::String(v) => visitor.visit_borrowed_str(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::String(v) => visitor.visit_borrowed_str(v),
            ValueNoObj::Array(v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_bytes(visitor)
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match *self {
            ValueNoObj::Null => visitor.visit_unit(),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit_struct<V>(self, _name: &'static str, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_unit(visitor)
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Array(v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_seq(visitor)
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        Err(self.invalid_type(&visitor))
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObj::Array(v) => visit_array_ref(v, visitor),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_str(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        visitor.visit_unit()
    }
}

struct EnumRefDeserializer<'de> {
    variant: &'de str,
    value: Option<&'de ValueNoObj>,
}

impl<'de> EnumAccess<'de> for EnumRefDeserializer<'de> {
    type Error = Error;
    type Variant = VariantRefDeserializer<'de>;

    fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Error>
    where
        V: DeserializeSeed<'de>,
    {
        let variant = self.variant.into_deserializer();
        let visitor = VariantRefDeserializer { value: self.value };
        seed.deserialize(variant).map(|v| (v, visitor))
    }
}

struct VariantRefDeserializer<'de> {
    value: Option<&'de ValueNoObj>,
}

impl<'de> VariantAccess<'de> for VariantRefDeserializer<'de> {
    type Error = Error;

    fn unit_variant(self) -> Result<(), Error> {
        match self.value {
            Some(value) => Deserialize::deserialize(value),
            None => Ok(()),
        }
    }

    fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.value {
            Some(value) => seed.deserialize(value),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"newtype variant",
            )),
        }
    }

    fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(ValueNoObj::Array(v)) => {
                if v.is_empty() {
                    visitor.visit_unit()
                } else {
                    visit_array_ref(v, visitor)
                }
            }
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"tuple variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"tuple variant",
            )),
        }
    }

    fn struct_variant<V>(
        self,
        _fields: &'static [&'static str],
        _visitor: V,
    ) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
            Some(other) => Err(serde::de::Error::invalid_type(
                other.unexpected(),
                &"struct variant",
            )),
            None => Err(serde::de::Error::invalid_type(
                Unexpected::UnitVariant,
                &"struct variant",
            )),
        }
    }
}

struct SeqRefDeserializer<'de> {
    iter: slice::Iter<'de, ValueNoObj>,
}

impl<'de> SeqRefDeserializer<'de> {
    fn new(slice: &'de [ValueNoObj]) -> Self {
        SeqRefDeserializer { iter: slice.iter() }
    }
}

impl<'de> SeqAccess<'de> for SeqRefDeserializer<'de> {
    type Error = Error;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Error>
    where
        T: DeserializeSeed<'de>,
    {
        match self.iter.next() {
            Some(value) => seed.deserialize(value).map(Some),
            None => Ok(None),
        }
    }

    fn size_hint(&self) -> Option<usize> {
        match self.iter.size_hint() {
            (lower, Some(upper)) if lower == upper => Some(upper),
            _ => None,
        }
    }
}

struct KeyClassifier;

enum KeyClass {
    #[cfg(feature = "arbitrary_precision")]
    Number,
    #[cfg(feature = "raw_value")]
    RawValueNoObj,
}

impl<'de> DeserializeSeed<'de> for KeyClassifier {
    type Value = KeyClass;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_str(self)
    }
}

impl<'de> Visitor<'de> for KeyClassifier {
    type Value = KeyClass;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a string key")
    }

    fn visit_str<E>(self, s: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(KeyClass::RawValueNoObj),
            _ => unreachable!(),
        }
    }

    #[cfg(any(feature = "std", feature = "alloc"))]
    fn visit_string<E>(self, s: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match s.as_str() {
            #[cfg(feature = "arbitrary_precision")]
            crate::number::TOKEN => Ok(KeyClass::Number),
            #[cfg(feature = "raw_value")]
            crate::raw::TOKEN => Ok(KeyClass::RawValueNoObj),
            _ => unreachable!(),
        }
    }
}

impl ValueNoObj {
    #[cold]
    fn invalid_type<E>(&self, exp: &dyn Expected) -> E
    where
        E: serde::de::Error,
    {
        serde::de::Error::invalid_type(self.unexpected(), exp)
    }

    #[cold]
    fn unexpected(&self) -> Unexpected {
        match self {
            ValueNoObj::Null => Unexpected::Unit,
            ValueNoObj::Bool(b) => Unexpected::Bool(*b),
            ValueNoObj::Number(n) => n.unexpected(),
            ValueNoObj::String(s) => Unexpected::Str(s),
            ValueNoObj::Array(_) => Unexpected::Seq,
        }
    }
}
