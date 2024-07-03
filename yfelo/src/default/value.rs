use std::collections::BTreeMap;
use std::{fmt, ops};

use yfelo_core::{factory, Definition};

use super::{Expr, Pattern, RuntimeError};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Value>),
    Object(BTreeMap<String, Value>),
    Abs(Vec<(Pattern, Option<Expr>)>, Definition),
}

impl Value {
    pub fn as_number(&self) -> Result<f64, RuntimeError> {
        match &self {
            Self::Number(n) => Ok(*n),
            Self::Bool(b) => Ok(if *b { 1. } else { 0. }),
            _ => Err(RuntimeError {}),
        }
    }

    pub fn as_str(&self) -> Result<&str, RuntimeError> {
        match &self {
            Self::String(s) => Ok(s),
            _ => Err(RuntimeError {}),
        }
    }

    pub fn as_bool(&self) -> Result<bool, RuntimeError> {
        match &self {
            Self::Null => Ok(false),
            Self::Bool(b) => Ok(*b),
            Self::Number(n) => Ok(*n != 0.),
            Self::String(s) => Ok(!s.is_empty()),
            _ => Ok(true),
        }
    }
}

macro_rules! impl_from_number {
    ($ty:ty) => {
        impl From<$ty> for Value {
            fn from(value: $ty) -> Self {
                Value::Number(value as f64)
            }
        }
    };
}

impl_from_number!(u8);
impl_from_number!(u16);
impl_from_number!(u32);
impl_from_number!(u64);
impl_from_number!(u128);
impl_from_number!(usize);
impl_from_number!(i8);
impl_from_number!(i16);
impl_from_number!(i32);
impl_from_number!(i64);
impl_from_number!(i128);
impl_from_number!(isize);
impl_from_number!(f32);
impl_from_number!(f64);

impl From<bool> for Value {
    fn from(value: bool) -> Self {
        Value::Bool(value)
    }
}

// we cannot use T: Into<String> here because of conflicts
impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Value::String(value.into())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Value::Null, Value::Null) => true,
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Number(a), Value::Number(b)) => a == b,
            (Value::String(a), Value::String(b)) => a == b,
            _ => false,
        }
    }
}

impl ops::Index<&str> for Value {
    type Output = Value;

    fn index(&self, index: &str) -> &Self::Output {
        match self {
            Value::Object(map) => map
                .get(index)
                .unwrap_or(&Value::Null),
            _ => panic!("cannot index into {}", self),
        }
    }
}

impl ops::IndexMut<&str> for Value {
    fn index_mut(&mut self, index: &str) -> &mut Self::Output {
        match self {
            Value::Object(map) => map
                .entry(index.to_string())
                .or_insert(Value::Null),
            _ => panic!("cannot index into {}", self),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "{}", s),
            Value::Array(vec) => {
                write!(f, "[")?;
                for (i, value) in vec.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            },
            Value::Object(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            },
            Value::Abs(_, _) => {
                write!(f, "fn")
            },
        }
    }
}

impl factory::Value<RuntimeError> for Value {
    fn to_string(&self) -> Result<String, RuntimeError> {
        match &self {
            Value::Null => Ok("".to_string()),
            // TODO partial object
            _ => Ok(format!("{}", self)),
        }
    }

    fn as_bool(&self) -> Result<bool, RuntimeError> {
        Value::as_bool(self)
    }

    fn as_entries(&self) -> Result<Vec<(Self, Self)>, RuntimeError> {
        match &self {
            Value::Array(vec) => Ok(vec.iter().enumerate().map(|(k, v)| {
                let value = v.clone();
                let key = Value::Number(k as f64);
                (value, key)
            }).collect()),
            Value::Object(map) => Ok(map.iter().map(|(k, v)| {
                let value = v.clone();
                let key = Value::String(k.clone());
                (value, key)
            }).collect()),
            _ => Err(RuntimeError {}),
        }
    }
}
