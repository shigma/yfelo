use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::builtin::Stub;
use yfelo::default::{Expr, Language, Pattern};
use yfelo::{Element, Header, Node, SyntaxError, Yfelo};

const HEADER: Lazy<Header> = Lazy::new(|| {
    let yfelo = Box::leak(Box::new(Yfelo::new()));
    yfelo.add_directive::<Stub>("foo");
    yfelo.add_directive::<Stub>("bar");
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo.prepare("{@yfelo}", false).unwrap()
});

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

macro_rules! index {
    ($lhs:expr, $rhs:expr, $is_expr:expr, $range:tt $(,)?) => {
        Expr::Index(Box::from($lhs), Box::from($rhs), $is_expr, Some($range))
    };
}

#[test]
pub fn basic_1() {
    let nodes = HEADER.parse("(Hello) {world}!").unwrap();
    assert_eq!(nodes, vec![
        Node::Text("(Hello) ".into()),
        Node::Expr(Box::from(Instance::new(ident!("world", (9, 14))))),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn basic_2() {
    let nodes = HEADER.parse("{world}").unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(ident!("world", (1, 6))))),
    ]);
}

#[test]
pub fn basic_3() {
    let nodes = HEADER.parse("{w(or[ld])}").unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(apply!(
            ident!("w", (1, 2)),
            index!(ident!("or", (3, 5)), ident!("ld", (6, 8)), true, (5, 9)),
            (2, 10),
        )))),
    ]);
}

#[test]
pub fn basic_4() {
    let nodes = HEADER.parse("{{w:or,ld}}!").unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(Expr::Object(vec![
            (ident!("w", (2, 3)), Some(ident!("or", (4, 6)))),
            (ident!("ld", (7, 9)), None),
        ], Some((1, 10)))))),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn invalid_tag_1() {
    let err = HEADER.parse("{Hello} {world").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (14, 14),
    });
}

#[test]
pub fn invalid_tag_2() {
    let err = HEADER.parse("{Hel(lo}").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (4, 4),
    });
}

#[test]
pub fn invalid_tag_3() {
    let err = HEADER.parse("{Hel)lo}").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (4, 4),
    });
}

#[test]
pub fn invalid_tag_4() {
    let err = HEADER.parse("{H(e[l)l]o}").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (2, 2),
    });
}

#[test]
pub fn tag_1() {
    let nodes = HEADER.parse("{#foo}Hello{/foo} {#bar}world{/bar}!").unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::new(Instance::new(Stub)),
            nodes: vec![Node::Text("Hello".into())],
            branches: vec![],
        }),
        Node::Text(" ".into()),
        Node::Element(Element {
            directive: Box::new(Instance::new(Stub)),
            nodes: vec![Node::Text("world".into())],
            branches: vec![],
        }),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn tag_2() {
    let nodes = HEADER.parse("{#foo}Hello{@bar} {#bar}world{/bar}!{/foo}").unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::new(Instance::new(Stub)),
            nodes: vec![
                Node::Text("Hello".into()),
                Node::Element(Element {
                    directive: Box::new(Instance::new(Stub)),
                    nodes: vec![],
                    branches: vec![],
                }),
                Node::Text(" ".into()),
                Node::Element(Element {
                    directive: Box::new(Instance::new(Stub)),
                    nodes: vec![Node::Text("world".into())],
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
    let error = HEADER.parse("{#foo}Hello {#bar}world{/foo}!{/bar}").unwrap_err();
    assert_eq!(error.message, "unmatched tag name: expect 'bar', found 'foo'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_2() {
    let error = HEADER.parse("{#foo}Hello{/foo} world{/bar}!").unwrap_err();
    assert_eq!(error.message, "unmatched tag name 'bar'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_3() {
    let error = HEADER.parse("{#foo}Hello{/foo} world{#bar}!").unwrap_err();
    assert_eq!(error.message, "unmatched tag name 'bar'");
    assert_eq!(error.range, (25, 28));
}
