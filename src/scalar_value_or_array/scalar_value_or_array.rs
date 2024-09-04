use core::fmt;
use std::fmt::Debug;

use crate::tri;

/// Taken from `serde::Value` but excludes `Object(Map<String, Value>),`
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum ScalarOrArrayValue {
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

    /// Represents a JSON array excluding internal objects.
    ///
    /// ```json
    /// ["an", "array", 5, 5.12, [5, 6], null, true]
    /// ```
    Array(Vec<ScalarOrArrayValue>),
}

impl Debug for ScalarOrArrayValue {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ScalarOrArrayValue::Null => formatter.write_str("Null"),
            ScalarOrArrayValue::Bool(boolean) => write!(formatter, "Bool({})", boolean),
            ScalarOrArrayValue::Number(number) => Debug::fmt(number, formatter),
            ScalarOrArrayValue::String(string) => write!(formatter, "String({:?})", string),
            ScalarOrArrayValue::Array(vec) => {
                tri!(formatter.write_str("Array "));
                Debug::fmt(vec, formatter)
            }
        }
    }
}
