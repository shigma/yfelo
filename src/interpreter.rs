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
    use std::collections::BTreeMap;
    use std::{fmt, ops};

    use pest::Parser;

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
        Shl, Shr,
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
        Number(f64),
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
        inner: Value,
    }

    impl Context {
        fn _eval(&self, expr: &Expr) -> Result<Value, Box<dyn Any>> {
            Ok(match expr {
                Expr::Number(n) => Value::Number(n.clone()),
                Expr::String(s) => Value::String(s.clone()),
                Expr::Ident(ident) => {
                    if ident == "true" {
                        Value::Bool(true)
                    } else if ident == "false" {
                        Value::Bool(false)
                    } else if ident == "null" {
                        Value::Null
                    } else {
                        self.inner[ident].clone()
                    }
                },
                Expr::Array(vec) => {
                    Value::Array(vec.iter().map(|expr| {
                        self._eval(expr).unwrap()
                    }).collect())
                },
                // Expr::Apply(func, args) => {
                //     let func = self._eval(func).unwrap();
                //     let args = args.iter().map(|expr| {
                //         self._eval(expr).unwrap()
                //     }).collect();
                //     Value::Null
                // },
                Expr::Unary(op, expr) => {
                    let value = self._eval(expr).unwrap();
                    match op {
                        UnaryOp::Not => Value::Bool(!value.to_bool()?),
                        UnaryOp::Pos => Value::Number(value.to_number()?),
                        UnaryOp::Neg => Value::Number(-value.to_number()?),
                    }
                },
                Expr::Binary(lhs, op, rhs) => {
                    let lhs = self._eval(lhs).unwrap();
                    let rhs = self._eval(rhs).unwrap();
                    match op {
                        BinaryOp::Pow => Value::Number(lhs.to_number()?.powf(rhs.to_number()?)),
                        BinaryOp::Mul => Value::Number(lhs.to_number()? * rhs.to_number()?),
                        BinaryOp::Div => Value::Number(lhs.to_number()? / rhs.to_number()?),
                        BinaryOp::Mod => Value::Number(lhs.to_number()? % rhs.to_number()?),
                        BinaryOp::Add => Value::Number(lhs.to_number()? + rhs.to_number()?),
                        BinaryOp::Sub => Value::Number(lhs.to_number()? - rhs.to_number()?),
                        BinaryOp::Shl => Value::Number(((lhs.to_number()? as i64) << (rhs.to_number()? as i64)) as f64),
                        BinaryOp::Shr => Value::Number(((lhs.to_number()? as i64) >> (rhs.to_number()? as i64)) as f64),
                        BinaryOp::Lt => Value::Bool(lhs.to_number()? < rhs.to_number()?),
                        BinaryOp::Le => Value::Bool(lhs.to_number()? <= rhs.to_number()?),
                        BinaryOp::Gt => Value::Bool(lhs.to_number()? > rhs.to_number()?),
                        BinaryOp::Ge => Value::Bool(lhs.to_number()? >= rhs.to_number()?),
                        BinaryOp::Eq => Value::Bool(lhs == rhs),
                        BinaryOp::Ne => Value::Bool(lhs != rhs),
                        BinaryOp::BitAnd => Value::Number((lhs.to_number()? as i64 & rhs.to_number()? as i64) as f64),
                        BinaryOp::BitXor => Value::Number((lhs.to_number()? as i64 ^ rhs.to_number()? as i64) as f64),
                        BinaryOp::BitOr => Value::Number((lhs.to_number()? as i64 | rhs.to_number()? as i64) as f64),
                        BinaryOp::And => Value::Bool(lhs.to_bool()? && rhs.to_bool()?),
                        BinaryOp::Or => Value::Bool(lhs.to_bool()? || rhs.to_bool()?),
                    }
                },
                _ => unimplemented!(),
            })
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
                    self.inner[ident] = *(value as Box<dyn Any>).downcast::<Value>().unwrap();
                    Ok(())
                },
            }
        }
    }
    
    #[derive(Debug, Clone)]
    pub enum Value {
        Null,
        Bool(bool),
        Number(f64),
        String(String),
        Array(Vec<Value>),
        Object(BTreeMap<String, Value>),
    }

    impl Value {
        fn to_number(&self) -> Result<f64, Box<dyn Any>> {
            match &self {
                Value::Number(n) => Ok(*n),
                Value::Bool(b) => Ok(if *b { 1. } else { 0. }),
                _ => Err(Box::new(())),
            }
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
            }
        }
    }

    impl super::Value for Value {
        fn to_string(&self) -> Result<String, Box<dyn Any>> {
            match &self {
                Value::Null => Ok("".to_string()),
                // TODO partial object
                _ => Ok(format!("{}", self)),
            }
        }

        fn to_bool(&self) -> Result<bool, Box<dyn Any>> {
            match &self {
                Value::Null => Ok(false),
                Value::Bool(b) => Ok(*b),
                Value::Number(n) => Ok(*n != 0.),
                Value::String(s) => Ok(!s.is_empty()),
                Value::Array(_) => Ok(true),
                Value::Object(_) => Ok(true),
            }
        }

        fn to_entries(&self) -> Result<Vec<(Box<dyn super::Value>, Box<dyn super::Value>)>, /* Self::Error */ Box<dyn Any>> {
            match &self {
                Value::Array(vec) => Ok(vec.iter().enumerate().map(|(k, v)| {
                    let value = Box::new(v.clone()) as Box<dyn super::Value>;
                    let key = Box::new(Value::Number(k as f64)) as Box<dyn super::Value>;
                    (value, key)
                }).collect()),
                Value::Object(map) => Ok(map.iter().map(|(k, v)| {
                    let value = Box::new(v.clone()) as Box<dyn super::Value>;
                    let key = Box::new(Value::String(k.clone())) as Box<dyn super::Value>;
                    (value, key)
                }).collect()),
                _ => Err(Box::new(())),
            }
        }
    }
}
