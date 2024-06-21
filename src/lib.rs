
#[derive(Debug, Clone)]
pub struct Element<T> {
    pub name: T,
    pub header: T,
    pub footer: T,
    pub body: Vec<Node<T>>,
}

#[derive(Debug, Clone)]
pub struct Tag<T> {
    pub name: T,
    pub header: T,
}

#[derive(Debug, Clone)]
pub enum Node<T> {
    Text(T),
    Element(Element<T>),
    Branch(Tag<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<T> {
    Text(T),
    Tag(T),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

pub fn tokenize(mut src: &str) -> Result<Vec<Token<&str>>, ParseError> {
    let mut nodes = vec![];
    let mut position = 0;
    while let Some(pos) = src.find("{") {
        if pos > 0 {
            nodes.push(Token::Text(&src[..pos]));
        }
        src = &src[pos + 1..];
        position += pos;
        if let Some(end) = src.find("}") {
            let tag = &src[..end];
            nodes.push(Token::Tag(tag));
            src = &src[end + 1..];
            position += end + 2;
        } else {
            return Err(ParseError {
                message: format!("unmatched tag syntax"),
                position,
            });
        }
    }
    if src.len() > 0 {
        nodes.push(Token::Text(src));
    }
    Ok(nodes)
}

// text
// {#name header}
//   text
//   {@name header}
// {/name footer}

pub fn parse() {}
