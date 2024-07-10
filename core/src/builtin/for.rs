use std::fmt::Debug;

use crate::directive::{DirectiveFactory as Directive, Node, Element};
use crate::language::{Context, Expr, Pattern, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};
use crate::writer::render;

/// Loop directive.
/// 
/// ### Example
/// ```yfelo
/// {#for PAT in EXPR}
///     ...
/// {/for}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct For {
    pub vpat: Box<dyn Pattern>,
    pub kpat: Option<Box<dyn Pattern>>,
    pub expr: Box<dyn Expr>,
}

impl Directive for For {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let vpat = reader.parse_pattern()?;
        let kpat = match reader.parse_punct(",") {
            Ok(_) => Some(reader.parse_pattern()?),
            Err(_) => None,
        };
        reader.parse_keyword("in")?;
        let expr = reader.parse_expr()?;
        Ok(Self { vpat, kpat, expr })
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let entries = ctx.eval(self.expr.as_ref())?.as_entries()?;
        let mut output = String::new();
        for entry in entries {
            let mut fork = ctx.fork();
            fork.bind(self.vpat.as_ref(), entry.0)?;
            if let Some(kpat) = &self.kpat {
                fork.bind(kpat.as_ref(), entry.1)?;
            }
            output += &render(fork.as_mut(), nodes)?;
        }
        Ok(output)
    }
}
