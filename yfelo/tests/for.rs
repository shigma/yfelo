use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::builtin::For;
use yfelo::{Element, Header, Node, SyntaxError, Yfelo};
use yfelo::default::{Context, Expr, Language, Pattern};

const HEADER: Lazy<Header> = Lazy::new(|| {
    let yfelo = Box::leak(Box::new(Yfelo::new()));
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo.prepare("{@yfelo}", false).unwrap()
});

#[test]
pub fn parse_1() {
    let error = HEADER.parse("{#for}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect pattern".into(),
        range: (5, 5),
    });
}

#[test]
pub fn parse_2() {
    let error = HEADER.parse("{#for x}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect keyword 'in'".into(),
        range: (7, 7),
    });
}

#[test]
pub fn parse_3() {
    let error = HEADER.parse("{@for}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "directive 'for' should not be empty".into(),
        range: (2, 5),
    });
}

#[test]
pub fn parse_4() {
    let error = HEADER.parse("{#for x in}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect expression".into(),
        range: (10, 10),
    });
}

#[test]
pub fn parse_5() {
    let error = HEADER.parse("{#for x in y z}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (13, 13),
    });
}

#[test]
pub fn parse_6() {
    let error = HEADER.parse("{#for x, y, z in w}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect keyword 'in'".into(),
        range: (10, 10),
    });
}

#[test]
pub fn parse_7() {
    let nodes = HEADER.parse("{#for x in y}{/for}").unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::new(Instance::new(For {
                vpat: Box::new(Instance::new(Pattern::Ident("x".into(), Some((6, 7))))),
                kpat: None,
                expr: Box::new(Instance::new(Expr::Ident("y".into(), Some((11, 12))))),
            })),
            nodes: vec![],
            branches: vec![],
        }),
    ]);
}

#[test]
pub fn parse_8() {
    let nodes = HEADER.parse("{#for x, y in z}{/for}").unwrap();
    assert_eq!(nodes, vec![
        Node::Element(Element {
            directive: Box::new(Instance::new(For {
                vpat: Box::new(Instance::new(Pattern::Ident("x".into(), Some((6, 7))))),
                kpat: Some(Box::new(Instance::new(Pattern::Ident("y".into(), Some((9, 10)))))),
                expr: Box::new(Instance::new(Expr::Ident("z".into(), Some((14, 15))))),
            })),
            nodes: vec![],
            branches: vec![],
        }),
    ]);
}

#[test]
pub fn render_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#for a in [1, 2, 3]}
            {a * 2}
        {/for}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "246");
}

#[test]
pub fn render_2() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#for a, b in [1, 2, 3]}
            {b + 1}. {a * 2}
        {/for}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "1. 22. 43. 6");
}

#[test]
pub fn render_3() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#for x in {x: 2, y: 3, z: 1}}
            {x}
        {/for}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "231");
}

#[test]
pub fn render_4() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#for x, y in {x: 2, y: 3, z: 1}}
            {y}. {x}
        {/for}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "x. 2y. 3z. 1");
}
