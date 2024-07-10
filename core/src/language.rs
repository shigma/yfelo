use std::{fmt::Debug, marker::PhantomData};

use dyn_std::Instance;

use crate::Node;

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

#[dyn_trait]
pub trait Language<#[dynamic] E: Expr, #[dynamic] P: Pattern> {
    fn parse_expr(input: &str, offset: usize) -> Result<(E, usize), SyntaxError>;
    fn parse_pattern(input: &str, offset: usize) -> Result<(P, usize), SyntaxError>;
}

#[dyn_trait]
pub trait Expr: Debug + Clone + PartialEq {}

#[dyn_trait]
pub trait Pattern: Debug + Clone + PartialEq {
    fn into_ident(self) -> Option<String>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Definition {
    Inline(Box<dyn Expr>),
    Block(Vec<Node>),
}

#[dyn_trait]
pub trait Context<#[dynamic] E: Expr, #[dynamic] P: Pattern, #[dynamic] V: Value<R>, #[dynamic] R: RuntimeError> {
    fn eval(&self, expr: &E) -> Result<V, R>;
    fn fork(&self) -> Self;
    fn bind(&mut self, pattern: &P, value: V) -> Result<(), R>;
    fn def(&mut self, name: &str, params: Vec<(P, Option<E>)>, definition: Definition) -> Result<(), R>;
    fn apply(&self, name: &str, args: Vec<E>, init: &mut dyn FnMut(&mut dyn Context) -> Result<String, Box<dyn RuntimeError>>) -> Result<V, R>;
}

#[dyn_trait]
pub trait Value<#[dynamic] R: RuntimeError>: Debug + Clone + PartialEq {
    fn to_string(&self) -> Result<String, R>;
    fn as_bool(&self) -> Result<bool, R>;
    fn as_entries(&self) -> Result<Vec<(Self, Self)>, R>;
}

#[dyn_trait]
pub trait RuntimeError: Debug + Clone {}

impl<E: ExprFactory, P: PatternFactory, T: LanguageFactory<E, P>> Language for PhantomData<(T, E, P)> {
    fn parse_expr(&self, input: &str, offset: usize) -> Result<(Box<dyn Expr>, usize), SyntaxError> {
        let (a, b) = <T as LanguageFactory<E, P>>::parse_expr(input, offset)?;
        Ok((Box::new(Instance::new(a)), b))
    }

    fn parse_pattern(&self, input: &str, offset: usize) -> Result<(Box<dyn Pattern>, usize), SyntaxError> {
        let (a, b) = <T as LanguageFactory<E, P>>::parse_pattern(input, offset)?;
        Ok((Box::new(Instance::new(a)), b))
    }
}
