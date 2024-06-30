use std::fmt::Debug;
use std::marker::PhantomData;

use dyn_std::Instance;

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

#[dyn_trait]
pub trait Directive: Sized + Debug + PartialEq {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError>;

    fn close(&mut self, _reader: &mut Reader, _info: &TagInfo) -> Result<(), SyntaxError> {
        Ok(())
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>>;
}

impl<T: 'static + DirectiveStatic> Directive for PhantomData<T> {
    fn open(&self, reader: &mut Reader, info: &TagInfo) -> Result<Box<dyn Directive>, SyntaxError> {
        Ok(Box::new(Instance::new(<T as DirectiveStatic>::open(reader, info)?)))
    }

    fn close(&mut self, _: &mut Reader, _: &TagInfo) -> Result<(), SyntaxError> {
        unreachable!("unexpected invocation of non-dispatchable function")
    }

    fn render<'i>(&self, _: &mut Writer<'i>, _: &'i Vec<Node>, _: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        unreachable!("unexpected invocation of non-dispatchable function")
    }
}
