use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub message: String,
    pub range: (usize, usize),
}

#[derive(Debug, Clone)]
pub enum Error {
    Syntax(SyntaxError),
    Runtime(Box<dyn RuntimeError>),
}

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
    fn eval(&self, expr: &dyn Expr) -> Result<Box<dyn Value>, Box<dyn RuntimeError>>;
    fn fork(&self) -> Box<dyn Context>;
    fn bind(&mut self, pattern: &dyn Pattern, value: Box<dyn Value>) -> Result<(), Box<dyn RuntimeError>>;
}

#[dyn_trait]
pub trait Value: Debug + Clone + PartialEq {
    fn to_string(&self) -> Result<String, Box<dyn RuntimeError>>;
    fn as_bool(&self) -> Result<bool, Box<dyn RuntimeError>>;
    fn as_entries(&self) -> Result<Vec<(Box<dyn Value>, Box<dyn Value>)>, Box<dyn RuntimeError>>;
}

#[dyn_trait]
pub trait RuntimeError: Debug + Clone {}
