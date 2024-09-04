use core::fmt;
use std::fmt::Debug;

/// Taken from `serde::Value` but excludes `Object(Map<String, Value>),` and `Array(Vec<ScalarOrArrayValue>),`
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ScalarValue {
    /// Represents a JSON null value.
    ///
    /// ```json
    /// null
    /// ```
    Null,

    /// Represents a JSON boolean.
    ///
    /// ```json
    /// true
    /// ```
    /// ```json
    /// false
    /// ```
    Bool(bool),

    /// Represents a JSON number, whether integer or floating point.
    ///
    /// ```json
    /// 5
    /// ```
    /// ```json
    /// 5.12
    /// ```
    Number(serde_json::Number),

    /// Represents a JSON string.
    ///
    /// ```json
    /// "a string"
    /// ```
    String(String),
}

impl Debug for ScalarValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScalarValue::Null => formatter.write_str("Null"),
            ScalarValue::Bool(boolean) => write!(formatter, "Bool({})", boolean),
            ScalarValue::Number(number) => Debug::fmt(number, formatter),
            ScalarValue::String(string) => write!(formatter, "String({:?})", string),
        }
    }
}

impl serde::Serialize for ScalarValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            ScalarValue::Null => serializer.serialize_none(),
            ScalarValue::Bool(b) => serializer.serialize_bool(*b),
            ScalarValue::Number(n) => serializer.serialize_i32(n.into()),
            ScalarValue::String(s) => serializer.serialize_str(s),
        }
    }
}

#[path = "./de.rs"]
pub mod de;
