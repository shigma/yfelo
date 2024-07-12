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

    pub fn prepare<'i, 'j>(&'i self, input: &'j str) -> Result<YfeloFile<'i, 'j>, SyntaxError> {
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
        let remain = input[header.len()..].trim_ascii_start();
        Ok(YfeloFile {
            input: remain,
            meta: MetaSyntax { left, right },
            lang: lang.as_ref(),
            yfelo: self,
        })
    }

    pub fn parse(&self, input: &str) -> Result<Vec<Node>, SyntaxError> {
        self.prepare(input)?.parse()
    }

    pub fn render(&self, input: &str, ctx: &mut dyn Context) -> Result<String, Error> {
        self.prepare(input).map_err(Error::Syntax)?.render(ctx)
    }
}

pub struct MetaSyntax<'i> {
    pub left: &'i str,
    pub right: &'i str,
}

pub struct YfeloFile<'i, 'j> {
    yfelo: &'i Yfelo,
    lang: &'i dyn Language,
    input: &'j str,
    meta: MetaSyntax<'j>,
}

impl<'i, 'j> YfeloFile<'i, 'j> {
    pub fn parse(&self) -> Result<Vec<Node>, SyntaxError> {
        let reader = Reader::new(self.input, &self.meta, self.lang, &self.yfelo.dirs);
        reader.run()
    }

    pub fn render(&self, ctx: &mut dyn Context) -> Result<String, Error> {
        let nodes = self.parse().map_err(Error::Syntax)?;
        render(ctx, &nodes).map_err(Error::Runtime)
    }
}
