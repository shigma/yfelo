use std::fmt::Debug;

use pest::{iterators::{Pair, Pairs}, Parser};
use yfelo_core::{factory, SyntaxError};

use super::operator::{BinaryOp, UnaryOp};
use super::parser::{DefaultParser, Rule};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64),
    String(String),
    Ident(String),
    Array(Vec<Expr>),
    Apply(Box<Expr>, Vec<Expr>),
    Unary(UnaryOp, Box<Expr>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>),
}

impl factory::Expr for Expr {}

macro_rules! left_assoc {
    ($curr:ident, $inner:ident) => {
        fn $curr(pair: Pair<Rule>) -> Self {
            let mut pairs = pair.into_inner();
            let mut expr = Expr::$inner(pairs.next().unwrap());
            while pairs.len() > 0 {
                let op = BinaryOp::from(pairs.next().unwrap());
                let rhs = Expr::$inner(pairs.next().unwrap());
                expr = Expr::Binary(Box::new(expr), op, Box::new(rhs));
            }
            expr
        }
    };
}

impl Expr {
    pub fn parse(input: &str) -> Result<(Expr, usize), SyntaxError> {
        match DefaultParser::parse(Rule::expr, input) {
            Ok(pairs) => {
                let len = pairs.as_str().len();
                Ok((Expr::from(pairs.into_iter().next().unwrap()), len))
            },
            Err(e) => Err(SyntaxError {
                message: e.to_string(),
                range: (0, 0), // TODO
            }),
        }
    }

    pub fn from(pair: Pair<Rule>) -> Self {
        assert!(matches!(pair.as_rule(), Rule::expr));
        Expr::from_or(pair.into_inner().next().unwrap())
    }

    fn from_list(pairs: Pairs<Rule>) -> Vec<Self> {
        let mut exprs = vec![];
        for pair in pairs {
            match pair.as_rule() {
                Rule::expr => {
                    exprs.push(Expr::from(pair));
                },
                _ => unreachable!(),
            }
        }
        exprs
    }

    fn from_suffix(self, pair: Pair<Rule>) -> Self {
        assert!(matches!(pair.as_rule(), Rule::suffix));
        match pair.as_str().chars().nth(0) {
            Some('(') => {
                let pairs = pair.into_inner();
                Expr::Apply(Box::new(self), Expr::from_list(pairs))
            },
            Some('[') => {
                let pair = pair.into_inner().into_iter().next().unwrap();
                Expr::Binary(Box::new(self), BinaryOp::Index, Box::from(Expr::from(pair)))
            },
            Some('.') => {
                let pair = pair.into_inner().into_iter().next().unwrap();
                let expr = Expr::String(pair.as_str().to_string());
                Expr::Binary(Box::new(self), BinaryOp::Index, Box::from(expr))
            },
            _ => unreachable!(),
        }
    }

    fn from_atom(pair: Pair<Rule>) -> Self {
        assert!(matches!(pair.as_rule(), Rule::atom));
        let pair = pair.into_inner().next().unwrap();
        match pair.as_rule() {
            Rule::number => Expr::Number(pair.as_str().parse().unwrap()),
            Rule::string => {
                let str = pair.as_str();
                let mut str = &str[1..str.len() - 1];
                let mut inner = String::new();
                while let Some(i) = str.find('\\') {
                    inner += &str[..i];
                    inner.push(match str.chars().nth(i + 1).unwrap() {
                        'n' => '\n',
                        'r' => '\r',
                        't' => '\t',
                        c => c,
                    });
                    str = &str[i + 2..];
                }
                inner += str;
                Expr::String(inner)
            },
            Rule::ident => Expr::Ident(pair.as_str().to_string()),
            Rule::array => {
                let pairs = pair.into_inner();
                Expr::Array(Expr::from_list(pairs))
            },
            Rule::expr => Expr::from(pair),
            _ => unreachable!("unexpected rule: {:?}", pair.as_rule()),
        }
    }

    fn from_unary(pair: Pair<Rule>) -> Self {
        let pairs = pair.into_inner().collect::<Vec<_>>();
        let index = pairs.iter().position(|pair| pair.as_rule() == Rule::atom).unwrap();
        let mut expr = Expr::from_atom(pairs[index].clone());
        for i in index + 1..pairs.len() {
            expr = Expr::from_suffix(expr, pairs[i].clone());
        }
        for i in (0..index).rev() {
            let op = UnaryOp::from(pairs[i].clone());
            expr = Expr::Unary(op, Box::new(expr));
        }
        expr
    }

    fn from_pow(pair: Pair<Rule>) -> Self {
        let mut pairs = pair.into_inner().rev();
        let mut expr = Expr::from_unary(pairs.next().unwrap());
        while pairs.len() > 0 {
            let op = BinaryOp::from(pairs.next().unwrap());
            let lhs = Expr::from_unary(pairs.next().unwrap());
            expr = Expr::Binary(Box::new(lhs), op, Box::new(expr));
        }
        expr
    }

    left_assoc!(from_mul, from_pow);
    left_assoc!(from_add, from_mul);
    left_assoc!(from_shift, from_add);
    left_assoc!(from_comp, from_shift);
    left_assoc!(from_eq, from_comp);
    left_assoc!(from_bitand, from_eq);
    left_assoc!(from_bitxor, from_bitand);
    left_assoc!(from_bitor, from_bitxor);
    left_assoc!(from_and, from_bitor);
    left_assoc!(from_or, from_and);
}
