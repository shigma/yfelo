use std::fmt::Debug;

use crate::directive::{Directive, Node};
use crate::language::{Context, Expr, Pattern, RuntimeError, SyntaxError};
use crate::reader::Reader;
use crate::writer::Writer;

#[derive(Debug, PartialEq)]
pub struct Stub;

impl Directive for Stub {
    fn open(reader: &mut Reader) -> Result<Self, SyntaxError> {
        reader.tag_close()?;
        Ok(Self)
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        writer.render(children, ctx)
    }
}

#[derive(Debug, PartialEq)]
pub struct If {
    expr: Box<dyn Expr>,
}

impl Directive for If {
    fn open(reader: &mut Reader) -> Result<Self, SyntaxError> {
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Self { expr })
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        if ctx.eval(self.expr.as_ref())?.as_bool()? {
            return writer.render(children, ctx);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct For {
    vpat: Box<dyn Pattern>,
    kpat: Option<Box<dyn Pattern>>,
    expr: Box<dyn Expr>,
}

impl Directive for For {
    fn open(reader: &mut Reader) -> Result<Self, SyntaxError> {
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

    fn render<'i>(&self, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        let entries = ctx.eval(self.expr.as_ref())?.as_entries()?;
        for entry in entries {
            let mut inner = ctx.fork();
            inner.bind(self.vpat.as_ref(), entry.0)?;
            if let Some(kpat) = &self.kpat {
                inner.bind(kpat.as_ref(), entry.1)?;
            }
            writer.render(children, inner.as_ref())?;
        }
        Ok(())
    }
}
