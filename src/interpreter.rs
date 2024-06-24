use std::{any::Any, fmt::Debug};

use crate::error::SyntaxError;

pub trait Interpreter {
    fn parse_expr(&self, input: &str) -> Result<(Box<dyn Expr>, usize), SyntaxError>;
    fn parse_pattern(&self, input: &str) -> Result<(Box<dyn Pattern>, usize), SyntaxError>;
}

pub trait Expr: Debug {}

pub trait Pattern: Debug {}

pub trait Context {
    fn eval(&self, expr: &dyn Expr) -> Result<Box<dyn Value>, /* Self::Error */ Box<dyn Any>>;
    fn fork(&self) -> Box<dyn Context>;
    fn bind(&mut self, pattern: &dyn Pattern, value: Box<dyn Value>) -> Result<(), /* Self::Error */ Box<dyn Any>>;
}

pub trait Value {
    fn to_string(&self) -> Result<String, /* Self::Error */ Box<dyn Any>>;
    fn to_bool(&self) -> Result<bool, /* Self::Error */ Box<dyn Any>>;
    fn to_entries(&self) -> Result<Vec<(Box<dyn Value>, Box<dyn Value>)>, /* Self::Error */ Box<dyn Any>>;
}

pub mod default {
    use std::any::Any;

    use pest::Parser;
    use serde_json::{json, Value as JsonValue};

    use crate::error::SyntaxError;

    #[derive(Parser)]
    #[grammar = "default.pest"]
    struct DefaultParser;

    pub struct Interpreter;

    impl super::Interpreter for Interpreter {
        fn parse_expr(&self, input: &str) -> Result<(Box<dyn super::Expr>, usize), SyntaxError> {
            match DefaultParser::parse(Rule::expr, input) {
                Ok(pairs) => Ok((Box::new(Expr {}), pairs.as_str().len())),
                Err(e) => Err(SyntaxError {
                    message: e.to_string(),
                    range: (0, 0), // TODO
                }),
            }
        }

        fn parse_pattern(&self, input: &str) -> Result<(Box<dyn super::Pattern>, usize), SyntaxError> {
            match DefaultParser::parse(Rule::ident, input) {
                Ok(pairs) => Ok((Box::new(Pattern {}), pairs.as_str().len())),
                Err(e) => return Err(SyntaxError {
                    message: e.to_string(),
                    range: (0, 0), // TODO
                }),
            }
        }
    }

    #[derive(Debug)]
    pub struct Expr {}

    impl super::Expr for Expr {}

    #[derive(Debug)]
    pub struct Pattern {}

    impl super::Pattern for Pattern {}

    pub struct Context {
        value: JsonValue,
    }

    impl super::Context for Context {
        fn eval(&self, _expr: &dyn super::Expr) -> Result<Box<dyn super::Value>, Box<dyn Any>> {
            Ok(Box::new(Value {
                value: JsonValue::Null,
            }))
        }

        fn fork(&self) -> Box<dyn super::Context> {
            Box::new(Context {
                value: self.value.clone(),
            })
        }

        fn bind(&mut self, _pattern: &dyn super::Pattern, _value: Box<dyn super::Value>) -> Result<(), /* Self::Error */ Box<dyn Any>> {
            todo!()
        }
    }

    pub struct Value {
        value: JsonValue,
    }

    impl super::Value for Value {
        fn to_string(&self) -> Result<String, Box<dyn Any>> {
            Ok(self.value.to_string())
        }

        fn to_bool(&self) -> Result<bool, Box<dyn Any>> {
            match &self.value {
                JsonValue::Null => Ok(false),
                JsonValue::Bool(b) => Ok(*b),
                JsonValue::Number(n) => Ok(n.as_i64().unwrap() != 0),
                JsonValue::String(s) => Ok(!s.is_empty()),
                JsonValue::Array(_) => Ok(true),
                JsonValue::Object(_) => Ok(true),
            }
        }

        fn to_entries(&self) -> Result<Vec<(Box<dyn super::Value>, Box<dyn super::Value>)>, /* Self::Error */ Box<dyn Any>> {
            match &self.value {
                JsonValue::Array(vec) => Ok(vec.iter().enumerate().map(|(k, v)| {
                    let value = Box::new(Value { value: v.clone() }) as Box<dyn super::Value>;
                    let key = Box::new(Value { value: json!(k) }) as Box<dyn super::Value>;
                    (value, key)
                }).collect()),
                JsonValue::Object(map) => Ok(map.iter().map(|(k, v)| {
                    let value = Box::new(Value { value: v.clone() }) as Box<dyn super::Value>;
                    let key = Box::new(Value { value: json!(k) }) as Box<dyn super::Value>;
                    (value, key)
                }).collect()),
                _ => Err(Box::new(())),
            }
        }
    }
}
