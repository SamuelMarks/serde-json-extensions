use super::ValueNoObjOrArr;
use crate::map::Map;
use crate::number::Number;
use alloc::borrow::Cow;
use alloc::string::{String, ToString};
use alloc::vec::Vec;

macro_rules! from_integer {
    ($($ty:ident)*) => {
        $(
            impl From<$ty> for ValueNoObjOrArr {
                fn from(n: $ty) -> Self {
                    ValueNoObjOrArr::Number(n.into())
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

impl From<f32> for ValueNoObjOrArr {
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
        Number::from_f32(f).map_or(ValueNoObjOrArr::Null, ValueNoObjOrArr::Number)
    }
}

impl From<f64> for ValueNoObjOrArr {
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
        Number::from_f64(f).map_or(ValueNoObjOrArr::Null, ValueNoObjOrArr::Number)
    }
}

impl From<bool> for ValueNoObjOrArr {
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
        ValueNoObjOrArr::Bool(f)
    }
}

impl From<String> for ValueNoObjOrArr {
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
        ValueNoObjOrArr::String(f)
    }
}

impl From<&str> for ValueNoObjOrArr {
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
        ValueNoObjOrArr::String(f.to_string())
    }
}

impl<'a> From<Cow<'a, str>> for ValueNoObjOrArr {
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
        ValueNoObjOrArr::String(f.into_owned())
    }
}

impl From<Number> for ValueNoObjOrArr {
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
        ValueNoObjOrArr::Number(f)
    }
}

impl From<Map<String, ValueNoObjOrArr>> for ValueNoObjOrArr {
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
    fn from(_: Map<String, ValueNoObjOrArr>) -> Self {
        unimplemented!()
    }
}

impl<T: Into<ValueNoObjOrArr>> From<Vec<T>> for ValueNoObjOrArr {
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
    fn from(_: Vec<T>) -> Self {
        unimplemented!()
    }
}

impl<T: Clone + Into<ValueNoObjOrArr>> From<&[T]> for ValueNoObjOrArr {
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
    fn from(_: &[T]) -> Self {
        unimplemented!()
    }
}

impl<T: Into<ValueNoObjOrArr>> FromIterator<T> for ValueNoObjOrArr {
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
    fn from_iter<I: IntoIterator<Item = T>>(_: I) -> Self {
        unimplemented!()
    }
}

impl<K: Into<String>, V: Into<ValueNoObjOrArr>> FromIterator<(K, V)> for ValueNoObjOrArr {
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
    fn from_iter<I: IntoIterator<Item = (K, V)>>(_: I) -> Self {
        unimplemented!()
    }
}

impl From<()> for ValueNoObjOrArr {
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
        ValueNoObjOrArr::Null
    }
}

impl<T> From<Option<T>> for ValueNoObjOrArr
where
    T: Into<ValueNoObjOrArr>,
{
    fn from(opt: Option<T>) -> Self {
        match opt {
            None => ValueNoObjOrArr::Null,
            Some(value) => Into::into(value),
        }
    }
}
