use super::prelude::*;

pub enum Value {
    String(String),
    Int(isize),
    UInt(usize),
    Bool(bool),
    Float(f64)
}

macro_rules! value_impl {
    ($name: ident => $type: ty) => {
        value_impl!(try_from_type $name => $type);
        value_impl!(from_value $name => $type);
    };
    (try_from_type $name: ident => $type: ty) => {
        impl TryFrom<&Value> for $type {
            type Error = &'static str;
            fn try_from(value: &Value) -> Result<Self, Self::Error> {
                match value {
                    Value::$name(s) => Ok(s.clone()),
                    _ => Err("is not a string")
                }
            }
        }
    };
    (from_value $name: ident => $type: ty) => {
        impl From<$type> for Value {
            fn from(s: $type) -> Self {
                Self::$name(s)
            }
        }
    }

}

value_impl!(String => String);
value_impl!(Int => isize);
value_impl!(UInt => usize);
value_impl!(Bool => bool);
value_impl!(Float => f64);