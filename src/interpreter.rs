use std::{any::Any, fmt::Debug};

use crate::error::SyntaxError;

pub trait Interpreter {
    fn parse_expr(&self, input: &str) -> Result<(Box<dyn Expr>, usize), SyntaxError>;
    fn parse_pattern(&self, input: &str) -> Result<(Box<dyn Pattern>, usize), SyntaxError>;
}

pub trait Expr: Debug + Any {}

pub trait Pattern: Debug + Any {}

pub trait Context: Any {
    fn eval(&self, expr: &dyn Expr) -> Result<Box<dyn Value>, /* Self::Error */ Box<dyn Any>>;
    fn fork(&self) -> Box<dyn Context>;
    fn bind(&mut self, pattern: &dyn Pattern, value: Box<dyn Value>) -> Result<(), /* Self::Error */ Box<dyn Any>>;
}

pub trait Value: Any {
    fn to_string(&self) -> Result<String, /* Self::Error */ Box<dyn Any>>;
    fn to_bool(&self) -> Result<bool, /* Self::Error */ Box<dyn Any>>;
    fn to_entries(&self) -> Result<Vec<(Box<dyn Value>, Box<dyn Value>)>, /* Self::Error */ Box<dyn Any>>;
}

pub mod default {
    use std::any::Any;

    use pest::Parser;
    use serde_json::{json, Number, Value as JsonValue};

    use crate::error::SyntaxError;
    use super::Value as _;

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

    #[derive(Debug, Clone, Copy)]
    pub enum UnaryOp {
        Not,
        Pos,
        Neg,
    }

    #[derive(Debug, Clone, Copy)]
    pub enum BinaryOp {
        Pow,
        Mul, Div, Mod,
        Add, Sub,
        Shl, Shr, UShr,
        Lt, Le, Gt, Ge,
        Eq, Ne,
        BitAnd,
        BitXor,
        BitOr,
        And,
        Or,
    }

    #[derive(Debug)]
    pub enum Expr {
        Number(Number),
        String(String),
        Ident(String),
        Array(Vec<Expr>),
        Apply(Box<Expr>, Vec<Expr>),
        Unary(UnaryOp, Box<Expr>),
        Binary(Box<Expr>, BinaryOp, Box<Expr>),
    }

    impl super::Expr for Expr {}

    #[derive(Debug)]
    pub enum Pattern {
        Ident(String),
    }

    impl super::Pattern for Pattern {}

    pub struct Context {
        inner: JsonValue,
    }

    impl Context {
        fn _eval(&self, expr: &Expr) -> Result<Value, Box<dyn Any>> {
            let inner = match expr {
                Expr::Number(n) => JsonValue::Number(n.clone()),
                Expr::String(s) => JsonValue::String(s.clone()),
                Expr::Ident(ident) => {
                    if ident == "true" {
                        JsonValue::Bool(true)
                    } else if ident == "false" {
                        JsonValue::Bool(false)
                    } else if ident == "null" {
                        JsonValue::Null
                    } else {
                        self.inner.get(ident).unwrap_or(&JsonValue::Null).clone()
                    }
                },
                // Expr::Array(vec) => {
                //     let vec = vec.iter().map(|expr| {
                //         self._eval(expr).unwrap()
                //     }).collect();
                //     JsonValue::Array(vec)
                // },
                // Expr::Apply(func, args) => {
                //     let func = self._eval(func).unwrap();
                //     let args = args.iter().map(|expr| {
                //         self._eval(expr).unwrap()
                //     }).collect();
                //     JsonValue::Null
                // },
                Expr::Unary(op, expr) => {
                    let value = self._eval(expr).unwrap();
                    match op {
                        UnaryOp::Not => JsonValue::Bool(!value.to_bool()?),
                        UnaryOp::Pos => JsonValue::Number(value.to_number()?),
                        // UnaryOp::Neg => JsonValue::Number(match value.to_number()? {
                        //     Number::PosInt(n) => Number::NegInt(-n),
                        //     Number::NegInt(n) => Number::PosInt(-n),
                        //     Number::Float(n) => Number::Float(-n),
                        // }),
                        _ => unimplemented!(),
                    }
                },
                Expr::Binary(lhs, op, rhs) => {
                    let lhs = self._eval(lhs).unwrap();
                    let rhs = self._eval(rhs).unwrap();
                    match op {
                        _ => unimplemented!(),
                    }
                },
                _ => unimplemented!(),
            };
            Ok(Value{ inner })
        }
    }

    impl super::Context for Context {
        fn eval(&self, expr: &dyn super::Expr) -> Result<Box<dyn super::Value>, Box<dyn Any>> {
            let expr = (expr as &dyn Any).downcast_ref::<Expr>().unwrap();
            Ok(Box::new(self._eval(expr)?))
        }

        fn fork(&self) -> Box<dyn super::Context> {
            Box::new(Context {
                inner: self.inner.clone(),
            })
        }

        fn bind(&mut self, pattern: &dyn super::Pattern, value: Box<dyn super::Value>) -> Result<(), /* Self::Error */ Box<dyn Any>> {
            match (pattern as &dyn Any).downcast_ref::<Pattern>().unwrap() {
                Pattern::Ident(ident) => {
                    self.inner[ident] = (value as Box<dyn Any>).downcast::<Value>().unwrap().inner;
                    Ok(())
                },
            }
        }
    }

    pub struct Value {
        inner: JsonValue,
    }

    impl Value {
        fn to_number(&self) -> Result<Number, Box<dyn Any>> {
            match &self.inner {
                JsonValue::Number(n) => Ok(n.clone()),
                _ => Err(Box::new(())),
            }
        }
    }

    impl super::Value for Value {
        fn to_string(&self) -> Result<String, Box<dyn Any>> {
            match &self.inner {
                JsonValue::Null => Ok("".to_string()),
                // TODO partial object
                _ => Ok(self.inner.to_string()),
            }
        }

        fn to_bool(&self) -> Result<bool, Box<dyn Any>> {
            match &self.inner {
                JsonValue::Null => Ok(false),
                JsonValue::Bool(b) => Ok(*b),
                JsonValue::Number(n) => Ok(n.as_i64().unwrap() != 0),
                JsonValue::String(s) => Ok(!s.is_empty()),
                JsonValue::Array(_) => Ok(true),
                JsonValue::Object(_) => Ok(true),
            }
        }

        fn to_entries(&self) -> Result<Vec<(Box<dyn super::Value>, Box<dyn super::Value>)>, /* Self::Error */ Box<dyn Any>> {
            match &self.inner {
                JsonValue::Array(vec) => Ok(vec.iter().enumerate().map(|(k, v)| {
                    let value = Box::new(Value { inner: v.clone() }) as Box<dyn super::Value>;
                    let key = Box::new(Value { inner: json!(k) }) as Box<dyn super::Value>;
                    (value, key)
                }).collect()),
                JsonValue::Object(map) => Ok(map.iter().map(|(k, v)| {
                    let value = Box::new(Value { inner: v.clone() }) as Box<dyn super::Value>;
                    let key = Box::new(Value { inner: json!(k) }) as Box<dyn super::Value>;
                    (value, key)
                }).collect()),
                _ => Err(Box::new(())),
            }
        }
    }
}
