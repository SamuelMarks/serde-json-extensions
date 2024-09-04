use core::fmt;

use serde::de::{MapAccess, SeqAccess, Visitor};
use serde::{de, Deserialize};
use serde_json::{Map, Number};

use crate::scalar_value::ScalarValue;
use crate::tri;

impl<'de> Deserialize<'de> for ScalarValue {
    #[inline]
    fn deserialize<D>(deserializer: D) -> Result<ScalarValue, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ScalarValue;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any valid JSON value")
            }

            #[inline]
            fn visit_bool<E>(self, value: bool) -> Result<ScalarValue, E> {
                Ok(ScalarValue::Bool(value))
            }

            #[inline]
            fn visit_i64<E>(self, value: i64) -> Result<ScalarValue, E> {
                Ok(ScalarValue::Number(value.into()))
            }

            #[inline]
            fn visit_u64<E>(self, value: u64) -> Result<ScalarValue, E> {
                Ok(ScalarValue::Number(value.into()))
            }

            #[inline]
            fn visit_f64<E>(self, value: f64) -> Result<ScalarValue, E> {
                Ok(Number::from_f64(value).map_or(ScalarValue::Null, ScalarValue::Number))
            }

            #[inline]
            fn visit_str<E>(self, value: &str) -> Result<ScalarValue, E>
            where
                E: serde::de::Error,
            {
                self.visit_string(String::from(value))
            }

            #[cfg(any(feature = "std", feature = "alloc"))]
            #[inline]
            fn visit_string<E>(self, value: String) -> Result<ScalarValue, E> {
                Ok(ScalarValue::String(value))
            }

            #[inline]
            fn visit_none<E>(self) -> Result<ScalarValue, E> {
                Ok(ScalarValue::Null)
            }

            #[inline]
            fn visit_some<D>(self, deserializer: D) -> Result<ScalarValue, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                Deserialize::deserialize(deserializer)
            }

            #[inline]
            fn visit_unit<E>(self) -> Result<ScalarValue, E> {
                Ok(ScalarValue::Null)
            }

            fn visit_map<V>(self, mut visitor: V) -> Result<ScalarValue, V::Error>
            where
                V: MapAccess<'de>,
            {
                match tri!(visitor.next_key_seed(KeyClassifier)) {
                    #[cfg(feature = "arbitrary_precision")]
                    Some(KeyClass::Number) => {
                        let number: NumberFromString = tri!(visitor.next_value());
                        Ok(ScalarValue::Number(number.value))
                    }
                    #[cfg(feature = "raw_value")]
                    Some(KeyClass::RawValue) => {
                        let value = tri!(visitor.next_value_seed(crate::raw::BoxedFromString));
                        crate::from_str(value.get()).map_err(de::Error::custom)
                    }
                    Some(KeyClass::Map(first_key)) => {
                        let mut values = Map::new();

                        values.insert(first_key, tri!(visitor.next_value()));
                        while let Some((key, value)) = tri!(visitor.next_entry()) {
                            values.insert(key, value);
                        }

                        Ok(ScalarValue::Object(values))
                    }
                    None => Ok(ScalarValue::Object(Map::new())),
                }
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}
