use std::collections::HashMap;

use pest_meta::ast::{Expr, Rule, RuleType};

use crate::Interpreter;

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
    fn rules(&self) -> Vec<Rule> {
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
                                Expr::Rep(Box::from(Expr::Ident(inner.clone()))),
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
        rules
    }
}
