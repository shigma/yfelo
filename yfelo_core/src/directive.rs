use std::fmt::Debug;

use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::Reader;
use crate::writer::Writer;

#[derive(Debug, PartialEq)]
pub struct Element<'i> {
    pub name: &'i str,
    pub meta: Box<dyn Meta>,
    pub children: Vec<Node<'i>>,
}

#[derive(Debug, PartialEq)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(Box<dyn Expr>),
    Element(Element<'i>),
}

#[dyn_trait]
pub trait Directive<M = Box<dyn Meta>> {
    fn parse_open(&self, reader: &mut Reader) -> Result<M, SyntaxError>;
    fn render<'i>(&self, this: &M, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>>;
}

#[dyn_trait]
pub trait Meta: Debug + PartialEq {}
