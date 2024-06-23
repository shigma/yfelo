use std::any::Any;

use crate::error::SyntaxError;
use crate::parser::Input;

pub trait Directive {
    fn parse(&self, input: &mut Input) -> Result<Box<dyn Any>, SyntaxError>;
}

pub struct IfMeta {
    expr: Box<dyn Any>,
}

pub struct If;

impl Directive for If {
    fn parse(&self, input: &mut Input) -> Result<Box<dyn Any>, SyntaxError> {
        let expr = input.expect_expr()?;
        input.expect_close()?;
        Ok(Box::new(IfMeta { expr }))
    }
}

pub struct ForMeta {
    item: Box<dyn Any>,
    expr: Box<dyn Any>,
}

pub struct For;

impl Directive for For {
    fn parse(&self, input: &mut Input) -> Result<Box<dyn Any>, SyntaxError> {
        let item = input.expect_pattern()?;
        input.expect_keyword("in")?;
        let expr = input.expect_expr()?;
        input.expect_close()?;
        Ok(Box::new(ForMeta { item, expr }))
    }
}
