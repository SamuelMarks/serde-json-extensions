use alloc::borrow::ToOwned;
use alloc::string::String;
#[cfg(feature = "raw_value")]
use alloc::string::ToString;
use core::fmt;
use core::str::FromStr;

use serde::de::{
    self, Deserialize, DeserializeSeed, EnumAccess, Expected, IntoDeserializer, MapAccess,
    Unexpected, VariantAccess, Visitor,
};

#[cfg(feature = "arbitrary_precision")]
use crate::number::NumberFromString;

use crate::error::Error;
use crate::number::Number;
use crate::value_no_obj_or_arr::ValueNoObjOrArr;

impl<'de> Deserialize<'de> for ValueNoObjOrArr {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<ValueNoObjOrArr, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ValueNoObjOrArr;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<ValueNoObjOrArr, E> {
                Ok(ValueNoObjOrArr::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<ValueNoObjOrArr, E> {
                Ok(ValueNoObjOrArr::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<ValueNoObjOrArr, E> {
                Ok(ValueNoObjOrArr::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<ValueNoObjOrArr, E> {
                Ok(Number::from_f64(value).map_or(ValueNoObjOrArr::Null, ValueNoObjOrArr::Number))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<ValueNoObjOrArr, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_string<E>(self, value: String) -> Result<ValueNoObjOrArr, E> {
                Ok(ValueNoObjOrArr::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<ValueNoObjOrArr, E> {
                Ok(ValueNoObjOrArr::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<ValueNoObjOrArr, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<ValueNoObjOrArr, E> {
                Ok(ValueNoObjOrArr::Null)
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            fn visit_map<V>(self, mut visitor: V) -> Result<ValueNoObjOrArr, V::Error>
            where
                V: MapAccess<'de>,
            {
                match tri!(visitor.next_key_seed(KeyClassifier)) {
                    #[cfg(feature = "arbitrary_precision")]
                    Some(KeyClass::Number) => {
                        let number: NumberFromString = tri!(visitor.next_value());
                        Ok(ValueNoObjOrArr::Number(number.value))
                    }
                    #[cfg(feature = "raw_value")]
                    Some(KeyClass::RawValue) => {
                        let value = tri!(visitor.next_value_seed(crate::raw::BoxedFromString));
                        crate::from_str(value.get()).map_err(de::Error::custom)
                    }
                    Some(KeyClass::Map(_first_key)) => {
                        Err(serde::de::Error::invalid_type(Unexpected::Map, &"non map"))
                    }
                    None => Err(serde::de::Error::invalid_type(
                        Unexpected::Other(""),
                        &"must provide non-array | non-object",
                    )),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl FromStr for ValueNoObjOrArr {
    type Err = Error;
    fn from_str(s: &str) -> Result<ValueNoObjOrArr, Error> {
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
                ValueNoObjOrArr::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObjOrArr::Number(n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for ValueNoObjOrArr {
    type Error = Error;

    #[inline]
    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObjOrArr::Null => visitor.visit_unit(),
            ValueNoObjOrArr::Bool(v) => visitor.visit_bool(v),
            ValueNoObjOrArr::Number(n) => n.deserialize_any(visitor),
            #[cfg(any(feature = "std", feature = "alloc"))]
            ValueNoObjOrArr::String(v) => visitor.visit_string(v),
            #[cfg(not(any(feature = "std", feature = "alloc")))]
            ValueNoObjOrArr::String(_) => unreachable!(),
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
            ValueNoObjOrArr::Null => visitor.visit_none(),
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
            ValueNoObjOrArr::String(variant) => (variant, None),
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
            ValueNoObjOrArr::Bool(v) => visitor.visit_bool(v),
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
            ValueNoObjOrArr::String(v) => visitor.visit_string(v),
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
            ValueNoObjOrArr::String(v) => visitor.visit_string(v),
            _ => Err(self.invalid_type(&visitor)),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObjOrArr::Null => visitor.visit_unit(),
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
        Err(self.invalid_type(&visitor))
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
        Err(self.invalid_type(&visitor))
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
    value: Option<ValueNoObjOrArr>,
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

impl<'de> IntoDeserializer<'de, Error> for ValueNoObjOrArr {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

impl<'de> IntoDeserializer<'de, Error> for &'de ValueNoObjOrArr {
    type Deserializer = Self;

    fn into_deserializer(self) -> Self::Deserializer {
        self
    }
}

struct VariantDeserializer {
    value: Option<ValueNoObjOrArr>,
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

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
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

macro_rules! deserialize_value_ref_number {
    ($method:ident) => {
        #[cfg(not(feature = "arbitrary_precision"))]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObjOrArr::Number(n) => n.deserialize_any(visitor),
                _ => Err(self.invalid_type(&visitor)),
            }
        }

        #[cfg(feature = "arbitrary_precision")]
        fn $method<V>(self, visitor: V) -> Result<V::Value, Error>
        where
            V: Visitor<'de>,
        {
            match self {
                ValueNoObjOrArr::Number(n) => n.$method(visitor),
                _ => self.deserialize_any(visitor),
            }
        }
    };
}

impl<'de> serde::Deserializer<'de> for &'de ValueNoObjOrArr {
    type Error = Error;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self {
            ValueNoObjOrArr::Null => visitor.visit_unit(),
            ValueNoObjOrArr::Bool(v) => visitor.visit_bool(*v),
            ValueNoObjOrArr::Number(n) => n.deserialize_any(visitor),
            ValueNoObjOrArr::String(v) => visitor.visit_borrowed_str(v),
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
            ValueNoObjOrArr::Null => visitor.visit_none(),
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
            ValueNoObjOrArr::String(variant) => (variant, None),
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
            ValueNoObjOrArr::Bool(v) => visitor.visit_bool(v),
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
            ValueNoObjOrArr::String(v) => visitor.visit_borrowed_str(v),
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
            ValueNoObjOrArr::String(v) => visitor.visit_borrowed_str(v),
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
            ValueNoObjOrArr::Null => visitor.visit_unit(),
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
        Err(self.invalid_type(&visitor))
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
        Err(self.invalid_type(&visitor))
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
    value: Option<&'de ValueNoObjOrArr>,
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
    value: Option<&'de ValueNoObjOrArr>,
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

    fn tuple_variant<V>(self, _len: usize, _visitor: V) -> Result<V::Value, Error>
    where
        V: Visitor<'de>,
    {
        match self.value {
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

struct KeyClassifier;

enum KeyClass {
    Map(String),
    #[cfg(feature = "arbitrary_precision")]
    Number,
    #[cfg(feature = "raw_value")]
    RawValue,
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
            crate::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s.to_owned())),
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
            crate::raw::TOKEN => Ok(KeyClass::RawValue),
            _ => Ok(KeyClass::Map(s)),
        }
    }
}

impl ValueNoObjOrArr {
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
            ValueNoObjOrArr::Null => Unexpected::Unit,
            ValueNoObjOrArr::Bool(b) => Unexpected::Bool(*b),
            ValueNoObjOrArr::Number(n) => n.unexpected(),
            ValueNoObjOrArr::String(s) => Unexpected::Str(s),
        }
    }
}
