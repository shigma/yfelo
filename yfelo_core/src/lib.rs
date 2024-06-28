#[macro_use]
extern crate dyn_derive;

use std::collections::HashMap;

use builtin::{For, If};
use reader::Reader;
use writer::Writer;

pub use directive::*;
pub use language::*;

pub mod builtin;
pub mod directive;
pub mod language;
pub mod reader;
pub mod writer;

pub struct MetaSyntax<'i> {
    pub left: &'i str,
    pub right: &'i str,
}

pub struct Yfelo {
    dirs: HashMap<String, Box<dyn Directive>>,
    langs: HashMap<String, Box<dyn Language>>,
}

impl Yfelo {
    pub fn new() -> Self {
        let mut dirs: HashMap<String, Box<dyn Directive>> = HashMap::new();
        dirs.insert("if".into(), Box::new(If));
        dirs.insert("for".into(), Box::new(For));
        Self {
            dirs,
            langs: HashMap::new(),
        }
    }

    pub fn add_directive<T: Into<String>>(&mut self, name: T, dir: Box<dyn Directive>) {
        self.dirs.insert(name.into(), dir);
    }

    pub fn add_language<T: Into<String>>(&mut self, name: T, lang: Box<dyn Language>) {
        self.langs.insert(name.into(), lang);
    }

    pub fn parse<'i>(&'i self, source: &'i str, lang: &'i dyn Language, meta: &'i MetaSyntax) -> Result<Vec<Node<'i>>, SyntaxError> {
        let reader = Reader::new(source, meta, lang, &self.dirs);
        reader.run()
    }

    pub fn render<'i>(&'i self, nodes: &'i Vec<Node<'i>>, lang: &'i dyn Language, ctx: &'i dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        let writer = Writer::new(lang, &self.dirs);
        writer.run(nodes, ctx)
    }

    pub fn run(&self, source: &str, lang: &dyn Language, meta: &MetaSyntax, ctx: &dyn Context) -> Result<String, Error> {
        // let lang = self.langs.get(lang_name).unwrap().as_ref();
        let nodes = self.parse(source, lang, meta).map_err(|e| Error::Syntax(e))?;
        self.render(&nodes, lang, ctx).map_err(|e| Error::Runtime(e))
    }
}