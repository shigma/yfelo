use std::fmt::Debug;

use crate::directive::{DirectiveFactory as Directive, Node, Element};
use crate::language::{Context, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};
use crate::writer::render;

/// No-op directive.
/// 
/// ### Example
/// ```yfelo
/// {@stub}
/// ```
/// 
/// ### Example
/// ```yfelo
/// {#stub}
///   ...
/// {/stub}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Stub;

impl Directive for Stub {
    fn open(_: &mut Reader, _: &TagInfo) -> Result<Self, SyntaxError> {
        Ok(Self)
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let mut fork = ctx.fork();
        render(fork.as_mut(), nodes)
    }
}
