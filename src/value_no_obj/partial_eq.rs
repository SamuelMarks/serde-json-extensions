use super::ValueNoObj;
use alloc::string::String;

fn eq_i64(value: &ValueNoObj, other: i64) -> bool {
    value.as_i64().map_or(false, |i| i == other)
}

fn eq_u64(value: &ValueNoObj, other: u64) -> bool {
    value.as_u64().map_or(false, |i| i == other)
}

fn eq_f32(value: &ValueNoObj, other: f32) -> bool {
    match value {
        ValueNoObj::Number(n) => n.as_f32().map_or(false, |i| i == other),
        _ => false,
    }
}

fn eq_f64(value: &ValueNoObj, other: f64) -> bool {
    value.as_f64().map_or(false, |i| i == other)
}

fn eq_bool(value: &ValueNoObj, other: bool) -> bool {
    value.as_bool().map_or(false, |i| i == other)
}

fn eq_str(value: &ValueNoObj, other: &str) -> bool {
    value.as_str().map_or(false, |i| i == other)
}

impl PartialEq<str> for ValueNoObj {
    fn eq(&self, other: &str) -> bool {
        eq_str(self, other)
    }
}

impl PartialEq<&str> for ValueNoObj {
    fn eq(&self, other: &&str) -> bool {
        eq_str(self, *other)
    }
}

impl PartialEq<ValueNoObj> for str {
    fn eq(&self, other: &ValueNoObj) -> bool {
        eq_str(other, self)
    }
}

impl PartialEq<ValueNoObj> for &str {
    fn eq(&self, other: &ValueNoObj) -> bool {
        eq_str(other, *self)
    }
}

impl PartialEq<String> for ValueNoObj {
    fn eq(&self, other: &String) -> bool {
        eq_str(self, other.as_str())
    }
}

impl PartialEq<ValueNoObj> for String {
    fn eq(&self, other: &ValueNoObj) -> bool {
        eq_str(other, self.as_str())
    }
}

macro_rules! partialeq_numeric {
    ($($eq:ident [$($ty:ty)*])*) => {
        $($(
            impl PartialEq<$ty> for ValueNoObj {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(self, *other as _)
                }
            }

            impl PartialEq<ValueNoObj> for $ty {
                fn eq(&self, other: &ValueNoObj) -> bool {
                    $eq(other, *self as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a ValueNoObj {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a mut ValueNoObj {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }
        )*)*
    }
}

partialeq_numeric! {
    eq_i64[i8 i16 i32 i64 isize]
    eq_u64[u8 u16 u32 u64 usize]
    eq_f32[f32]
    eq_f64[f64]
    eq_bool[bool]
}
