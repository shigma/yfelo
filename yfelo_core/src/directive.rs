use std::fmt::Debug;
use std::marker::PhantomData;

use dyn_std::Downcast;

use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::{TagInfo, Reader};
use crate::writer::Writer;

#[derive(Debug, PartialEq)]
pub struct Element<'i> {
    pub directive: Box<dyn Directive>,
    pub children: Vec<Node<'i>>,
}

#[derive(Debug, PartialEq)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(Box<dyn Expr>),
    Element(Element<'i>),
}

pub trait DirectiveConstructor: Sized + Debug + PartialEq {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError>;

    fn close(&mut self, reader: &mut Reader, _: &TagInfo) -> Result<(), SyntaxError> {
        reader.tag_close()
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>>;
}

#[dyn_trait]
pub trait Directive: Debug + PartialEq {
    fn open(&self, reader: &mut Reader, info: &TagInfo) -> Result<Box<dyn Directive>, SyntaxError>;
    fn close(&mut self, reader: &mut Reader, info: &TagInfo) -> Result<(), SyntaxError>;
    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>>;
}

impl<T: 'static + DirectiveConstructor> Directive for T {
    fn open(&self, reader: &mut Reader, info: &TagInfo) -> Result<Box<dyn Directive>, SyntaxError> {
        Ok(Box::new(<T as DirectiveConstructor>::open(reader, info)?))
    }

    fn close(&mut self, reader: &mut Reader, info: &TagInfo) -> Result<(), SyntaxError> {
        <T as DirectiveConstructor>::close(self.downcast_mut().unwrap(), reader, info)
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        <T as DirectiveConstructor>::render(self.downcast_ref().unwrap(), writer, children, ctx)
    }
}

impl<T: 'static + DirectiveConstructor> Directive for PhantomData<T> {
    fn open(&self, reader: &mut Reader, info: &TagInfo) -> Result<Box<dyn Directive>, SyntaxError> {
        Ok(Box::new(<T as DirectiveConstructor>::open(reader, info)?))
    }

    fn close(&mut self, reader: &mut Reader, info: &TagInfo) -> Result<(), SyntaxError> {
        <T as DirectiveConstructor>::close(self.downcast_mut().unwrap(), reader, info)
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        <T as DirectiveConstructor>::render(self.downcast_ref().unwrap(), writer, children, ctx)
    }
}
