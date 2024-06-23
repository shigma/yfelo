#[macro_use]
extern crate pest_derive;

use std::any::Any;
use std::collections::HashMap;

use reader::Reader;

pub use crate::directive::Directive;
pub use crate::error::{Error, SyntaxError};
pub use crate::interpreter::Interpreter;

pub mod directive;
pub mod error;
pub mod interpreter;
pub mod reader;

#[derive(Debug)]
pub struct Element<'i> {
    pub name: &'i str,
    pub meta: Box<dyn Any>,
    pub children: Option<Vec<Node<'i>>>,
}

#[derive(Debug)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(Box<dyn Any>),
    Element(Element<'i>),
}

pub struct MetaSyntax {
    left: String,
    right: String,
}

pub struct Yfelo<'i> {
    meta: MetaSyntax,
    dirs: HashMap<&'i str, &'i dyn Directive>,
    lang: &'i dyn Interpreter,
}

impl<'i> Yfelo<'i> {
    pub fn new(meta: MetaSyntax, interpreter: &'i dyn Interpreter) -> Self {
        Self {
            meta,
            dirs: HashMap::new(),
            lang: interpreter,
        }
    }

    pub fn parse(&'i self, source: &'i str) -> Result<Vec<Node<'i>>, SyntaxError> {
        Reader::new(source, &self.meta, self.lang, &self.dirs).parse()
    }

    // pub fn transform(&'i self, reader: &'i str, ctx: &'i C) -> Result<String, Error<R>> {
    //     let nodes = self.parse(reader).map_err(|e| Error::Syntax(e))?;
    //     self.transform_nodes(nodes, ctx)
    // }

    // fn transform_nodes(&'i self, nodes: Vec<Node<'i>>, ctx: &'i C) -> Result<String, Error<R>> {
    //     let mut output = String::new();
    //     for node in nodes {
    //         match node {
    //             Node::Text(text) => output += text,
    //             Node::Expr(expr) => {
    //                 let value = self.interpreter.eval(expr, ctx)?;
    //                 output += &self.interpreter.serialize(&value);
    //             },
    //             _ => unimplemented!(),
    //         }
    //     }
    //     Ok(output)
    // }
}
