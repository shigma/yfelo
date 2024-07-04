use pest::{iterators::Pair, Parser};
use yfelo_core::{factory, SyntaxError};

use super::parser::{DefaultParser, Rule, ToRange};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ident(String, Option<(usize, usize)>),
}

impl factory::Pattern for Pattern {
    fn into_ident(self) -> Option<String> {
        match self {
            Pattern::Ident(ident, _) => Some(ident),
        }
    }
}

impl Pattern {
    pub fn parse(input: &str, offset: usize) -> Result<(Pattern, usize), SyntaxError> {
        match DefaultParser::parse(Rule::pattern, input) {
            Ok(pairs) => {
                let len = pairs.as_str().len();
                Ok((Pattern::from(pairs.into_iter().next().unwrap(), offset), len))
            },
            Err(e) => return Err(SyntaxError {
                message: e.to_string(),
                range: e.location.to_range(offset),
            }),
        }
    }

    fn from(pair: Pair<Rule>, offset: usize) -> Self {
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::ident => Pattern::Ident(pair.as_str().to_string(), Some(pair.to_range(offset))),
            _ => unreachable!(),
        }
    }
}

impl<T: Into<String>> From<T> for Pattern {
    fn from(value: T) -> Self {
        Pattern::Ident(value.into(), None)
    }
}
