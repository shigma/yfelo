use once_cell::sync::Lazy;
use yfelo::directive::{Stub, StubMeta};
use yfelo::{Element, MetaSyntax, Node, Yfelo};
use yfelo::language::default::{BinaryOp, Expr, Language};

const YFELO: Lazy<Yfelo> = Lazy::new(|| {
    let mut yfelo = Yfelo::new();
    yfelo.add_directive("foo", Box::new(Stub));
    yfelo.add_directive("bar", Box::new(Stub));
    yfelo.add_language("default", Box::new(Language));
    yfelo
});

const LANG_NAME: &str = "default";

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
    let y = YFELO;
    let nodes = y.parse("(Hello) {world}!", LANG_NAME, &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Text("(Hello) "),
        Node::Expr(Box::from(ident!("world"))),
        Node::Text("!"),
    ]);
}

#[test]
pub fn basic_2() {
    let y = YFELO;
    let nodes = y.parse("{world}", LANG_NAME, &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(ident!("world"))),
    ]);
}

#[test]
pub fn basic_3() {
    let y = YFELO;
    let nodes = y.parse("{w(or[ld])}", LANG_NAME, &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(apply!(
            ident!("w"),
            binary!(ident!("or"), Index, ident!("ld")),
        ))),
    ]);
}

#[test]
pub fn basic_4() {
    let y = YFELO;
    let meta = MetaSyntax {
        left: "[",
        right: "]",
    };
    let nodes = y.parse("[w[or][ld]]!", LANG_NAME, &meta).unwrap();
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
    let y = YFELO;
    let err = y.parse("{Hello} {world", LANG_NAME, &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (14, 15));
}

#[test]
pub fn invalid_tag_2() {
    let y = YFELO;
    let err = y.parse("{Hel(lo}", LANG_NAME, &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (4, 5));
}

#[test]
pub fn invalid_tag_3() {
    let y = YFELO;
    let err = y.parse("{Hel)lo}", LANG_NAME, &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (4, 5));
}

#[test]
pub fn invalid_tag_4() {
    let y = YFELO;
    let err = y.parse("{H(e[l)l]o}", LANG_NAME, &META_SYNTAX).unwrap_err();
    assert_eq!(err.message, "invalid tag syntax");
    assert_eq!(err.range, (2, 3));
}

#[test]
pub fn tag_1() {
    let y = YFELO;
    let nodes = y.parse("{#foo}Hello{/foo} {#bar}world{/bar}!", LANG_NAME, &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            name: "foo",
            meta: Box::from(StubMeta),
            children: Some(vec![Node::Text("Hello")]),
        }),
        Node::Text(" "),
        Node::Element(Element {
            name: "bar",
            meta: Box::from(StubMeta),
            children: Some(vec![Node::Text("world")]),
        }),
        Node::Text("!"),
    ]);
}

#[test]
pub fn tag_2() {
    let y = YFELO;
    let nodes = y.parse("{#foo}Hello{@bar} {#bar}world{/bar}!{/foo}", LANG_NAME, &META_SYNTAX).unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            name: "foo",
            meta: Box::from(StubMeta),
            children: Some(vec![
                Node::Text("Hello"),
                Node::Element(Element {
                    name: "bar",
                    meta: Box::from(StubMeta),
                    children: None,
                }),
                Node::Text(" "),
                Node::Element(Element {
                    name: "bar",
                    meta: Box::from(StubMeta),
                    children: Some(vec![Node::Text("world")]),
                }),
                Node::Text("!"),
            ]),
        }),
    ]);
}

#[test]
pub fn unmatched_tag_1() {
    let y = YFELO;
    let error = y.parse("{#foo}Hello {#bar}world{/foo}!{/bar}", LANG_NAME, &META_SYNTAX).unwrap_err();
    assert_eq!(error.message, "unmatched tag name");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_2() {
    let y = YFELO;
    let error = y.parse("{#foo}Hello{/foo} world{/bar}!", LANG_NAME, &META_SYNTAX).unwrap_err();
    assert_eq!(error.message, "unmatched tag name");
    assert_eq!(error.range, (25, 28));
}
