use super::ValueNoObjOrArr;
use alloc::string::String;

fn eq_i64(value: &ValueNoObjOrArr, other: i64) -> bool {
    value.as_i64().map_or(false, |i| i == other)
}

fn eq_u64(value: &ValueNoObjOrArr, other: u64) -> bool {
    value.as_u64().map_or(false, |i| i == other)
}

fn eq_f32(value: &ValueNoObjOrArr, other: f32) -> bool {
    match value {
        ValueNoObjOrArr::Number(n) => n.as_f32().map_or(false, |i| i == other),
        _ => false,
    }
}

fn eq_f64(value: &ValueNoObjOrArr, other: f64) -> bool {
    value.as_f64().map_or(false, |i| i == other)
}

fn eq_bool(value: &ValueNoObjOrArr, other: bool) -> bool {
    value.as_bool().map_or(false, |i| i == other)
}

fn eq_str(value: &ValueNoObjOrArr, other: &str) -> bool {
    value.as_str().map_or(false, |i| i == other)
}

impl PartialEq<str> for ValueNoObjOrArr {
    fn eq(&self, other: &str) -> bool {
        eq_str(self, other)
    }
}

impl PartialEq<&str> for ValueNoObjOrArr {
    fn eq(&self, other: &&str) -> bool {
        eq_str(self, *other)
    }
}

impl PartialEq<ValueNoObjOrArr> for str {
    fn eq(&self, other: &ValueNoObjOrArr) -> bool {
        eq_str(other, self)
    }
}

impl PartialEq<ValueNoObjOrArr> for &str {
    fn eq(&self, other: &ValueNoObjOrArr) -> bool {
        eq_str(other, *self)
    }
}

impl PartialEq<String> for ValueNoObjOrArr {
    fn eq(&self, other: &String) -> bool {
        eq_str(self, other.as_str())
    }
}

impl PartialEq<ValueNoObjOrArr> for String {
    fn eq(&self, other: &ValueNoObjOrArr) -> bool {
        eq_str(other, self.as_str())
    }
}

macro_rules! partialeq_numeric {
    ($($eq:ident [$($ty:ty)*])*) => {
        $($(
            impl PartialEq<$ty> for ValueNoObjOrArr {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(self, *other as _)
                }
            }

            impl PartialEq<ValueNoObjOrArr> for $ty {
                fn eq(&self, other: &ValueNoObjOrArr) -> bool {
                    $eq(other, *self as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a ValueNoObjOrArr {
                fn eq(&self, other: &$ty) -> bool {
                    $eq(*self, *other as _)
                }
            }

            impl<'a> PartialEq<$ty> for &'a mut ValueNoObjOrArr {
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
