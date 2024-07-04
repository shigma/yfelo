use pest::error::InputLocation;
use pest::iterators::Pair;

#[derive(Parser)]
#[grammar = "default/grammar.pest"]
pub struct DefaultParser;

pub trait ToRange {
    fn to_range(&self, offset: usize) -> (usize, usize);
}

impl ToRange for Pair<'_, Rule> {
    fn to_range(&self, offset: usize) -> (usize, usize) {
        (self.as_span().start() + offset, self.as_span().end() + offset)
    }
}

impl ToRange for InputLocation {
    fn to_range(&self, offset: usize) -> (usize, usize) {
        match self {
            InputLocation::Pos(pos) => (*pos + offset, *pos + offset),
            InputLocation::Span((start, end)) => (*start + offset, *end + offset),
        }
    }
}
