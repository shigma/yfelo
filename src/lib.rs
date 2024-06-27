#[macro_use]
extern crate dyn_derive;

#[macro_use]
extern crate pest_derive;

use std::collections::HashMap;

use directive::{For, If, Meta};
use language::{Context, Error, Expr, RuntimeError, SyntaxError};
use reader::Reader;
use writer::Writer;

pub use crate::directive::Directive;
pub use crate::language::Language;

pub mod directive;
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

    pub fn parse(&'i self, source: &'i str, lang: &'i dyn Language, meta: &'i MetaSyntax) -> Result<Vec<Node<'i>>, SyntaxError> {
        let reader = Reader::new(source, meta, lang, &self.dirs);
        reader.run()
    }

    pub fn render(&'i self, nodes: &'i Vec<Node<'i>>, lang: &'i dyn Language, ctx: &'i dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        let writer = Writer::new(lang, &self.dirs);
        writer.run(nodes, ctx)
    }

    pub fn run(&'i self, source: &'i str, lang: &'i dyn Language, meta: &'i MetaSyntax, ctx: &'i dyn Context) -> Result<String, Error> {
        // let lang = self.langs.get(lang_name).unwrap().as_ref();
        let nodes = self.parse(source, lang, meta).map_err(|e| Error::Syntax(e))?;
        self.render(&nodes, lang, ctx).map_err(|e| Error::Runtime(e))
    }
}
