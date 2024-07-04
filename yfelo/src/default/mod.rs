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
    fn parse_expr(input: &str, offset: usize) -> Result<(Expr, usize), SyntaxError> {
        Expr::parse(input, offset)
    }

    fn parse_pattern(input: &str, offset: usize) -> Result<(Pattern, usize), SyntaxError> {
        Pattern::parse(input, offset)
    }
}

#[derive(Debug, Clone)]
pub struct RuntimeError {
    pub message: String,
}

impl factory::RuntimeError for RuntimeError {}
