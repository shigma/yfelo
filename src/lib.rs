
#[derive(Debug, Clone, PartialEq)]
pub struct Element<T> {
    pub name: T,
    pub header: T,
    pub footer: T,
    pub children: Vec<Node<T>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression<T> {
    pub content: T,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag<T> {
    pub name: T,
    pub header: T,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<T> {
    Text(T),
    Expr(T),
    Element(Element<T>),
    Branch(Tag<T>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<T> {
    Text(T),
    Tag(T, (usize, usize)),
}

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub range: (usize, usize),
}

pub fn tokenize(mut src: &str) -> Result<Vec<Token<&str>>, ParseError> {
    let mut nodes = vec![];
    let mut position = 0;
    while let Some(pos) = src.find("{") {
        if pos > 0 {
            nodes.push(Token::Text(&src[..pos]));
        }
        src = &src[pos + 1..];
        position += pos + 1;
        if let Some(end) = src.find("}") {
            let tag = &src[..end];
            nodes.push(Token::Tag(tag, (position, position + end)));
            src = &src[end + 1..];
            position += end + 1;
        } else {
            return Err(ParseError {
                message: format!("unterminated tag syntax"),
                range: (position - 1, position),
            });
        }
    }
    if src.len() > 0 {
        nodes.push(Token::Text(src));
    }
    Ok(nodes)
}

fn split(content: &str) -> (&str, &str) {
    if let Some(pos) = content.find(char::is_whitespace) {
        (&content[..pos], &content[pos + 1..].trim())
    } else {
        (content, "")
    }
}

pub fn parse(src: &str) -> Result<Vec<Node<&str>>, ParseError> {
    let tokens = tokenize(src)?;
    let mut stack = vec![(Element {
        name: "",
        header: "",
        footer: "",
        children: vec![],
    }, (0, 0))];
    for token in tokens {
        match token {
            Token::Text(text) => {
                stack.last_mut().unwrap().0.children.push(Node::Text(text))
            },
            Token::Tag(content, range) => {
                if let Some(c @ ('#' | '/' | ':' | '@')) = content.chars().nth(0) {
                    let (name, header) = split(&content[1..]);
                    if name.len() == 0 {
                        return Err(ParseError {
                            message: format!("empty tag name"),
                            range,
                        });
                    }
                    match c {
                        '#' => {
                            stack.push((Element {
                                name,
                                header,
                                footer: "",
                                children: vec![],
                            }, range));
                        },
                        '/' => {
                            let element = stack.pop().unwrap().0;
                            if element.name != name {
                                return Err(ParseError {
                                    message: format!("unmatched tag name"),
                                    range,
                                });
                            }
                            stack.last_mut().unwrap().0.children.push(Node::Element(element));
                        },
                        '@' => {
                            stack.last_mut().unwrap().0.children.push(Node::Element(Element {
                                name,
                                header,
                                footer: "",
                                children: vec![],
                            }));
                        },
                        ':' => {
                            stack.last_mut().unwrap().0.children.push(Node::Branch(Tag {
                                name,
                                header,
                            }));
                        },
                        _ => unreachable!(),
                    }
                } else {
                    stack.last_mut().unwrap().0.children.push(Node::Expr(content.trim()));
                }
            },
        }
    }
    if stack.len() > 1 {
        return Err(ParseError {
            message: format!("unmatched tag name"),
            range: stack.last().unwrap().1,
        });
    }
    Ok(stack.pop().unwrap().0.children)
}
