use std::fmt::Debug;

use dyn_std::Downcast;

use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::Reader;
use crate::writer::Writer;

#[derive(Debug, PartialEq)]
pub struct Element<'i> {
    pub directive: Box<dyn DirectiveConstructor>,
    pub children: Vec<Node<'i>>,
}

#[derive(Debug, PartialEq)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(Box<dyn Expr>),
    Element(Element<'i>),
}

pub trait Directive: Sized + Debug + PartialEq {
    fn open(reader: &mut Reader) -> Result<Self, SyntaxError>;
    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>>;

    fn close(&mut self) -> Result<(), SyntaxError> {
        Ok(())
    }
}

#[dyn_trait]
pub trait DirectiveConstructor: Debug + PartialEq {
    fn open(&self, reader: &mut Reader) -> Result<Box<dyn DirectiveConstructor>, SyntaxError>;
    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>>;
    fn close(&mut self) -> Result<(), SyntaxError>;
}

impl<T: 'static + Directive> DirectiveConstructor for T {
    fn open(&self, reader: &mut Reader) -> Result<Box<dyn DirectiveConstructor>, SyntaxError> {
        Ok(Box::new(T::open(reader)?))
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        self.downcast_ref::<T>().unwrap().render(writer, children, ctx)
    }

    fn close(&mut self) -> Result<(), SyntaxError> {
        self.downcast_mut::<T>().unwrap().close()
    }
}

#[derive(Debug, PartialEq)]
pub struct Constructor<T> {
    phantom: std::marker::PhantomData<T>,
}

impl<T: 'static> Constructor<T> {
    pub fn new() -> Self {
        Self {
            phantom: std::marker::PhantomData,
        }
    }
}

impl<T: Directive + 'static> DirectiveConstructor for Constructor<T> {
    fn open(&self, reader: &mut Reader) -> Result<Box<dyn DirectiveConstructor>, SyntaxError> {
        match <T as Directive>::open(reader) {
            Ok(meta) => Ok(Box::new(meta)),
            Err(e) => Err(e),
        }
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        <T as Directive>::render(self.downcast_ref().unwrap(), writer, children, ctx)
    }

    fn close(&mut self) -> Result<(), SyntaxError> {
        <T as Directive>::close(self.downcast_mut().unwrap())
    }
}
