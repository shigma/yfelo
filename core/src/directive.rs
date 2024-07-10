use std::fmt::Debug;
use std::marker::PhantomData;

use dyn_std::Instance;

use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::{TagInfo, Reader};

#[derive(Debug, Clone, PartialEq)]
pub struct Element {
    pub directive: Box<dyn Directive>,
    pub nodes: Vec<Node>,
    pub branches: Vec<Element>,
}

impl Element {
    pub fn new(directive: Box<dyn Directive>) -> Self {
        Self {
            directive,
            nodes: vec![],
            branches: vec![],
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
    Text(String),
    Expr(Box<dyn Expr>),
    Element(Element),
}

#[dyn_trait]
pub trait Directive: Debug + Clone + PartialEq {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError>;

    fn branch(_tags: &[TagInfo], _info: &TagInfo) -> Result<(), SyntaxError> {
        Ok(())
    }

    fn close(&mut self, _reader: &mut Reader, _info: &TagInfo) -> Result<(), SyntaxError> {
        Ok(())
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], branches: &[Element]) -> Result<String, Box<dyn RuntimeError>>;
}

impl<T: 'static + DirectiveFactory> Directive for PhantomData<T> {
    fn open(&self, reader: &mut Reader, info: &TagInfo) -> Result<Box<dyn Directive>, SyntaxError> {
        Ok(Box::new(Instance::new(<T as DirectiveFactory>::open(reader, info)?)))
    }

    fn close(&mut self, _: &mut Reader, _: &TagInfo) -> Result<(), SyntaxError> {
        unreachable!("unexpected invocation of non-dispatchable function")
    }

    fn render(&self, _: &mut dyn Context, _: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        unreachable!("unexpected invocation of non-dispatchable function")
    }
}
