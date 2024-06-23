use crate::error::SyntaxError;
use crate::interpreter::{Input, Interpreter};

pub trait Directive<E, P>: Sized {
    fn parse<C, V, R>(&self, lang: &dyn Interpreter<Expr = E, Pattern = P, Context = C, Value = V, Error = R>, input: &mut Input) -> Result<Self, SyntaxError>;
}

pub struct If<E> {
    expr: E,
}

impl<E, P> Directive<E, P> for If<E> {
    fn parse<C, V, R>(&self, lang: &dyn Interpreter<Expr = E, Pattern = P, Context = C, Value = V, Error = R>, input: &mut Input) -> Result<Self, SyntaxError> {
        let expr = lang.parse_expr(input)?;
        Ok(Self { expr })
    }
}

pub struct For<E, P> {
    item: P,
    expr: E,
}

impl<E, P> Directive<E, P> for For<E, P> {
    fn parse<C, V, R>(&self, lang: &dyn Interpreter<Expr = E, Pattern = P, Context = C, Value = V, Error = R>, input: &mut Input) -> Result<Self, SyntaxError> {
        let item = lang.parse_pattern(input)?;
        input.expect_word("in")?;
        let expr = lang.parse_expr(input)?;
        Ok(Self { item, expr })
    }
}
