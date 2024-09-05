use super::ValueNoObj;
use crate::map::Map;
use crate::number::Number;
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for ValueNoObj {
                fn from(n: $ty) -> Self {
                    ValueNoObj::Number(n.into())
                }
            }
        )*
    };
}

from_integer! {
    i8 i16 i32 i64 isize
    u8 u16 u32 u64 usize
}

#[cfg(feature = "arbitrary_precision")]
from_integer! {
    i128 u128
}

impl From<f32> for ValueNoObj {
    /// Convert 32-bit floating point number to `Value::Number`, or
    /// `Value::Null` if infinite or NaN.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let f: f32 = 13.37;
    /// let x: Value = f.into();
    /// ```
    fn from(f: f32) -> Self {
        Number::from_f32(f).map_or(ValueNoObj::Null, ValueNoObj::Number)
    }
}

impl From<f64> for ValueNoObj {
    /// Convert 64-bit floating point number to `Value::Number`, or
    /// `Value::Null` if infinite or NaN.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let f: f64 = 13.37;
    /// let x: Value = f.into();
    /// ```
    fn from(f: f64) -> Self {
        Number::from_f64(f).map_or(ValueNoObj::Null, ValueNoObj::Number)
    }
}

impl From<bool> for ValueNoObj {
    /// Convert boolean to `Value::Bool`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let b = false;
    /// let x: Value = b.into();
    /// ```
    fn from(f: bool) -> Self {
        ValueNoObj::Bool(f)
    }
}

impl From<String> for ValueNoObj {
    /// Convert `String` to `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let s: String = "lorem".to_string();
    /// let x: Value = s.into();
    /// ```
    fn from(f: String) -> Self {
        ValueNoObj::String(f)
    }
}

impl From<&str> for ValueNoObj {
    /// Convert string slice to `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let s: &str = "lorem";
    /// let x: Value = s.into();
    /// ```
    fn from(f: &str) -> Self {
        ValueNoObj::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for ValueNoObj {
    /// Convert copy-on-write string to `Value::String`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Borrowed("lorem");
    /// let x: Value = s.into();
    /// ```
    ///
    /// ```
    /// use serde_json::Value;
    /// use std::borrow::Cow;
    ///
    /// let s: Cow<str> = Cow::Owned("lorem".to_string());
    /// let x: Value = s.into();
    /// ```
    fn from(f: Cow<'a, str>) -> Self {
        ValueNoObj::String(f.into_owned())
    }
}

impl From<Number> for ValueNoObj {
    /// Convert `Number` to `Value::Number`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::{Number, Value};
    ///
    /// let n = Number::from(7);
    /// let x: Value = n.into();
    /// ```
    fn from(f: Number) -> Self {
        ValueNoObj::Number(f)
    }
}

impl From<Map<String, ValueNoObj>> for ValueNoObj {
    /// Convert map (with string keys) to `Value::Object`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::{Map, Value};
    ///
    /// let mut m = Map::new();
    /// m.insert("Lorem".to_string(), "ipsum".into());
    /// let x: Value = m.into();
    /// ```
    fn from(_f: Map<String, ValueNoObj>) -> Self {
        unimplemented!()
    }
}

impl<T: Into<ValueNoObj>> From<Vec<T>> for ValueNoObj {
    /// Convert a `Vec` to `Value::Array`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// ```
    fn from(f: Vec<T>) -> Self {
        ValueNoObj::Array(f.into_iter().map(Into::into).collect())
    }
}

impl<T: Clone + Into<ValueNoObj>> From<&[T]> for ValueNoObj {
    /// Convert a slice to `Value::Array`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v: &[&str] = &["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into();
    /// ```
    fn from(f: &[T]) -> Self {
        ValueNoObj::Array(f.iter().cloned().map(Into::into).collect())
    }
}

impl<T: Into<ValueNoObj>> FromIterator<T> for ValueNoObj {
    /// Create a `Value::Array` by collecting an iterator of array elements.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v = std::iter::repeat(42).take(5);
    /// let x: Value = v.collect();
    /// ```
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v: Vec<_> = vec!["lorem", "ipsum", "dolor"];
    /// let x: Value = v.into_iter().collect();
    /// ```
    ///
    /// ```
    /// use std::iter::FromIterator;
    /// use serde_json::Value;
    ///
    /// let x: Value = Value::from_iter(vec!["lorem", "ipsum", "dolor"]);
    /// ```
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        ValueNoObj::Array(iter.into_iter().map(Into::into).collect())
    }
}

impl<K: Into<String>, V: Into<ValueNoObj>> FromIterator<(K, V)> for ValueNoObj {
    /// Create a `Value::Object` by collecting an iterator of key-value pairs.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let v: Vec<_> = vec![("lorem", 40), ("ipsum", 2)];
    /// let x: Value = v.into_iter().collect();
    /// ```
    fn from_iter<I: IntoIterator<Item = (K, V)>>(_iter: I) -> Self {
        unimplemented!()
    }
}

impl From<()> for ValueNoObj {
    /// Convert `()` to `Value::Null`.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::Value;
    ///
    /// let u = ();
    /// let x: Value = u.into();
    /// ```
    fn from((): ()) -> Self {
        ValueNoObj::Null
    }
}

impl<T> From<Option<T>> for ValueNoObj
where
    T: Into<ValueNoObj>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => ValueNoObj::Null,
            Some(value) => Into::into(value),
        }
    }
}
