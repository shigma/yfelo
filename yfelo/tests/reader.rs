use once_cell::sync::Lazy;
use yfelo::builtin::Stub;
use yfelo::default::{BinaryOp, Expr, Language};
use yfelo::{Element, MetaSyntax, Node, Yfelo};

const YFELO: Lazy<Yfelo> = Lazy::new(|| {
    let mut yfelo = Yfelo::new();
    yfelo.add_directive::<Stub>("foo");
    yfelo.add_directive::<Stub>("bar");
    yfelo.add_language("default", Box::new(Language));
    yfelo
});

const LANG: Lazy<Box<dyn yfelo::Language>> = Lazy::new(|| Box::new(Language));

const META_SYNTAX: MetaSyntax = MetaSyntax {
    left: "{",
    right: "}",
};

macro_rules! ident {
    ($v:expr $(,)?) => {
        Expr::Ident($v.into())
    };
}

macro_rules! apply {
    ($lhs:expr, $rhs:expr $(,)?) => {
        Expr::Apply(Box::from($lhs), vec![$rhs])
    };
}

macro_rules! binary {
    ($lhs:expr, $op:ident, $rhs:expr $(,)?) => {
        Expr::Binary(Box::from($lhs), BinaryOp::$op, Box::from($rhs))
    };
}

#[test]
pub fn basic_1() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("(Hello) {world}!", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Text("(Hello) "),
        Node::Expr(Box::from(ident!("world"))),
        Node::Text("!"),
    ]);
}

#[test]
pub fn basic_2() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{world}", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(ident!("world"))),
    ]);
}

#[test]
pub fn basic_3() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{w(or[ld])}", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(apply!(
            ident!("w"),
            binary!(ident!("or"), Index, ident!("ld")),
        ))),
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
        Node::Expr(Box::from(binary!(
            binary!(ident!("w"), Index, ident!("or")),
            Index,
            ident!("ld"),
        ))),
        Node::Text("!"),
    ]);
}

#[test]
pub fn invalid_tag_1() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{Hello} {world", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (14, 15));
}

#[test]
pub fn invalid_tag_2() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{Hel(lo}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (4, 5));
}

#[test]
pub fn invalid_tag_3() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{Hel)lo}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (4, 5));
}

#[test]
pub fn invalid_tag_4() {
    let (y, l) = (YFELO, LANG);
    let err = y.parse("{H(e[l)l]o}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (2, 3));
}

#[test]
pub fn tag_1() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{#foo}Hello{/foo} {#bar}world{/bar}!", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::from(Stub),
            children: vec![Node::Text("Hello")],
        }),
        Node::Text(" "),
        Node::Element(Element {
            directive: Box::from(Stub),
            children: vec![Node::Text("world")],
        }),
        Node::Text("!"),
    ]);
}

#[test]
pub fn tag_2() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{#foo}Hello{@bar} {#bar}world{/bar}!{/foo}", l.as_ref(), &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::from(Stub),
            children: vec![
                Node::Text("Hello"),
                Node::Element(Element {
                    directive: Box::from(Stub),
                    children: vec![],
                }),
                Node::Text(" "),
                Node::Element(Element {
                    directive: Box::from(Stub),
                    children: vec![Node::Text("world")],
                }),
                Node::Text("!"),
            ],
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
