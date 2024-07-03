use pest::{iterators::Pair, Parser};
use yfelo_core::{factory, SyntaxError};

use super::parser::{make_span, DefaultParser, Rule};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ident(String),
}

impl factory::Pattern for Pattern {
    fn into_ident(self) -> Option<String> {
        match self {
            Pattern::Ident(ident) => Some(ident),
        }
    }
}

impl Pattern {
    pub fn parse(input: &str) -> Result<(Pattern, usize), SyntaxError> {
        match DefaultParser::parse(Rule::pattern, input) {
            Ok(pairs) => {
                let len = pairs.as_str().len();
                Ok((Pattern::from(pairs.into_iter().next().unwrap()), len))
            },
            Err(e) => return Err(SyntaxError {
                message: e.to_string(),
                range: make_span(e.location),
            }),
        }
    }

    fn from(pair: Pair<Rule>) -> Self {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::ident => Pattern::Ident(pair.as_str().to_string()),
            _ => unreachable!(),
        }
    }
}

impl<T: Into<String>> From<T> for Pattern {
    fn from(value: T) -> Self {
        Pattern::Ident(value.into())
    }
}
