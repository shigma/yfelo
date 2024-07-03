use pest::error::InputLocation;

#[derive(Parser)]
#[grammar = "default/grammar.pest"]
pub struct DefaultParser;

pub fn make_span(loc: InputLocation) -> (usize, usize) {
    match loc {
        InputLocation::Pos(pos) => (pos, pos),
        InputLocation::Span((start, end)) => (start, end),
    }
}
