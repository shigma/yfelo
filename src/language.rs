use std::{any::Any, fmt::Debug};

use crate::error::SyntaxError;

pub trait Language {
    fn parse_expr(&self, input: &str) -> Result<(Box<dyn Expr>, usize), SyntaxError>;
    fn parse_pattern(&self, input: &str) -> Result<(Box<dyn Pattern>, usize), SyntaxError>;
}

#[dyn_trait]
pub trait Expr: Debug + Clone + PartialEq {}

#[dyn_trait]
pub trait Pattern: Debug + Clone + PartialEq {}

#[dyn_trait]
pub trait Context {
    fn eval(&self, expr: &dyn Expr) -> Result<Box<dyn Value>, /* Self::Error */ Box<dyn Any>>;
    fn fork(&self) -> Box<dyn Context>;
    fn bind(&mut self, pattern: &dyn Pattern, value: Box<dyn Value>) -> Result<(), /* Self::Error */ Box<dyn Any>>;
}

#[dyn_trait]
pub trait Value: Debug + PartialEq {
    fn to_string(&self) -> Result<String, /* Self::Error */ Box<dyn Any>>;
    fn as_bool(&self) -> Result<bool, /* Self::Error */ Box<dyn Any>>;
    fn as_entries(&self) -> Result<Vec<(Box<dyn Value>, Box<dyn Value>)>, /* Self::Error */ Box<dyn Any>>;
}

pub mod default {
    use std::any::Any;
    use std::collections::BTreeMap;
    use std::{fmt, ops};

    use dyn_std::Downcast;
    use pest::iterators::{Pair, Pairs};
    use pest::Parser;

    use crate::error::SyntaxError;
    use super::Value as _;

    #[derive(Parser)]
    #[grammar = "default.pest"]
    struct DefaultParser;

    pub struct Language;

    impl super::Language for Language {
        fn parse_expr(&self, input: &str) -> Result<(Box<dyn super::Expr>, usize), SyntaxError> {
            match DefaultParser::parse(Rule::expr, input) {
                Ok(pairs) => {
                    let len = pairs.as_str().len();
                    Ok((Box::new(Expr::from(pairs.into_iter().next().unwrap())), len))
                },
                Err(e) => Err(SyntaxError {
                    message: e.to_string(),
                    range: (0, 0), // TODO
                }),
            }
        }

