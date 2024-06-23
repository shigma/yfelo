use std::any::Any;

use crate::error::SyntaxError;
use crate::reader::Reader;

pub trait Directive {
    fn parse(&self, reader: &mut Reader) -> Result<Box<dyn Any>, SyntaxError>;
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
}
