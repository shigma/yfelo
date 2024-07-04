use std::marker::PhantomData;

use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::builtin::Stub;
use yfelo::default::{BinaryOp, Expr, Language, Pattern};
use yfelo::{Element, MetaSyntax, Node, SyntaxError, Yfelo};

const YFELO: Lazy<Yfelo> = Lazy::new(|| {
    let mut yfelo = Yfelo::new();
    yfelo.add_directive::<Stub>("foo");
    yfelo.add_directive::<Stub>("bar");
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo
});

const LANG: Lazy<Box<dyn yfelo::Language>> = Lazy::new(|| Box::new(PhantomData::<(Language, Expr, Pattern)>));

const META_SYNTAX: MetaSyntax = MetaSyntax {
    left: "{",
    right: "}",
};

macro_rules! ident {
    ($v:expr, $range:tt $(,)?) => {
        Expr::Ident($v.into(), Some($range))
    };
}

macro_rules! apply {
    ($lhs:expr, $rhs:expr, $range:tt $(,)?) => {
        Expr::Apply(Box::from($lhs), vec![$rhs], Some($range))
    };
}

macro_rules! binary {
    ($lhs:expr, $op:ident, $rhs:expr, $range:tt $(,)?) => {
        Expr::Binary(Box::from($lhs), BinaryOp::$op, Box::from($rhs), Some($range))
    };
}

#[test]
pub fn basic_1() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("(Hello) {world}!", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Text("(Hello) ".into()),
        Node::Expr(Box::from(Instance::new(ident!("world", (9, 14))))),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn basic_2() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{world}", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(ident!("world", (1, 6))))),
    ]);
}

#[test]
pub fn basic_3() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{w(or[ld])}", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(apply!(
            ident!("w", (1, 2)),
            binary!(ident!("or", (3, 5)), Index, ident!("ld", (6, 8)), (5, 9)),
            (2, 10),
        )))),
    ]);
}

#[test]
pub fn basic_4() {
    let (y, l) = (YFELO, LANG);
    let meta = MetaSyntax {
        left: "[",
        right: "]",
    };
    let nodes = y.parse("[w[or][ld]]!", l.as_ref(), &meta).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(binary!(
            binary!(ident!("w", (1, 2)), Index, ident!("or", (3, 5)), (2, 6)),
            Index,
            ident!("ld", (7, 9)),
            (6, 10),
        )))),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn invalid_tag_1() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{Hello} {world", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (14, 14),
    });
}

#[test]
pub fn invalid_tag_2() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{Hel(lo}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (4, 4),
    });
}

#[test]
pub fn invalid_tag_3() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{Hel)lo}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (4, 4),
    });
}

#[test]
pub fn invalid_tag_4() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{H(e[l)l]o}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (2, 2),
    });
}

#[test]
pub fn tag_1() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{#foo}Hello{/foo} {#bar}world{/bar}!", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::new(Instance::new(Stub)),
            children: vec![Node::Text("Hello".into())],
            branches: vec![],
        }),
        Node::Text(" ".into()),
        Node::Element(Element {
            directive: Box::new(Instance::new(Stub)),
            children: vec![Node::Text("world".into())],
            branches: vec![],
        }),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn tag_2() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{#foo}Hello{@bar} {#bar}world{/bar}!{/foo}", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::new(Instance::new(Stub)),
            children: vec![
                Node::Text("Hello".into()),
                Node::Element(Element {
                    directive: Box::new(Instance::new(Stub)),
                    children: vec![],
                    branches: vec![],
                }),
                Node::Text(" ".into()),
                Node::Element(Element {
                    directive: Box::new(Instance::new(Stub)),
                    children: vec![Node::Text("world".into())],
                    branches: vec![],
                }),
                Node::Text("!".into()),
            ],
            branches: vec![],
        }),
    ]);
}

#[test]
pub fn unmatched_tag_1() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#foo}Hello {#bar}world{/foo}!{/bar}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error.message, "unmatched tag name: expect 'bar', found 'foo'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_2() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#foo}Hello{/foo} world{/bar}!", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error.message, "unmatched tag name 'bar'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_3() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#foo}Hello{/foo} world{#bar}!", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error.message, "unmatched tag name 'bar'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn for_syntax_1() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect pattern".into(),
        range: (5, 5),
    });
}

#[test]
pub fn for_syntax_2() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for x}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect keyword 'in'".into(),
        range: (7, 7),
    });
}

#[test]
pub fn for_syntax_3() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{@for}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "directive 'for' should not be empty".into(),
        range: (2, 5),
    });
}
