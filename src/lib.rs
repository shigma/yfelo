#[macro_use]
extern crate pest_derive;

use pest::error::InputLocation;
use pest_meta::ast::{Expr, Rule, RuleType};
use pest_meta::optimizer::optimize;
use pest_vm::Vm;

pub use crate::directive::Directive;
pub use crate::error::{Error, SyntaxError};
pub use crate::interpreter::Interpreter;

pub mod directive;
pub mod error;
pub mod interpreter;

#[derive(Debug, Clone, PartialEq)]
pub struct Element<'i> {
    pub name: &'i str,
    pub header: &'i str,
    pub footer: &'i str,
    pub children: Vec<Node<'i>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Expression<'i> {
    pub content: &'i str,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Tag<'i> {
    pub name: &'i str,
    pub header: &'i str,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(&'i str),
    Element(Element<'i>),
    Branch(Tag<'i>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token<'i> {
    Text(&'i str),
    Tag(&'i str, (usize, usize)),
}

pub struct Yfelo<'i, E, P, C, V, R> {
    left: String,
    #[allow(dead_code)]
    right: String,
    parser: Vm,
    interpreter: &'i dyn Interpreter<Expr = E, Pattern = P, Context = C, Value = V, Error = R>,
}

impl<'i, E, P, C, V, R> Yfelo<'i, E, P, C, V, R> {
    pub fn new(left: String, right: String, interpreter: &'i dyn Interpreter<Expr = E, Pattern = P, Context = C, Value = V, Error = R>) -> Self {
        let mut rules = interpreter.rules().clone();
        rules.push(Rule {
            name: "EXIT".to_string(),
            ty: RuleType::Silent,
            expr: Expr::Str(right.clone()),
        });
        let parser = Vm::new(optimize(rules));
        Self {
            left,
            right,
            parser,
            interpreter,
        }
    }

    pub fn tokenize(&'i self, mut input: &'i str) -> Result<Vec<Token<'i>>, SyntaxError> {
        let mut nodes = vec![];
        let mut offset = 0;
        while let Some(pos) = input.find(&self.left) {
            if pos > 0 {
                nodes.push(Token::Text(&input[..pos]));
            }
            input = &input[pos + 1..];
            offset += pos + 1;
            match self.parser.parse("ENTRY", input) {
                Ok(pairs) => {
                    let tag = pairs.as_str();
                    let end = tag.len();
                    nodes.push(Token::Tag(tag, (offset, offset + end)));
                    input = &input[end + 1..];
                    offset += end + 1;
                },
                Err(err) => {
                    println!("Error: {:?}", err.variant);
                    println!("Error: {:?}", err.location);
                    println!("Error: {:?}", err.line_col);
                    return Err(SyntaxError {
                        message: format!("unterminated tag syntax"),
                        range: match err.location {
                            // TODO
                            InputLocation::Pos(pos) => {
                                if pos == input.len() {
                                    (offset - 1, offset)
                                } else {
                                    // unmatched closing bracket
                                    (offset + pos, offset + pos + 1)
                                }
                            },
                            _ => (offset - 1, offset),
                        },
                    });
                },
            }
        }
        if input.len() > 0 {
            nodes.push(Token::Text(input));
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

    pub fn parse(&'i self, input: &'i str) -> Result<Vec<Node<'i>>, SyntaxError> {
        let tokens = self.tokenize(input)?;
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
                        let (name, header) = Self::split(&content[1..]);
                        if name.len() == 0 {
                            return Err(SyntaxError {
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
                                    return Err(SyntaxError {
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
            return Err(SyntaxError {
                message: format!("unmatched tag name"),
                range: stack.last().unwrap().1,
            });
        }
        Ok(stack.pop().unwrap().0.children)
    }

    pub fn transform(&'i self, input: &'i str, ctx: &'i C) -> Result<String, Error<R>> {
        let nodes = self.parse(input).map_err(|e| Error::Syntax(e))?;
        self.transform_nodes(nodes, ctx)
    }

    fn transform_nodes(&'i self, nodes: Vec<Node<'i>>, ctx: &'i C) -> Result<String, Error<R>> {
        let mut output = String::new();
        for node in nodes {
            match node {
                Node::Text(text) => output += text,
                Node::Expr(expr) => {
                    let value = self.interpreter.eval(expr, ctx)?;
                    output += &self.interpreter.serialize(&value);
                },
                _ => unimplemented!(),
            }
        }
        Ok(output)
    }
}
