use std::collections::HashMap;
use std::rc::Rc;

use pest::error::InputLocation;
use pest_meta::ast::{Expr, Rule, RuleType};
use pest_meta::optimizer::optimize;
use pest_vm::Vm;

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

#[derive(Debug, Clone, PartialEq)]
pub struct ParseError {
    pub message: String,
    pub range: (usize, usize),
}

pub enum Syntax {
    Bracket(String, String, String),
    Escape(String),
}

macro_rules! expr {
    // expr!["name"] => Expr::Str("name".to_string())
    ($a:literal $(,)?) => {
        Expr::Ident($a.to_string())
    };
    // expr![value] => value
    ($a:expr $(,)?) => {
        $a
    };
    // expr![x, y] => Expr::Seq(Box::new(x), Box::new(y))
    ($a:expr, $($b:expr),+ $(,)?) => {
        Expr::Seq(Box::new(expr!($a)), Box::new(expr!($($b),+)))
    };
}

fn join(exprs: Vec<Expr>, f: fn(Box<Expr>, Box<Expr>) -> Expr) -> Expr {
    let mut iter = exprs.into_iter().rev();
    let first = iter.next().unwrap();
    iter.fold(first, |acc, expr| f(Box::new(expr), Box::new(acc)))
}

pub struct Interpreter {
    rules: Vec<Rule>,
}

impl Interpreter {
    pub fn new(grammar: HashMap<String, Vec<Syntax>>) -> Self {
        let mut rules = vec![Rule {
            name: "ENTRY".to_string(),
            ty: RuleType::Normal,
            expr: expr![
                "SOI",
                Expr::Rep(Box::new(expr![
                    Expr::NegPred(Box::new(expr!("EXIT"))),
                    "main",
                ])),
                Expr::PosPred(Box::new(expr!("EXIT"))),
            ],
        }];
        for (name, syntax) in grammar {
            let mut neg = vec![];
            let count = syntax.len();
            for (index, syn) in syntax.into_iter().enumerate() {
                match syn {
                    Syntax::Bracket(left, right, inner) => {
                        neg.push(Expr::Str(left.clone()));
                        neg.push(Expr::Str(right.clone()));
                        rules.push(Rule {
                            name: format!("{}_{}", name, index),
                            ty: RuleType::Normal,
                            expr: join(vec![
                                Expr::Str(left),
                                Expr::Ident(inner.clone()),
                                Expr::Str(right),
                            ], Expr::Seq),
                        });
                    },
                    Syntax::Escape(left) => {
                        neg.push(Expr::Str(left.clone()));
                        rules.push(Rule {
                            name: format!("{}_{}", name, index),
                            ty: RuleType::Normal,
                            expr: join(vec![
                                Expr::Str(left),
                                Expr::Ident("ANY".to_string()),
                            ], Expr::Seq),
                        });
                    },
                }
            }
            rules.push(Rule {
                name: name.clone(),
                ty: RuleType::Normal,
                expr: Expr::Choice(
                    Box::new(expr![
                        Expr::NegPred(Box::new(join(neg, Expr::Choice))),
                        "ANY",
                    ]),
                    Box::new(join((0..count).map(|index| {
                        Expr::Ident(format!("{}_{}", name, index))
                    }).collect(), Expr::Choice)),
                ),
            });
        }
        Self {
            rules,
        }
    }
}

pub struct Yfelo {
    left: String,
    #[allow(dead_code)]
    right: String,
    parser: Vm,
    #[allow(dead_code)]
    interpreter: Rc<Interpreter>,
}

impl Yfelo {
    pub fn new(left: String, right: String, interpreter: Rc<Interpreter>) -> Self {
        let mut rules = interpreter.rules.clone();
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

    pub fn tokenize<'i>(&'i self, mut input: &'i str) -> Result<Vec<Token<'i>>, ParseError> {
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
                    println!("Error: {}", err);
                    return Err(ParseError {
                        message: format!("unterminated tag syntax"),
                        range: (offset - 1, offset),
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

    pub fn parse<'i>(&'i self, input: &'i str) -> Result<Vec<Node<'i>>, ParseError> {
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
}