        fn parse_pattern(&self, input: &str) -> Result<(Box<dyn super::Pattern>, usize), SyntaxError> {
            match DefaultParser::parse(Rule::pattern, input) {
                Ok(pairs) => {
                    let len = pairs.as_str().len();
                    Ok((Box::new(Pattern::from(pairs.into_iter().next().unwrap())), len))
                },
                Err(e) => return Err(SyntaxError {
                    message: e.to_string(),
                    range: (0, 0), // TODO
                }),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UnaryOp {
        Not,
        Pos,
        Neg,
    }

    impl UnaryOp {
        fn from(pair: Pair<Rule>) -> Self {
            match pair.as_str() {
                "!" => Self::Not,
                "+" => Self::Pos,
                "-" => Self::Neg,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
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
        Index,
    }

    impl BinaryOp {
        fn from(pair: Pair<Rule>) -> Self {
            match pair.as_str() {
                "**" => Self::Pow,
                "*" => Self::Mul,
                "/" => Self::Div,
                "%" => Self::Mod,
                "+" => Self::Add,
                "-" => Self::Sub,
                "<<" => Self::Shl,
                ">>" => Self::Shr,
                "<" => Self::Lt,
                "<=" => Self::Le,
                ">" => Self::Gt,
                ">=" => Self::Ge,
                "==" => Self::Eq,
                "!=" => Self::Ne,
                "&" => Self::BitAnd,
                "^" => Self::BitXor,
                "|" => Self::BitOr,
                "&&" => Self::And,
                "||" => Self::Or,
                _ => unreachable!(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq)]
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

    macro_rules! left_assoc {
        ($curr:ident, $inner:ident) => {
            fn $curr(pair: Pair<Rule>) -> Self {
                let mut pairs = pair.into_inner();
                let mut expr = Expr::$inner(pairs.next().unwrap());
                while pairs.len() > 0 {
                    let op = BinaryOp::from(pairs.next().unwrap());
                    let rhs = Expr::$inner(pairs.next().unwrap());
                    expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
                }
                expr
            }
        };
    }

    impl Expr {
        fn from(pair: Pair<Rule>) -> Self {
            assert!(matches!(pair.as_rule(), Rule::expr));
            Expr::from_or(pair.into_inner().next().unwrap())
        }

        fn from_list(pairs: Pairs<Rule>) -> Vec<Self> {
            let mut exprs = vec![];
            for pair in pairs {
                match pair.as_rule() {
                    Rule::expr => {
                        exprs.push(Expr::from(pair));
                    },
                    _ => unreachable!(),
                }
            }
            exprs
        }

        fn from_suffix(self, pair: Pair<Rule>) -> Self {
            assert!(matches!(pair.as_rule(), Rule::suffix));
            match pair.as_str().chars().nth(0) {
                Some('(') => {
                    let pairs = pair.into_inner();
                    Expr::Apply(Box::new(self), Expr::from_list(pairs))
                },
                Some('[') => {
                    let pair = pair.into_inner().into_iter().next().unwrap();
                    Expr::Binary(Box::new(self), BinaryOp::Index, Box::from(Expr::from(pair)))
                },
                Some('.') => {
                    let pair = pair.into_inner().into_iter().next().unwrap();
                    let expr = Expr::String(pair.as_str().to_string());
                    Expr::Binary(Box::new(self), BinaryOp::Index, Box::from(expr))
                },
                _ => unreachable!(),
            }
        }

        fn from_atom(pair: Pair<Rule>) -> Self {
            assert!(matches!(pair.as_rule(), Rule::atom));
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::number => Expr::Number(pair.as_str().parse().unwrap()),
                Rule::string => Expr::String(pair.as_str().to_string()),
                Rule::ident => Expr::Ident(pair.as_str().to_string()),
                Rule::array => {
                    let pairs = pair.into_inner();
                    Expr::Array(Expr::from_list(pairs))
                },
                Rule::expr => Expr::from(pair),
                _ => unreachable!("unexpected rule: {:?}", pair.as_rule()),
            }
        }

        fn from_unary(pair: Pair<Rule>) -> Self {
            let pairs = pair.into_inner().collect::<Vec<_>>();
            let index = pairs.iter().position(|pair| pair.as_rule() == Rule::atom).unwrap();
            let mut expr = Expr::from_atom(pairs[index].clone());
            for i in index + 1..pairs.len() {
                expr = Expr::from_suffix(expr, pairs[i].clone());
            }
            for i in (0..index).rev() {
                let op = UnaryOp::from(pairs[i].clone());
                expr = Expr::Unary(op, Box::new(expr));
            }
            expr
        }

        fn from_pow(pair: Pair<Rule>) -> Self {
            let mut pairs = pair.into_inner().rev();
            let mut expr = Expr::from_unary(pairs.next().unwrap());
            while pairs.len() > 0 {
                let op = BinaryOp::from(pairs.next().unwrap());
                let lhs = Expr::from_unary(pairs.next().unwrap());
                expr = Expr::Binary(Box::new(lhs), op, Box::new(expr));
            }
            expr
        }

        left_assoc!(from_mul, from_pow);
        left_assoc!(from_add, from_mul);
        left_assoc!(from_shift, from_add);
        left_assoc!(from_comp, from_shift);
        left_assoc!(from_eq, from_comp);
        left_assoc!(from_bitand, from_eq);
        left_assoc!(from_bitxor, from_bitand);
        left_assoc!(from_bitor, from_bitxor);
        left_assoc!(from_and, from_bitor);
        left_assoc!(from_or, from_and);
    }

    #[derive(Debug, Clone, PartialEq)]
    pub enum Pattern {
        Ident(String),
    }

    impl super::Pattern for Pattern {}

    impl Pattern {
        fn from(pair: Pair<Rule>) -> Self {
            let pair = pair.into_inner().next().unwrap();
            match pair.as_rule() {
                Rule::ident => Pattern::Ident(pair.as_str().to_string()),
                _ => unreachable!(),
            }
        }
    }

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
                        UnaryOp::Not => Value::Bool(!value.as_bool()?),
                        UnaryOp::Pos => Value::Number(value.as_number()?),
                        UnaryOp::Neg => Value::Number(-value.as_number()?),
                    }
                },
                Expr::Binary(lhs, op, rhs) => {
                    let lhs = self._eval(lhs).unwrap();
                    let rhs = self._eval(rhs).unwrap();
                    match op {
                        BinaryOp::Pow => Value::Number(lhs.as_number()?.powf(rhs.as_number()?)),
                        BinaryOp::Mul => Value::Number(lhs.as_number()? * rhs.as_number()?),
                        BinaryOp::Div => Value::Number(lhs.as_number()? / rhs.as_number()?),
                        BinaryOp::Mod => Value::Number(lhs.as_number()? % rhs.as_number()?),
                        BinaryOp::Add => Value::Number(lhs.as_number()? + rhs.as_number()?),
                        BinaryOp::Sub => Value::Number(lhs.as_number()? - rhs.as_number()?),
                        BinaryOp::Shl => Value::Number(((lhs.as_number()? as i64) << (rhs.as_number()? as i64)) as f64),
                        BinaryOp::Shr => Value::Number(((lhs.as_number()? as i64) >> (rhs.as_number()? as i64)) as f64),
                        BinaryOp::Lt => Value::Bool(lhs.as_number()? < rhs.as_number()?),
                        BinaryOp::Le => Value::Bool(lhs.as_number()? <= rhs.as_number()?),
                        BinaryOp::Gt => Value::Bool(lhs.as_number()? > rhs.as_number()?),
                        BinaryOp::Ge => Value::Bool(lhs.as_number()? >= rhs.as_number()?),
                        BinaryOp::Eq => Value::Bool(lhs == rhs),
                        BinaryOp::Ne => Value::Bool(lhs != rhs),
                        BinaryOp::BitAnd => Value::Number((lhs.as_number()? as i64 & rhs.as_number()? as i64) as f64),
                        BinaryOp::BitXor => Value::Number((lhs.as_number()? as i64 ^ rhs.as_number()? as i64) as f64),
                        BinaryOp::BitOr => Value::Number((lhs.as_number()? as i64 | rhs.as_number()? as i64) as f64),
                        BinaryOp::And => Value::Bool(lhs.as_bool()? && rhs.as_bool()?),
                        BinaryOp::Or => Value::Bool(lhs.as_bool()? || rhs.as_bool()?),
                        BinaryOp::Index => match lhs {
                            Value::Array(vec) => {
                                vec.get(rhs.as_number()? as usize).unwrap_or(&Value::Null).clone()
                            },
                            Value::Object(map) => {
                                map.get(rhs.as_string()?).unwrap_or(&Value::Null).clone()
                            },
                            _ => todo!(),
                        },
                    }
                },
                _ => unimplemented!(),
            })
        }
    }

