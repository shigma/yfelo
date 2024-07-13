#[macro_use]
extern crate dyn_derive;

use std::collections::HashMap;
use std::marker::PhantomData;

use builtin::{Apply, Def, For, If, Stub};
use reader::Reader;

pub use directive::*;
pub use language::*;
use writer::render;

/// Built-in directives of Yfelo language.
pub mod builtin;
pub mod directive;
pub mod factory;
pub mod language;
pub mod reader;
pub mod writer;

pub struct Yfelo {
    dirs: HashMap<String, Box<dyn Directive>>,
    langs: HashMap<String, Box<dyn Language>>,
}

impl Yfelo {
    pub fn new() -> Self {
        let mut dirs: HashMap<String, Box<dyn Directive>> = HashMap::new();
        dirs.insert("apply".into(), Box::new(PhantomData::<Apply>));
        dirs.insert("def".into(), Box::new(PhantomData::<Def>));
        dirs.insert("if".into(), Box::new(PhantomData::<If>));
        dirs.insert("if:else".into(), Box::new(PhantomData::<Stub>));
        dirs.insert("if:elif".into(), Box::new(PhantomData::<If>));
        dirs.insert("for".into(), Box::new(PhantomData::<For>));
        Self {
            dirs,
            langs: HashMap::new(),
        }
    }

    pub fn add_directive<D: DirectiveFactory>(&mut self, name: impl Into<String>) {
        self.dirs.insert(name.into(), Box::new(PhantomData::<D>));
    }

    pub fn add_language<E: ExprFactory, P: PatternFactory, T: LanguageFactory<E, P>>(&mut self, name: impl Into<String>) {
        self.langs.insert(name.into(), Box::new(PhantomData::<(T, E, P)>));
    }

    pub fn prepare<'i, 'j>(&'i self, input: &'j str, with_offset: bool) -> Result<Header<'i, 'j>, SyntaxError> {
        let header = input.split('\n').next().unwrap();
        let Some(pos) = header.find("@yfelo") else {
            return Err(SyntaxError {
                message: "missing @yfelo directive".into(),
                range: (0, 0),
            })
        };
        let left = header[0..pos].trim_ascii();
        let right = header[pos + 6..].trim_ascii();
        let Some(lang) = self.langs.get("default") else {
            return Err(SyntaxError {
                message: "missing default language".into(),
                range: (0, 0),
            })
        };
        let offset = if with_offset {
            let len = input.len();
            let body = input[header.len()..].trim_ascii_start();
            len - body.len()
        } else {
            0
        };
        Ok(Header {
            meta: MetaSyntax { left, right },
            lang: lang.as_ref(),
            dirs: &self.dirs,
            offset,
        })
    }

    pub fn parse(&self, input: &str) -> Result<Vec<Node>, SyntaxError> {
        let header = self.prepare(input, true)?;
        header.parse(&input[header.offset..])
    }

    pub fn render(&self, input: &str, ctx: &mut dyn Context) -> Result<String, Error> {
        let header = self.prepare(input, true).map_err(Error::Syntax)?;
        header.render(&input[header.offset..], ctx)
    }
}

pub struct MetaSyntax<'i> {
    pub left: &'i str,
    pub right: &'i str,
}

pub struct Header<'i, 'j> {
    dirs: &'i HashMap<String, Box<dyn Directive>>,
    lang: &'i dyn Language,
    meta: MetaSyntax<'j>,
    offset: usize,
}

impl<'i, 'j> Header<'i, 'j> {
    pub fn parse(&self, input: &str) -> Result<Vec<Node>, SyntaxError> {
        let reader = Reader::new(input, self.offset, &self.meta, self.lang, &self.dirs);
        reader.run()
    }

    pub fn render(&self, input: &str, ctx: &mut dyn Context) -> Result<String, Error> {
        let nodes = self.parse(input).map_err(Error::Syntax)?;
        render(ctx, &nodes).map_err(Error::Runtime)
    }
}
