use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::builtin::Stub;
use yfelo::default::{Expr, Language, Pattern};
use yfelo::{Element, Node, SyntaxError, Yfelo};

const YFELO: Lazy<Yfelo> = Lazy::new(|| {
    let mut yfelo = Yfelo::new();
    yfelo.add_directive::<Stub>("foo");
    yfelo.add_directive::<Stub>("bar");
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo
});

fn parse(input: &str) -> Result<Vec<Node>, SyntaxError> {
    YFELO.parse(&(String::from("{@yfelo}\n") + input))
}

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
    let nodes = parse("(Hello) {world}!").unwrap();
    assert_eq!(nodes, vec![
        Node::Text("(Hello) ".into()),
        Node::Expr(Box::from(Instance::new(ident!("world", (9, 14))))),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn basic_2() {
    let nodes = parse("{world}").unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(ident!("world", (1, 6))))),
    ]);
}

#[test]
pub fn basic_3() {
    let nodes = parse("{w(or[ld])}").unwrap();
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
    let nodes = YFELO.parse("[@yfelo]\n[w[or][ld]]!").unwrap();
    assert_eq!(nodes, vec![
        Node::Expr(Box::from(Instance::new(index!(
            index!(ident!("w", (1, 2)), ident!("or", (3, 5)), true, (2, 6)),
            ident!("ld", (7, 9)),
            true, 
            (6, 10),
        )))),
        Node::Text("!".into()),
    ]);
}

#[test]
pub fn invalid_tag_1() {
    let err = parse("{Hello} {world").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (14, 14),
    });
}

#[test]
pub fn invalid_tag_2() {
    let err = parse("{Hel(lo}").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (4, 4),
    });
}

#[test]
pub fn invalid_tag_3() {
    let err = parse("{Hel)lo}").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (4, 4),
    });
}

#[test]
pub fn invalid_tag_4() {
    let err = parse("{H(e[l)l]o}").unwrap_err();
    assert_eq!(err, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (2, 2),
    });
}

#[test]
pub fn tag_1() {
    let nodes = parse("{#foo}Hello{/foo} {#bar}world{/bar}!").unwrap();
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
    let nodes = parse("{#foo}Hello{@bar} {#bar}world{/bar}!{/foo}").unwrap();
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
    let error = parse("{#foo}Hello {#bar}world{/foo}!{/bar}").unwrap_err();
    assert_eq!(error.message, "unmatched tag name: expect 'bar', found 'foo'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_2() {
    let error = parse("{#foo}Hello{/foo} world{/bar}!").unwrap_err();
    assert_eq!(error.message, "unmatched tag name 'bar'");
    assert_eq!(error.range, (25, 28));
}

#[test]
pub fn unmatched_tag_3() {
    let error = parse("{#foo}Hello{/foo} world{#bar}!").unwrap_err();
    assert_eq!(error.message, "unmatched tag name 'bar'");
    assert_eq!(error.range, (25, 28));
}
