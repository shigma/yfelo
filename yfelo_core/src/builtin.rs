use std::fmt::Debug;

use crate::directive::{DirectiveStatic as Directive, Node};
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

impl Directive for Stub {
    fn open(_: &mut Reader, _: &TagInfo) -> Result<Self, SyntaxError> {
        Ok(Self)
    }

    fn render<'i>(&self, writer: &Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<String, Box<dyn RuntimeError>> {
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

impl Directive for If {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let expr = reader.parse_expr()?;
        Ok(Self { expr })
    }

    fn render<'i>(&self, writer: &Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        if ctx.eval(&self.expr)?.as_bool()? {
            return writer.render(children, ctx);
        }
        Ok(String::new())
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

    fn render<'i>(&self, writer: &Writer<'i>, children: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        let entries = ctx.eval(&self.expr)?.as_entries()?;
        let mut output = String::new();
        for entry in entries {
            let mut inner = ctx.fork();
            inner.bind(&self.vpat, entry.0)?;
            if let Some(kpat) = &self.kpat {
                inner.bind(&kpat, entry.1)?;
            }
            output += &writer.render(children, inner.as_mut())?;
        }
        Ok(output)
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
    params: Option<Vec<Box<dyn Pattern>>>,
    expr: Option<Box<dyn Expr>>,
}

impl Directive for Def {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        let pat = reader.parse_pattern()?;
        let params = if let Ok(_) = reader.parse_punct("(") {
            let mut params = vec![];
            loop {
                if let Ok(_) = reader.parse_punct(")") {
                    break;
                }
                params.push(reader.parse_pattern()?);
                if let Ok(_) = reader.parse_punct(",") {
                    continue;
                }
                reader.parse_punct(")")?;
                break;
            }
            Some(params)
        } else {
            None
        };
        let expr = if let Ok(_) = reader.parse_punct("=") {
            info.expect_empty()?;
            Some(reader.parse_expr()?)
        } else {
            info.expect_children()?;
            None
        };
        Ok(Self { pat, params, expr })
    }

    fn render<'i>(&self, writer: &Writer<'i>, nodes: &'i Vec<Node>, ctx: &mut dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        if let Some(_) = &self.params {
            todo!();
        } else {
            if let Some(expr) = &self.expr {
                let value = ctx.eval(expr)?;
                ctx.bind(&self.pat, value)?;
            } else {
                let output = writer.render(nodes, ctx)?;
                let value = ctx.value_from_string(output)?;
                ctx.bind(&self.pat, value)?;
            }
        }
        Ok(String::new())
    }
}
