use std::fmt::Debug;

use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::Reader;
use crate::writer::Writer;

#[derive(Debug, PartialEq)]
pub struct Element<'i> {
    pub name: &'i str,
    pub meta: Box<dyn Meta>,
    pub children: Option<Vec<Node<'i>>>,
}

#[derive(Debug, PartialEq)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(Box<dyn Expr>),
    Element(Element<'i>),
}

#[dyn_trait]
pub trait Directive {
    fn parse_open(&self, reader: &mut Reader) -> Result<Box<dyn Meta>, SyntaxError>;
    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>>;
}

#[dyn_trait]
pub trait Meta: Debug + PartialEq {}
