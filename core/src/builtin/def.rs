use std::fmt::Debug;

use crate::directive::{DirectiveFactory as Directive, Node, Element};
use crate::language::{Context, Definition, Expr, Pattern, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};

/// Definition directive.
/// 
/// ### Example
/// Bind the result of `EXPR` to `PAT`.
/// 
/// ```yfelo
/// {@def PAT = EXPR}
/// ```
/// 
/// ### Example
/// Define a function `NAME` with `PARAMS`.
/// 
/// ```yfelo
/// {#def NAME(PARAMS)}
///     ...
/// {/def}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Def {
    Var(DefVar),
    Fn(DefFn),
}

/// Variable definition. Must have an inline expression.
/// 
/// ```yfelo
/// {@def PAT = EXPR}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DefVar {
    pat: Box<dyn Pattern>,
    expr: Box<dyn Expr>,
}

impl DefVar {
    fn render(&self, ctx: &mut dyn Context, _: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let value = ctx.eval(self.expr.as_ref())?;
        ctx.bind(self.pat.as_ref(), value)?;
        Ok(String::new())
    }
}

/// Function definition, which can be further divided into two types:
/// 
/// 1. With inline expression.
/// ```yfelo
/// {@def NAME(...) = EXPR}
/// ```
/// 
/// 2. With block content (parentheses are optional).
/// ```yfelo
/// {#def NAME(...)}
///     ...
/// {/def}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DefFn {
    ident: String,
    params: Vec<(Box<dyn Pattern>, Option<Box<dyn Expr>>)>,
    expr: Option<Box<dyn Expr>>,
}

impl DefFn {
    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        ctx.def(&self.ident, self.params.clone(), match &self.expr {
            Some(expr) => Definition::Inline(expr.clone()),
            None => Definition::Block(nodes.into()),
        })?;
        Ok(String::new())
    }
}

impl Def {
    fn as_ident(pat: Box<dyn Pattern>) -> Result<String, SyntaxError> {
        pat.into_ident().ok_or_else(|| SyntaxError {
            message: "expected identifier".into(),
            range: (0, 0), // fixme
        })
    }
}

impl Directive for Def {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        let pat = reader.parse_pattern()?;
        Ok(if let Ok(_) = reader.parse_punct("(") {
            let ident = Self::as_ident(pat)?;
            let mut params = vec![];
            loop {
                if let Ok(_) = reader.parse_punct(")") {
                    break;
                }
                let pat = reader.parse_pattern()?;
                let expr = if let Ok(_) = reader.parse_punct("=") {
                    Some(reader.parse_expr()?)
                } else {
                    None
                };
                params.push((pat, expr));
                if let Ok(_) = reader.parse_punct(",") {
                    continue;
                }
                reader.parse_punct(")")?;
                break;
            }
            let expr = if let Ok(_) = reader.parse_punct("=") {
                info.expect_empty()?;
                Some(reader.parse_expr()?)
            } else {
                info.expect_children()?;
                None
            };
            Self::Fn(DefFn { ident, params, expr })
        } else {
            if let Ok(_) = reader.parse_punct("=") {
                info.expect_empty()?;
                let expr = reader.parse_expr()?;
                Self::Var(DefVar { pat, expr })
            } else {
                let ident = Self::as_ident(pat)?;
                info.expect_children()?;
                Self::Fn(DefFn { ident, expr: None, params: vec![] })
            }
        })
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], branches: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        match self {
            Self::Var(def) => def.render(ctx, nodes, branches),
            Self::Fn(def) => def.render(ctx, nodes, branches),
        }
    }
}
