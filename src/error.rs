#[derive(Debug, Clone, PartialEq)]
pub struct SyntaxError {
    pub message: String,
    pub range: (usize, usize),
}

#[derive(Debug, Clone)]
pub enum Error<T> {
    Syntax(SyntaxError),
    Runtime(T),
}
