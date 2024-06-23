#[macro_use]
extern crate pest_derive;

use std::any::Any;
use std::collections::HashMap;

use parser::Input;
use pest_meta::ast::{Expr, Rule, RuleType};
use pest_meta::optimizer::optimize;
use pest_vm::Vm;

pub use crate::directive::Directive;
pub use crate::error::{Error, SyntaxError};
pub use crate::interpreter::Interpreter;

pub mod directive;
pub mod error;
pub mod interpreter;
pub mod parser;

#[derive(Debug)]
pub struct Element<'i> {
    pub name: &'i str,
    pub meta: Box<dyn Any>,
    pub children: Option<Vec<Node<'i>>>,
}

#[derive(Debug)]
pub struct Expression<'i> {
    pub content: &'i str,
}

#[derive(Debug)]
pub struct Tag<'i> {
    pub name: &'i str,
    pub header: &'i str,
}

#[derive(Debug)]
pub enum Node<'i> {
    Text(&'i str),
    Expr(&'i str),
    Element(Element<'i>),
    Branch(Tag<'i>),
}

#[derive(Debug)]
pub enum Token<'i> {
    Text(&'i str),
    Tag(&'i str, (usize, usize)),
}

pub struct Yfelo<'i> {
    left: String,
    right: String,
    parser: Vm,
    directives: HashMap<String, &'i dyn Directive>,
    interpreter: &'i dyn Interpreter,
}

impl<'i> Yfelo<'i> {
    pub fn new(left: String, right: String, interpreter: &'i dyn Interpreter) -> Self {
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
            directives: HashMap::new(),
            interpreter,
        }
    }

    pub fn parse(&'i self, source: &'i str) -> Result<Vec<Node<'i>>, SyntaxError> {
        let mut offset = 0;
        let mut stack = vec![(Element {
            name: "",
            meta: Box::new(()),
            children: Some(vec![]),
        }, (0, 0))];
        let mut input = Input { source, offset: 0, close: self.right.clone(), lang: self.interpreter };
        while let Some(pos) = input.source.find(&self.left) {
            if pos > 0 {
                stack.last_mut().unwrap().0.children.unwrap().push(Node::Text(&input.source[..pos]));
            }
            input.source = &input.source[pos + 1..];
            input.offset += pos + 1;
            if let Some((c, name)) = input.expect_directive() {
                let Some(directive) = self.directives.get(name) else {
                    return Err(SyntaxError {
                        message: format!("unknown directive '{}'", name),
                        range: (input.offset - name.len(), input.offset),
                    });
                };
                match c {
                    '#' => {
                        let meta = directive.parse(&mut input)?;
                        stack.push((Element {
                            name,
                            meta,
                            children: Some(vec![]),
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
                        stack.last_mut().unwrap().0.children.unwrap().push(Node::Element(element));
                    },
                    '@' => {
                        let meta = directive.parse(&mut input)?;
                        stack.last_mut().unwrap().0.children.unwrap().push(Node::Element(Element {
                            name,
                            meta,
                            children: None,
                        }));
                    },
                    ':' => {
                        stack.last_mut().unwrap().0.children.unwrap().push(Node::Branch(Tag {
                            name,
                            header,
                        }));
                    },
                    _ => unreachable!(),
                }
            } else {
                stack.last_mut().unwrap().0.children.unwrap().push(Node::Expr(content.trim()));
            }
        }
        if input.source.len() > 0 {
            stack.last_mut().unwrap().0.children.unwrap().push(Node::Text(input.source));
        }
        if stack.len() > 1 {
            return Err(SyntaxError {
                message: format!("unmatched tag name"),
                range: stack.last().unwrap().1,
            });
        }
        Ok(stack.pop().unwrap().0.children)
    }

    // pub fn transform(&'i self, input: &'i str, ctx: &'i C) -> Result<String, Error<R>> {
    //     let nodes = self.parse(input).map_err(|e| Error::Syntax(e))?;
    //     self.transform_nodes(nodes, ctx)
    // }

    // fn transform_nodes(&'i self, nodes: Vec<Node<'i>>, ctx: &'i C) -> Result<String, Error<R>> {
    //     let mut output = String::new();
    //     for node in nodes {
    //         match node {
    //             Node::Text(text) => output += text,
    //             Node::Expr(expr) => {
    //                 let value = self.interpreter.eval(expr, ctx)?;
    //                 output += &self.interpreter.serialize(&value);
    //             },
    //             _ => unimplemented!(),
    //         }
    //     }
    //     Ok(output)
    // }
}
