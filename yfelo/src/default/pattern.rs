use std::fmt;

use pest::iterators::Pairs;
use pest::{iterators::Pair, Parser};
use yfelo_core::{factory, SyntaxError};

use super::parser::{DefaultParser, Rule, ToRange};

#[derive(Debug, Clone, PartialEq)]
pub enum Pattern {
    Ident(String, Option<(usize, usize)>),
    Array(Vec<Pattern>, Option<(usize, usize)>),
    Object(Vec<(Expr, Option<Pattern>)>, Option<(usize, usize)>),
}

impl factory::Pattern for Pattern {
    fn into_ident(self) -> Option<String> {
        match self {
            Pattern::Ident(ident, _) => Some(ident),
            _ => None,
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
        let range = Some(pair.to_range(offset));
        match pair.as_rule() {
            Rule::ident => Pattern::Ident(pair.as_str().to_string(), range),
            Rule::pat_array => {
                let pairs = pair.into_inner();
                Self::Array(Self::from_list(pairs, offset), range)
            },
            _ => unreachable!(),
        }
    }

    fn from_list(pairs: Pairs<Rule>, offset: usize) -> Vec<Self> {
        let mut exprs = vec![];
        for pair in pairs {
            exprs.push(Self::from(pair, offset));
        }
        exprs
    }
}

impl<T: Into<String>> From<T> for Pattern {
    fn from(value: T) -> Self {
        Pattern::Ident(value.into(), None)
    }
}

impl fmt::Display for Pattern {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Ident(ident, _) => write!(f, "{}", ident),
            Self::Array(vec, _) => {
                write!(f, "[")?;
                for (i, pattern) in vec.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", pattern)?;
                }
                write!(f, "]")
            },
        }
    }
}
