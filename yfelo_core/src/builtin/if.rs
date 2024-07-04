use std::fmt::Debug;

use dyn_std::Instance;

use crate::directive::{DirectiveFactory as Directive, Node, Element};
use crate::language::{Context, Expr, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};
use crate::writer::render;

use super::Stub;

/// Conditional directive.
/// 
/// ### Example
/// ```yfelo
/// {#if EXPR}
///     ...
/// {/if}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct If {
    expr: Box<dyn Expr>,
}

impl Directive for If {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let expr = reader.parse_expr()?;
        Ok(Self { expr })
    }

    fn branch(tags: &[TagInfo], info: &TagInfo) -> Result<(), SyntaxError> {
        if let Some(prev) = tags.last() {
            if prev.name == "else" {
                return Err(SyntaxError {
                    message: format!("'{}' cannot come after 'else'", info.name),
                    range: info.range,
                })
            }
        }
        Ok(())
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], branches: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        if ctx.eval(&self.expr)?.as_bool()? {
            return render(ctx, nodes);
        }
        for branch in branches {
            if let Some(instance) = branch.directive.as_any().downcast_ref::<Instance<If, ()>>() {
                if ctx.eval(&instance.0.expr)?.as_bool()? {
                    return render(ctx, &branch.children);
                }
            } else if let Some(_) = branch.directive.as_any().downcast_ref::<Instance<Stub, ()>>() {
                return render(ctx, &branch.children);
            } else {
                panic!("unexpected directive instance: {:?}", branch.directive)
            }
        }
        Ok(String::new())
    }
}
