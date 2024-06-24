use std::any::Any;

use pest::Parser;
use serde_json::Value;

use crate::error::SyntaxError;
use crate::reader::Reader;

#[derive(Parser)]
#[grammar = "default.pest"]
struct DefaultParser;

pub trait Interpreter {
    // type Expr;
    // type Pattern;
    // type Context;
    // type Value;
    // type Error;

    fn parse_expr(&self, reader: &mut Reader) -> Result</* Self::Expr */ Box<dyn Any>, SyntaxError>;
    fn parse_pattern(&self, reader: &mut Reader) -> Result</* Self::Pattern */ Box<dyn Any>, SyntaxError>;
    fn evaluate(&self, expr: /* Self::Expr */ &dyn Any, ctx: /* Self::Context */ &dyn Any) -> Result</* Self::Value */ Box<dyn Any>, /* Self::Error */ Box<dyn Any>>;
    fn as_string(&self, value: /* Self::Value */ &dyn Any) -> Result<String, /* Self::Error */ Box<dyn Any>>;
    fn as_bool(&self, value: /* Self::Value */ &dyn Any) -> Result<bool, /* Self::Error */ Box<dyn Any>>;

    fn eval_as_string(&self, expr: /* Self::Value */ &dyn Any, ctx: /* Self::Context */ &dyn Any) -> Result<String, /* Self::Error */ Box<dyn Any>> {
        let value = self.evaluate(expr, ctx)?;
        self.as_string(value.as_ref())
    }

    fn eval_as_bool(&self, expr: /* Self::Value */ &dyn Any, ctx: /* Self::Context */ &dyn Any) -> Result<bool, /* Self::Error */ Box<dyn Any>> {
        let value = self.evaluate(expr, ctx)?;
        self.as_bool(value.as_ref())
    }
}

pub struct DefaultInterpreter;

impl Interpreter for DefaultInterpreter {
    // type Expr = ();
    // type Pattern = ();
    // type Context = Value;
    // type Value = Value;
    // type Error = ();

    fn parse_expr(&self, reader: &mut Reader) -> Result<Box<dyn Any>, SyntaxError> {
        let pairs = match DefaultParser::parse(Rule::expr, reader.source) {
            Ok(v) => v,
            Err(e) => return Err(SyntaxError {
                message: e.to_string(),
                range: (0, 0), // TODO
            }),
        };
        reader.skip(pairs.as_str().len());
        reader.trim_start();
        Ok(Box::new(()))
    }

    fn parse_pattern(&self, reader: &mut Reader) -> Result<Box<dyn Any>, SyntaxError> {
        let pairs = match DefaultParser::parse(Rule::ident, reader.source) {
            Ok(v) => v,
            Err(e) => return Err(SyntaxError {
                message: e.to_string(),
                range: (0, 0), // TODO
            }),
        };
        reader.skip(pairs.as_str().len());
        reader.trim_start();
        Ok(Box::new(()))
    }

    fn evaluate(&self, _: &dyn Any, _: &dyn Any) -> Result<Box<dyn Any>, Box<dyn Any>> {
        Ok(Box::new(Value::Null))
    }

    fn as_string(&self, value: &dyn Any) -> Result<String, Box<dyn Any>> {
        Ok(value.downcast_ref::<Value>().unwrap().to_string())
    }

    fn as_bool(&self, _value: &dyn Any) -> Result<bool, Box<dyn Any>> {
        Ok(true)
    }
}
