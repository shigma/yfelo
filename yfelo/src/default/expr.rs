use std::fmt::Debug;

use pest::{iterators::{Pair, Pairs}, Parser};
use yfelo_core::{factory, SyntaxError};

use super::operator::{BinaryOp, UnaryOp};
use super::parser::{DefaultParser, Rule, ToRange};

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Number(f64, Option<(usize, usize)>),
    String(String, Option<(usize, usize)>),
    Ident(String, Option<(usize, usize)>),
    Array(Vec<Expr>, Option<(usize, usize)>),
    Object(Vec<(Expr, Option<Expr>)>, Option<(usize, usize)>),
    Apply(Box<Expr>, Vec<Expr>, Option<(usize, usize)>),
    Unary(UnaryOp, Box<Expr>, Option<(usize, usize)>),
    Binary(Box<Expr>, BinaryOp, Box<Expr>, Option<(usize, usize)>),
    Index(Box<Expr>, Box<Expr>, bool, Option<(usize, usize)>),
}

impl factory::Expr for Expr {}

macro_rules! left_assoc {
    ($curr:ident, $inner:ident) => {
        fn $curr(pair: Pair<Rule>, offset: usize) -> Self {
            let mut pairs = pair.into_inner();
            let mut expr = Expr::$inner(pairs.next().unwrap(), offset);
            while pairs.len() > 0 {
                let pair = pairs.next().unwrap();
                let op = BinaryOp::from(pair.as_str());
                let rhs = Expr::$inner(pairs.next().unwrap(), offset);
                expr = Expr::Binary(Box::new(expr), op, Box::new(rhs), Some(pair.to_range(offset)));
            }
            expr
        }
    };
}

impl Expr {
    pub fn parse(input: &str, offset: usize) -> Result<(Expr, usize), SyntaxError> {
        match DefaultParser::parse(Rule::expr, input) {
            Ok(pairs) => {
                let len = pairs.as_str().len();
                Ok((Expr::from(pairs.into_iter().next().unwrap(), offset), len))
            },
            Err(e) => Err(SyntaxError {
                message: e.to_string(),
                range: e.location.to_range(offset),
            }),
        }
    }

    fn from(pair: Pair<Rule>, offset: usize) -> Self {
        assert!(matches!(pair.as_rule(), Rule::expr));
        Expr::from_or(pair.into_inner().next().unwrap(), offset)
    }

    fn from_list(pairs: Pairs<Rule>, offset: usize) -> Vec<Self> {
        let mut exprs = vec![];
        for pair in pairs {
            exprs.push(Self::from(pair, offset));
        }
        exprs
    }

    fn from_entry(pair: Pair<Rule>, offset: usize) -> (Self, Option<Self>) {
        assert!(matches!(pair.as_rule(), Rule::entry));
        let mut pairs = pair.into_inner();
        let pair = pairs.next().unwrap();
        let key = Self::parse_literal(pair, offset);
        let value = pairs.next().map(|p| Self::from(p, offset));
        (key, value)
    }

    fn from_entries(pairs: Pairs<Rule>, offset: usize) -> Vec<(Self, Option<Self>)> {
        let mut entries = vec![];
        for pair in pairs {
            entries.push(Self::from_entry(pair, offset));
        }
        entries
    }

    fn from_suffix(self, pair: Pair<Rule>, offset: usize) -> Self {
        assert!(matches!(pair.as_rule(), Rule::suffix));
        let range = Some(pair.to_range(offset));
        match pair.as_str().chars().nth(0) {
            Some('(') => {
                let pairs = pair.into_inner();
                Expr::Apply(Box::new(self), Expr::from_list(pairs, offset), range)
            },
            Some('[') => {
                let pair = pair.into_inner().into_iter().next().unwrap();
                Expr::Index(Box::new(self), Box::from(Expr::from(pair, offset)), true, range)
            },
            Some('.') => {
                let pair = pair.into_inner().into_iter().next().unwrap();
                let expr = Expr::String(pair.as_str().to_string(), Some(pair.to_range(offset)));
                Expr::Index(Box::new(self), Box::from(expr), false, range)
            },
            _ => unreachable!(),
        }
    }

    fn parse_literal(pair: Pair<Rule>, offset: usize) -> Self {
        let range = Some(pair.to_range(offset));
        match pair.as_rule() {
            Rule::ident => Expr::Ident(pair.as_str().to_string(), range),
            Rule::number => Expr::Number(pair.as_str().parse().unwrap(), range),
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
                Expr::String(inner, range)
            },
            Rule::array => {
                let pairs = pair.into_inner();
                Expr::Array(Expr::from_list(pairs, offset), range)
            },
            Rule::object => {
                let pairs = pair.into_inner();
                Expr::Object(Expr::from_entries(pairs, offset), range)
            },
            Rule::expr => Expr::from(pair, offset),
            _ => unreachable!("unexpected rule: {:?}", pair.as_rule()),
        }
    }

    fn from_unary(pair: Pair<Rule>, offset: usize) -> Self {
        let pairs = pair.into_inner().collect::<Vec<_>>();
        let index = pairs.iter().position(|pair| pair.as_rule() == Rule::literal).unwrap();
        let pair = pairs[index].clone().into_inner().next().unwrap();
        let mut expr = Expr::parse_literal(pair, offset);
        for i in index + 1..pairs.len() {
            expr = Expr::from_suffix(expr, pairs[i].clone(), offset);
        }
        for i in (0..index).rev() {
            let op = UnaryOp::from(pairs[i].as_str());
            expr = Expr::Unary(op, Box::new(expr), Some(pairs[i].to_range(offset)));
        }
        expr
    }

    fn from_pow(pair: Pair<Rule>, offset: usize) -> Self {
        let mut pairs = pair.into_inner().rev();
        let mut expr = Expr::from_unary(pairs.next().unwrap(), offset);
        while pairs.len() > 0 {
            let pair = pairs.next().unwrap();
            let op = BinaryOp::from(pair.as_str());
            let lhs = Expr::from_unary(pairs.next().unwrap(), offset);
            expr = Expr::Binary(Box::new(lhs), op, Box::new(expr), Some(pair.to_range(offset)));
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
