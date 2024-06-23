use std::collections::HashMap;

use crate::error::{Error, SyntaxError};
use pest::Parser;
use pest_meta::ast;
use serde_json::Value;

#[derive(Parser)]
#[grammar = "default.pest"]
struct DefaultParser;

pub trait Interpreter {
    type Expr;
    type Pattern;
    type Context;
    type Value;
    type Error;

    fn parse_expr(&self, input: &mut Input) -> Result<Self::Expr, SyntaxError>;
    fn parse_pattern(&self, input: &mut Input) -> Result<Self::Pattern, SyntaxError>;
    fn rules(&self) -> Vec<ast::Rule>;
    fn eval(&self, input: &str, ctx: &Self::Context) -> Result<Self::Value, Error<Self::Error>>;
    fn serialize(&self, value: &Self::Value) -> String;
}

macro_rules! expr {
    // expr!["name"] => ast::Expr::Str("name".to_string())
    ($a:literal $(,)?) => {
        ast::Expr::Ident($a.to_string())
    };
    // expr![value] => value
    ($a:expr $(,)?) => {
        $a
    };
    // expr![x, y] => ast::Expr::Seq(Box::new(x), Box::new(y))
    ($a:expr, $($b:expr),+ $(,)?) => {
        ast::Expr::Seq(Box::new(expr!($a)), Box::new(expr!($($b),+)))
    };
}

fn join(exprs: Vec<ast::Expr>, f: fn(Box<ast::Expr>, Box<ast::Expr>) -> ast::Expr) -> ast::Expr {
    let mut iter = exprs.into_iter().rev();
    let first = iter.next().unwrap();
    iter.fold(first, |acc, expr| f(Box::new(expr), Box::new(acc)))
}

pub struct Input<'i>(pub &'i str, pub usize);

impl<'i> Input<'i> {
    pub fn shift(&mut self, offset: usize) {
        let old_len = self.0.len();
        self.0 = self.0[offset..].trim_start();
        self.1 += old_len - self.0.len();
    }

    pub fn expect_word(&mut self, word: &str) -> Result<(), SyntaxError> {
        if self.0.starts_with(word) && !self.0[word.len()..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
            self.shift(word.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected keyword {}", word),
                range: (self.1, self.1 + 1),
            })
        }
    }
}

pub struct DefaultInterpreter;

pub enum Syntax {
    Bracket(String, String, String),
    Escape(String),
}

macro_rules! syntax {
    ($l:literal, $m:literal, $r:literal) => {
        Syntax::Bracket($l.into(), $r.into(), $m.into())
    };
}

impl Interpreter for DefaultInterpreter {
    type Expr = ();
    type Pattern = ();
    type Context = Value;
    type Value = Value;
    type Error = ();

    fn parse_expr(&self, input: &mut Input) -> Result<Self::Expr, SyntaxError> {
        let pairs = match DefaultParser::parse(Rule::expr, input.0) {
            Ok(v) => v,
            Err(e) => return Err(SyntaxError {
                message: e.to_string(),
                range: (0, 0), // TODO
            }),
        };
        input.shift(pairs.as_str().len());
        Ok(())
    }

    fn parse_pattern(&self, input: &mut Input) -> Result<Self::Pattern, SyntaxError> {
        let pairs = match DefaultParser::parse(Rule::ident, input.0) {
            Ok(v) => v,
            Err(e) => return Err(SyntaxError {
                message: e.to_string(),
                range: (0, 0), // TODO
            }),
        };
        input.shift(pairs.as_str().len());
        Ok(())
    }

    fn rules(&self) -> Vec<ast::Rule> {
        let mut grammar = HashMap::new();
        grammar.insert("main".to_string(), vec![
            syntax!("(", "main", ")"),
            syntax!("[", "main", "]"),
            syntax!("{", "main", "}"),
            syntax!("\"", "string", "\""),
        ]);
        grammar.insert("string".to_string(), vec![
            Syntax::Escape("\\".into()),
        ]);
        let mut rules = vec![ast::Rule {
            name: "ENTRY".to_string(),
            ty: ast::RuleType::Normal,
            expr: expr![
                "SOI",
                ast::Expr::Rep(Box::new(expr![
                    ast::Expr::NegPred(Box::new(expr!("EXIT"))),
                    "main",
                ])),
                ast::Expr::PosPred(Box::new(expr!("EXIT"))),
            ],
        }];
        for (name, syntax) in grammar {
            let mut neg = vec![];
            let count = syntax.len();
            for (index, syn) in syntax.into_iter().enumerate() {
                match syn {
                    Syntax::Bracket(left, right, inner) => {
                        neg.push(ast::Expr::Str(left.clone()));
                        neg.push(ast::Expr::Str(right.clone()));
                        rules.push(ast::Rule {
                            name: format!("{}_{}", name, index),
                            ty: ast::RuleType::Normal,
                            expr: join(vec![
                                ast::Expr::Str(left),
                                ast::Expr::Rep(Box::from(ast::Expr::Ident(inner.clone()))),
                                ast::Expr::Str(right),
                            ], ast::Expr::Seq),
                        });
                    },
                    Syntax::Escape(left) => {
                        neg.push(ast::Expr::Str(left.clone()));
                        rules.push(ast::Rule {
                            name: format!("{}_{}", name, index),
                            ty: ast::RuleType::Normal,
                            expr: join(vec![
                                ast::Expr::Str(left),
                                ast::Expr::Ident("ANY".to_string()),
                            ], ast::Expr::Seq),
                        });
                    },
                }
            }
            rules.push(ast::Rule {
                name: name.clone(),
                ty: ast::RuleType::Normal,
                expr: ast::Expr::Choice(
                    Box::new(expr![
                        ast::Expr::NegPred(Box::new(join(neg, ast::Expr::Choice))),
                        "ANY",
                    ]),
                    Box::new(join((0..count).map(|index| {
                        ast::Expr::Ident(format!("{}_{}", name, index))
                    }).collect(), ast::Expr::Choice)),
                ),
            });
        }
        rules
    }

    fn eval(&self, input: &str, _: &Value) -> Result<Value, Error<Self::Error>> {
        let _ = match DefaultParser::parse(Rule::expr, input) {
            Ok(v) => v,
            Err(e) => return Err(Error::Syntax(SyntaxError {
                message: e.to_string(),
                range: (0, 0), // TODO
            })),
        };
        Ok(Value::Null)
    }

    fn serialize(&self, value: &Self::Value) -> String {
        value.to_string()
    }
}
