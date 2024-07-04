use std::fmt::Debug;

use crate::directive::{DirectiveFactory as Directive, Node, Element};
use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};
use crate::writer::render;

/// Application directive.
/// 
/// ### Example
/// Apply function `NAME` with `PARAMS`.
/// 
/// ```yfelo
/// {@apply NAME(PARAMS)}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Apply {
    name: String,
    args: Vec<Box<dyn Expr>>,
}

impl Directive for Apply {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        let name = reader.parse_ident()?.0.into();
        let args = if let Ok(_) = reader.parse_punct("(") {
            let mut args = vec![];
            loop {
                if let Ok(_) = reader.parse_punct(")") {
                    break;
                }
                args.push(reader.parse_expr()?);
                if let Ok(_) = reader.parse_punct(",") {
                    continue;
                }
                reader.parse_punct(")")?;
                break;
            }
            args
        } else {
            vec![]
        };
        info.expect_empty()?;
        Ok(Self { name, args })
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let value = ctx.apply(&self.name, self.args.clone(), &mut |ctx| {
            render(ctx, nodes)
        })?;
        Ok(value.to_string()?)
    }
}
