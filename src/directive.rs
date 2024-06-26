use std::any::Any;
use std::fmt::Debug;

use dyn_std::Downcast;

use crate::error::SyntaxError;
use crate::language::{Context, Expr, Pattern};
use crate::reader::Reader;
use crate::writer::Writer;
use crate::Element;

pub trait Directive {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Meta>, SyntaxError>;
    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>>;
}

#[dyn_trait]
pub trait Meta: Debug + PartialEq {}

#[derive(Debug, PartialEq)]
pub struct StubMeta;

impl Meta for StubMeta {}

pub struct Stub;

impl Directive for Stub {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Meta>, SyntaxError> {
        reader.tag_close()?;
        Ok(Box::new(StubMeta))
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        let nodes = element.children.as_ref().unwrap();
        writer.render_layer(nodes, ctx)
    }
}

#[derive(Debug, PartialEq)]
pub struct IfMeta {
    expr: Box<dyn Expr>,
}

impl Meta for IfMeta {}

pub struct If;

impl Directive for If {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Meta>, SyntaxError> {
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Box::new(IfMeta { expr }))
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        let meta = element.meta.downcast_ref::<IfMeta>().unwrap();
        let bool = ctx.eval(meta.expr.as_ref())?.as_bool()?;
        if bool {
            let nodes = element.children.as_ref().unwrap();
            return writer.render_layer(nodes, ctx);
        }
        Ok(())
    }
}

#[derive(Debug, PartialEq)]
pub struct ForMeta {
    pat: Box<dyn Pattern>,
    expr: Box<dyn Expr>,
}

impl Meta for ForMeta {}

pub struct For;

impl Directive for For {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Meta>, SyntaxError> {
        let pat = reader.parse_pattern()?;
        reader.parse_keyword("in")?;
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Box::new(ForMeta { pat, expr }))
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        let meta = element.meta.downcast_ref::<ForMeta>().unwrap();
        let entries = ctx.eval(meta.expr.as_ref())?.as_entries()?;
        for entry in entries {
            let mut inner = ctx.fork();
            inner.bind(meta.pat.as_ref(), entry.0)?;
            let nodes = element.children.as_ref().unwrap();
            writer.render_layer(nodes, inner.as_ref())?;
        }
        Ok(())
    }
}
