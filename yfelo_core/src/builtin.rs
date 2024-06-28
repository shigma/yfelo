use std::fmt::Debug;

use crate::directive::{DirectiveConstructor, Node};
use crate::language::{Context, Expr, Pattern, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};
use crate::writer::Writer;

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
#[derive(Debug, PartialEq)]
pub struct Stub;

impl DirectiveConstructor for Stub {
    fn open(reader: &mut Reader, _: &TagInfo) -> Result<Self, SyntaxError> {
        reader.tag_close()?;
        Ok(Self)
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        writer.render(children, ctx)
    }
}

/// Conditional directive.
/// 
/// ### Example
/// ```yfelo
/// {#if EXPR}
///   ...
/// {/if}
/// ```
#[derive(Debug, PartialEq)]
pub struct If {
    expr: Box<dyn Expr>,
}

impl DirectiveConstructor for If {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Self { expr })
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        if ctx.eval(self.expr.as_ref())?.as_bool()? {
            return writer.render(children, ctx);
        }
        Ok(())
    }
}

/// Loop directive.
/// 
/// ### Example
/// ```yfelo
/// {#for PAT in EXPR}
///  ...
/// {/for}
/// ```
#[derive(Debug, PartialEq)]
pub struct For {
    vpat: Box<dyn Pattern>,
    kpat: Option<Box<dyn Pattern>>,
    expr: Box<dyn Expr>,
}

impl DirectiveConstructor for For {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let vpat = reader.parse_pattern()?;
        let kpat = match reader.parse_punct(",") {
            Ok(_) => Some(reader.parse_pattern()?),
            Err(_) => None,
        };
        reader.parse_keyword("in")?;
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Self { vpat, kpat, expr })
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        let entries = ctx.eval(self.expr.as_ref())?.as_entries()?;
        for entry in entries {
            let mut inner = ctx.fork();
            inner.bind(self.vpat.as_ref(), entry.0)?;
            if let Some(kpat) = &self.kpat {
                inner.bind(kpat.as_ref(), entry.1)?;
            }
            writer.render(children, inner.as_mut())?;
        }
        Ok(())
    }
}

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
///   ...
/// {/def}
/// ```
#[derive(Debug, PartialEq)]
pub struct Def {
    pat: Box<dyn Pattern>,
    expr: Box<dyn Expr>,
}

impl DirectiveConstructor for Def {
    fn open(reader: &mut Reader, _: &TagInfo) -> Result<Self, SyntaxError> {
        let pat = reader.parse_pattern()?;
        reader.parse_punct("=")?;
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Self { pat, expr })
    }

    fn render<'i>(&self, _: &mut Writer<'i>, _: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        let value = ctx.eval(self.expr.as_ref())?;
        ctx.bind(self.pat.as_ref(), value)?;
        Ok(())
    }
}