    impl super::Context for Context {
        fn eval<'i>(&'i self, expr: &'i dyn super::Expr) -> Result<Box<dyn super::Value>, Box<dyn Any>> {
            let expr = expr.downcast_ref::<Expr>().unwrap();
            Ok(Box::new(self._eval(expr)?))
        }

        fn fork(&self) -> Box<dyn super::Context> {
            Box::new(Context {
                inner: self.inner.clone(),
            })
        }

        fn bind(&mut self, pattern: &dyn super::Pattern, value: Box<dyn super::Value>) -> Result<(), /* Self::Error */ Box<dyn Any>> {
            match pattern.downcast_ref::<Pattern>().unwrap() {
                Pattern::Ident(ident) => {
                    self.inner[ident] = value.downcast_ref::<Value>().unwrap().clone();
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
        pub fn as_number(&self) -> Result<f64, Box<dyn Any>> {
            match &self {
                Value::Number(n) => Ok(*n),
                Value::Bool(b) => Ok(if *b { 1. } else { 0. }),
                _ => Err(Box::new(())),
            }
        }

        pub fn as_string(&self) -> Result<&String, Box<dyn Any>> {
            match &self {
                Value::String(s) => Ok(s),
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

        fn as_bool(&self) -> Result<bool, Box<dyn Any>> {
            match &self {
                Value::Null => Ok(false),
                Value::Bool(b) => Ok(*b),
                Value::Number(n) => Ok(*n != 0.),
                Value::String(s) => Ok(!s.is_empty()),
                Value::Array(_) => Ok(true),
                Value::Object(_) => Ok(true),
            }
        }

        fn as_entries(&self) -> Result<Vec<(Box<dyn super::Value>, Box<dyn super::Value>)>, /* Self::Error */ Box<dyn Any>> {
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