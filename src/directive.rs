use std::any::Any;

use crate::error::SyntaxError;
use crate::interpreter::Context;
use crate::reader::Reader;
use crate::writer::Writer;
use crate::Element;

pub trait Directive {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Any>, SyntaxError>;
    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>>;
}

pub struct IfMeta {
    expr: Box<dyn Any>,
}

pub struct If;

impl Directive for If {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Any>, SyntaxError> {
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Box::new(IfMeta { expr }))
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        let meta = element.meta.downcast_ref::<IfMeta>().unwrap();
        let bool = ctx.eval(meta.expr.as_ref())?.to_bool()?;
        if bool {
            let nodes = element.children.as_ref().unwrap();
            return writer.render_layer(nodes, ctx);
        }
        Ok(())
    }
}

pub struct ForMeta {
    item: Box<dyn Any>,
    expr: Box<dyn Any>,
}

pub struct For;

impl Directive for For {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Any>, SyntaxError> {
        let item = reader.parse_pattern()?;
        reader.parse_keyword("in")?;
        let expr = reader.parse_expr()?;
        reader.tag_close()?;
        Ok(Box::new(ForMeta { item, expr }))
    }

    fn render<'i>(&self, writer: &mut Writer<'i>, element: &'i Element, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        let _meta = element.meta.downcast_ref::<ForMeta>().unwrap();
        Ok(())
    }
}
