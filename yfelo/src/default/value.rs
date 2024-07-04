use std::collections::BTreeMap;
use std::rc::Rc;
use std::fmt;

use yfelo_core::{factory, Definition};

use super::{Expr, Pattern, RuntimeError};

#[derive(Debug, Clone)]
pub enum Value {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Array(Vec<Rc<Value>>),
    Object(BTreeMap<String, Rc<Value>>),
    Lazy(Vec<(Pattern, Option<Expr>)>, Definition),
    Ref(Rc<Value>),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match &self {
            Self::Null => "null",
            Self::Bool(_) => "bool",
            Self::Number(_) => "number",
            Self::String(_) => "string",
            Self::Array(_) => "array",
            Self::Object(_) => "object",
            Self::Lazy(_, _) => "function",
            Self::Ref(v) => v.type_name(),
        }
    }

    pub fn as_number(&self) -> Result<f64, RuntimeError> {
        match &self {
            Self::Number(n) => Ok(*n),
            Self::Bool(b) => Ok(if *b { 1. } else { 0. }),
            Self::Ref(v) => v.as_number(),
            _ => Err(RuntimeError {
                message: format!("expect number or bool, found {}", self.type_name()),
            }),
        }
    }

    pub fn as_str(&self) -> Result<&str, RuntimeError> {
        match &self {
            Self::String(s) => Ok(s),
            Self::Ref(v) => v.as_str(),
            _ => Err(RuntimeError {
                message: format!("expect string, found {}", self.type_name()),
            }),
        }
    }

    pub fn as_bool(&self) -> Result<bool, RuntimeError> {
        match &self {
            Self::Null => Ok(false),
            Self::Bool(b) => Ok(*b),
            Self::Number(n) => Ok(*n != 0.),
            Self::String(s) => Ok(!s.is_empty()),
            Self::Ref(v) => v.as_bool(),
            _ => Ok(true),
        }
    }

    pub fn get(&self, key: &Self) -> Result<Self, RuntimeError> {
        match self {
            Self::Ref(v) => v.get(key),
            Self::Object(map) => {
                match map.get(key.as_str()?) {
                    Some(rc) => Ok(Self::Ref(rc.clone())),
                    None => Ok(Self::Null),
                }
            },
            Self::Array(vec) => {
                match vec.get(key.as_number()? as usize) {
                    Some(rc) => Ok(Self::Ref(rc.clone())),
                    None => Ok(Self::Null),
                }
            },
            _ => Err(RuntimeError {
                message: format!("cannot index into {}", self.type_name()),
            }),
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
        Self::Bool(value)
    }
}

// we cannot use T: Into<String> here because of conflicts
impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl From<&str> for Value {
    fn from(value: &str) -> Self {
        Self::String(value.into())
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Null, Self::Null) => true,
            (Self::Bool(a), Self::Bool(b)) => a == b,
            (Self::Number(a), Self::Number(b)) => a == b,
            (Self::String(a), Self::String(b)) => a == b,
            (Self::Ref(a), b) => a.as_ref() == b,
            (a, Self::Ref(b)) => a == b.as_ref(),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Null => write!(f, "null"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Number(n) => write!(f, "{}", n),
            Self::String(s) => write!(f, "{}", s),
            Self::Array(vec) => {
                write!(f, "[")?;
                for (i, value) in vec.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", value)?;
                }
                write!(f, "]")
            },
            Self::Object(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            },
            Self::Lazy(_, _) => {
                write!(f, "fn")
            },
            Self::Ref(v) => {
                write!(f, "{}", v)
            },
        }
    }
}

impl factory::Value<RuntimeError> for Value {
    fn to_string(&self) -> Result<String, RuntimeError> {
        match &self {
            Self::Null => Ok("".to_string()),
            // TODO partial object
            _ => Ok(format!("{}", self)),
        }
    }

    fn as_bool(&self) -> Result<bool, RuntimeError> {
        Self::as_bool(self)
    }

    fn as_entries(&self) -> Result<Vec<(Self, Self)>, RuntimeError> {
        match &self {
            Self::Array(vec) => Ok(vec.iter().enumerate().map(|(k, v)| {
                let k = Self::Number(k as f64);
                (Self::Ref(v.clone()), k)
            }).collect()),
            Self::Object(map) => Ok(map.iter().map(|(k, v)| {
                let k = Self::String(k.clone());
                (Self::Ref(v.clone()), k)
            }).collect()),
            _ => Err(RuntimeError {
                message: format!("expect array or object, found {}", self.type_name()),
            }),
        }
    }
}
