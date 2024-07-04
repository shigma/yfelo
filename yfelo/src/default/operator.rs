use super::{RuntimeError, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Not,
    Pos,
    Neg,
}

impl UnaryOp {
    pub fn from(input: &str) -> Self {
        match input {
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
}

impl BinaryOp {
    pub fn from(input: &str) -> Self {
        match input {
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
            Self::Add => match lhs.deref() {
                Value::String(lhs) => Value::String(lhs.clone() + &rhs.as_string()?),
                Value::Number(lhs) => Value::Number(*lhs + rhs.as_number()?),
                _ => return Err(RuntimeError {
                    message: format!("cannot add {} and {}", lhs.type_name(), rhs.type_name()),
                }),
            },
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
        })
    }
}
