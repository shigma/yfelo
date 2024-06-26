#[macro_use]
extern crate dyn_derive;

#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;

use directive::{For, If, Meta};
use language::Expr;
use reader::Reader;

pub use crate::directive::Directive;
pub use crate::error::{Error, SyntaxError};
pub use crate::language::Language;

pub mod directive;
pub mod error;
pub mod language;
pub mod reader;
pub mod writer;

#[derive(Debug)]
pub struct Element<'i> {
    pub name: &'i str,
    pub meta: Box<dyn Meta>,
    pub children: Option<Vec<Node<'i>>>,
}

impl<'i> PartialEq for Element<'i> {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.children == other.children && self.meta.dyn_eq(other.meta.as_any())
    }
}

#[derive(Debug)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(Box<dyn Expr>),
    Element(Element<'i>),
}

impl<'i> PartialEq for Node<'i> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Node::Text(a), Node::Text(b)) => a == b,
            (Node::Expr(a), Node::Expr(b)) => a.dyn_eq(b.as_any()),
            (Node::Element(a), Node::Element(b)) => a == b,
            _ => false,
        }
    }
}

pub struct MetaSyntax<'i> {
    pub left: &'i str,
    pub right: &'i str,
}

pub struct Yfelo<'i> {
    dirs: HashMap<&'i str, Box<dyn Directive>>,
    langs: HashMap<&'i str, Box<dyn Language>>,
}

impl<'i> Yfelo<'i> {
    pub fn new() -> Self {
        let mut dirs: HashMap<&str, Box<dyn Directive>> = HashMap::new();
        dirs.insert("if", Box::new(If));
        dirs.insert("for", Box::new(For));
        Self {
            dirs,
            langs: HashMap::new(),
        }
    }

    pub fn add_directive(&mut self, name: &'i str, dir: Box<dyn Directive>) {
        self.dirs.insert(name, dir);
    }

    pub fn add_language(&mut self, name: &'i str, lang: Box<dyn Language>) {
        self.langs.insert(name, lang);
    }

    pub fn parse(&'i self, source: &'i str, lang_name: &'i str, meta: &'i MetaSyntax) -> Result<Vec<Node<'i>>, SyntaxError> {
        let lang = self.langs.get(lang_name).unwrap().as_ref();
        Reader::new(source, meta, lang, &self.dirs).parse()
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
