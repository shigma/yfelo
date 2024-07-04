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

    pub fn parse<'i>(&'i self, source: &'i str, lang: &'i dyn Language, meta: &'i MetaSyntax) -> Result<Vec<Node>, SyntaxError> {
        let reader = Reader::new(source, meta, lang, &self.dirs);
        reader.run()
    }

    pub fn render<'i>(&'i self, source: &'i str, lang: &'i dyn Language, meta: &'i MetaSyntax, ctx: &mut dyn Context) -> Result<String, Error> {
        let nodes = self.parse(source, lang, meta).map_err(|e| Error::Syntax(e))?;
        render(ctx, &nodes).map_err(|e| Error::Runtime(e))
    }
}
