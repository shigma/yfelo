use std::fmt::Debug;

use dyn_std::Downcast;

use crate::directive::{Directive, Meta, Node};
use crate::language::{Context, Expr, Pattern, RuntimeError, SyntaxError};
use crate::reader::Reader;
use crate::writer::Writer;

#[derive(Debug, PartialEq)]
pub struct StubMeta;

impl Meta for StubMeta {}

pub struct Stub;

impl Directive<StubMeta> for Stub {
    fn parse_open(&self, reader: &mut Reader) -> Result<StubMeta, SyntaxError> {
        reader.tag_close()?;
        Ok(StubMeta)
    }

    fn render<'i>(&self, _: &StubMeta, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        writer.render(children, ctx)
    }
}

#[derive(Debug, PartialEq)]
pub struct IfMeta {
    expr: Box<dyn Expr>,
}

impl Meta for IfMeta {}

pub struct If;

impl Directive<IfMeta> for If {
    fn parse_open(&self, reader: &mut Reader) -> Result<IfMeta, SyntaxError> {
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(IfMeta { expr })
    }

    fn render<'i>(&self, meta: &IfMeta, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        if ctx.eval(meta.expr.as_ref())?.as_bool()? {
            return writer.render(children, ctx);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct ForMeta {
    vpat: Box<dyn Pattern>,
    kpat: Option<Box<dyn Pattern>>,
    expr: Box<dyn Expr>,
}

impl Meta for ForMeta {}

pub struct For;

impl Directive<ForMeta> for For {
    fn parse_open(&self, reader: &mut Reader) -> Result<ForMeta, SyntaxError> {
        let vpat = reader.parse_pattern()?;
        let kpat = match reader.parse_punct(",") {
            Ok(_) => Some(reader.parse_pattern()?),
            Err(_) => None,
        };
        reader.parse_keyword("in")?;
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(ForMeta { vpat, kpat, expr })
    }

    fn render<'i>(&self, meta: &ForMeta, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        let entries = ctx.eval(meta.expr.as_ref())?.as_entries()?;
        for entry in entries {
            let mut inner = ctx.fork();
            inner.bind(meta.vpat.as_ref(), entry.0)?;
            if let Some(kpat) = &meta.kpat {
                inner.bind(kpat.as_ref(), entry.1)?;
            }
            writer.render(children, inner.as_ref())?;
        }
        Ok(())
    }
}

// TODO: auto generate this
macro_rules! impl_directive {
    ($name:ident, $meta:ident) => {
        impl Directive<Box<dyn Meta>> for $name {
            #[inline]
            fn parse_open(&self, reader: &mut Reader) -> Result<Box<dyn Meta>, SyntaxError> {
                match Directive::<$meta>::parse_open(self, reader) {
                    Ok(meta) => Ok(Box::new(meta)),
                    Err(e) => Err(e),
                }
            }

            #[inline]
            fn render<'i>(&self, meta: &Box<dyn Meta>, writer: &mut Writer<'i>, children: &'i Vec<Node>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
                Directive::<$meta>::render(self, meta.as_ref().downcast_ref().unwrap(), writer, children, ctx)
            }
        }
    };
}

impl_directive!(Stub, StubMeta);
impl_directive!(If, IfMeta);
impl_directive!(For, ForMeta);
