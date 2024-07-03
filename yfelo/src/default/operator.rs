use pest::iterators::Pair;

use super::{parser::Rule, RuntimeError, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Pos,
    Neg,
}

impl UnaryOp {
    pub fn from(pair: Pair<Rule>) -> Self {
        match pair.as_str() {
            "!" => Self::Not,
            "+" => Self::Pos,
            "-" => Self::Neg,
            _ => unreachable!(),
        }
    }

    pub fn eval(&self, value: Value) -> Result<Value, RuntimeError> {
        Ok(match self {
            Self::Not => Value::Bool(!value.as_bool()?),
            Self::Pos => Value::Number(value.as_number()?),
            Self::Neg => Value::Number(-value.as_number()?),
        })
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
    pub fn from(pair: Pair<Rule>) -> Self {
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

    pub fn eval(&self, lhs: Value, rhs: Value) -> Result<Value, RuntimeError> {
        Ok(match self {
            Self::Pow => Value::Number(lhs.as_number()?.powf(rhs.as_number()?)),
            Self::Mul => Value::Number(lhs.as_number()? * rhs.as_number()?),
            Self::Div => Value::Number(lhs.as_number()? / rhs.as_number()?),
            Self::Mod => Value::Number(lhs.as_number()? % rhs.as_number()?),
            Self::Add => Value::Number(lhs.as_number()? + rhs.as_number()?),
            Self::Sub => Value::Number(lhs.as_number()? - rhs.as_number()?),
            Self::Shl => Value::Number(((lhs.as_number()? as i64) << (rhs.as_number()? as i64)) as f64),
            Self::Shr => Value::Number(((lhs.as_number()? as i64) >> (rhs.as_number()? as i64)) as f64),
            Self::Lt => Value::Bool(lhs.as_number()? < rhs.as_number()?),
            Self::Le => Value::Bool(lhs.as_number()? <= rhs.as_number()?),
            Self::Gt => Value::Bool(lhs.as_number()? > rhs.as_number()?),
            Self::Ge => Value::Bool(lhs.as_number()? >= rhs.as_number()?),
            Self::Eq => Value::Bool(lhs == rhs),
            Self::Ne => Value::Bool(lhs != rhs),
            Self::BitAnd => Value::Number((lhs.as_number()? as i64 & rhs.as_number()? as i64) as f64),
            Self::BitXor => Value::Number((lhs.as_number()? as i64 ^ rhs.as_number()? as i64) as f64),
            Self::BitOr => Value::Number((lhs.as_number()? as i64 | rhs.as_number()? as i64) as f64),
            Self::And => Value::Bool(lhs.as_bool()? && rhs.as_bool()?),
            Self::Or => Value::Bool(lhs.as_bool()? || rhs.as_bool()?),
            Self::Index => match lhs {
                Value::Array(vec) => {
                    vec.get(rhs.as_number()? as usize).unwrap_or(&Value::Null).clone()
                },
                Value::Object(map) => {
                    map.get(rhs.as_string()?).unwrap_or(&Value::Null).clone()
                },
                _ => todo!(),
            },
        })
    }
}
