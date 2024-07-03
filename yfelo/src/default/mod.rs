use std::fmt::Debug;

use super::{SyntaxError, factory};

mod context;
mod expr;
mod operator;
mod parser;
mod pattern;
mod value;

pub use context::*;
pub use expr::*;
pub use operator::*;
pub use pattern::*;
pub use value::*;

pub struct Language;

impl factory::Language<Expr, Pattern> for Language {
    fn parse_expr(input: &str) -> Result<(Expr, usize), SyntaxError> {
        Expr::parse(input)
    }

    fn parse_pattern(input: &str) -> Result<(Pattern, usize), SyntaxError> {
        Pattern::parse(input)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {}

impl factory::RuntimeError for RuntimeError {}
