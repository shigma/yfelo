use std::any::Any;

use crate::error::SyntaxError;

pub trait Interpreter {
    fn parse_expr(&self, input: &str) -> Result<(/* Self::Expr */ Box<dyn Any>, usize), SyntaxError>;
    fn parse_pattern(&self, input: &str) -> Result<(/* Self::Pattern */ Box<dyn Any>, usize), SyntaxError>;
}

pub trait Context {
    fn eval(&self, expr: /* Self::Expr */ &dyn Any) -> Result<Box<dyn Value>, /* Self::Error */ Box<dyn Any>>;
    fn fork(&self) -> Box<dyn Context>;
    fn bind(&mut self, pattern: /* Self::Pattern */ &dyn Any, value: Box<dyn Value>) -> Result<(), /* Self::Error */ Box<dyn Any>>;
}

pub trait Value {
    fn to_string(&self) -> Result<String, /* Self::Error */ Box<dyn Any>>;
    fn to_bool(&self) -> Result<bool, /* Self::Error */ Box<dyn Any>>;
}

pub mod default {
    use std::any::Any;

    use pest::Parser;
    use serde_json::Value as JsonValue;

    use crate::error::SyntaxError;

    #[derive(Parser)]
    #[grammar = "default.pest"]
    struct DefaultParser;

    pub struct Interpreter;

    impl super::Interpreter for Interpreter {
        fn parse_expr(&self, input: &str) -> Result<(Box<dyn Any>, usize), SyntaxError> {
            match DefaultParser::parse(Rule::expr, input) {
                Ok(pairs) => Ok((Box::new(()), pairs.as_str().len())),
                Err(e) => Err(SyntaxError {
                    message: e.to_string(),
                    range: (0, 0), // TODO
                }),
            }
        }

        fn parse_pattern(&self, input: &str) -> Result<(Box<dyn Any>, usize), SyntaxError> {
            match DefaultParser::parse(Rule::ident, input) {
                Ok(pairs) => Ok((Box::new(()), pairs.as_str().len())),
                Err(e) => return Err(SyntaxError {
                    message: e.to_string(),
                    range: (0, 0), // TODO
                }),
            }
        }
    }

    pub struct Context {
        value: JsonValue,
    }

    impl super::Context for Context {
        fn eval(&self, _expr: &dyn Any) -> Result<Box<dyn super::Value>, Box<dyn Any>> {
            Ok(Box::new(Value {
                value: JsonValue::Null,
            }))
        }

        fn fork(&self) -> Box<dyn super::Context> {
            Box::new(Context {
                value: self.value.clone(),
            })
        }

        fn bind(&mut self, _pattern: /* Self::Pattern */ &dyn Any, _value: Box<dyn super::Value>) -> Result<(), /* Self::Error */ Box<dyn Any>> {
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
    }
}
