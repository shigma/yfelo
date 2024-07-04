use std::marker::PhantomData;

use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::builtin::For;
use yfelo::{Element, MetaSyntax, Node, SyntaxError, Yfelo};
use yfelo::default::{Context, Expr, Language, Pattern};

const YFELO: Lazy<Yfelo> = Lazy::new(|| Yfelo::new());

const LANG: Lazy<Box<dyn yfelo::Language>> = Lazy::new(|| Box::new(PhantomData::<(Language, Expr, Pattern)>));

const META_SYNTAX: MetaSyntax = MetaSyntax {
    left: "{",
    right: "}",
};

#[test]
pub fn parse_1() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect pattern".into(),
        range: (5, 5),
    });
}

#[test]
pub fn parse_2() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for x}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect keyword 'in'".into(),
        range: (7, 7),
    });
}

#[test]
pub fn parse_3() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{@for}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "directive 'for' should not be empty".into(),
        range: (2, 5),
    });
}

#[test]
pub fn parse_4() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for x in}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect expression".into(),
        range: (10, 10),
    });
}

#[test]
pub fn parse_5() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for x in y z}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (13, 13),
    });
}

#[test]
pub fn parse_6() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#for x, y, z in w}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'for': expect keyword 'in'".into(),
        range: (10, 10),
    });
}

#[test]
pub fn parse_7() {
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{#for x in y}{/for}", l.as_ref(), &META_SYNTAX).unwrap();
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
    let (y, l) = (YFELO, LANG);
    let nodes = y.parse("{#for x, y in z}{/for}", l.as_ref(), &META_SYNTAX).unwrap();
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
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#for a in [1, 2, 3]}
            {a * 2}
        {/for}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "246");
}

#[test]
pub fn render_2() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#for a, b in [1, 2, 3]}
            {b + 1}. {a * 2}
        {/for}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "1. 22. 43. 6");
}

#[test]
pub fn render_3() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#for x in {x: 2, y: 3, z: 1}}
            {x}
        {/for}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "231");
}

#[test]
pub fn render_4() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#for x, y in {x: 2, y: 3, z: 1}}
            {y}. {x}
        {/for}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "x. 2y. 3z. 1");
}
