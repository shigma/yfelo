use std::marker::PhantomData;

use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::{MetaSyntax, SyntaxError, Yfelo};
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
    let error = y.parse("{#if}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'if': expect expression".into(),
        range: (4, 4),
    });
}

#[test]
pub fn parse_2() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#if x x}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (7, 7),
    });
}

#[test]
pub fn parse_3() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{@if}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "directive 'if' should not be empty".into(),
        range: (2, 4),
    });
}

#[test]
pub fn parse_4() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#if x}{:else y}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (14, 14),
    });
}

#[test]
pub fn parse_5() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#if x}{:else}{:else}{/if}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "'else' cannot come after 'else'".into(),
        range: (16, 20),
    });
}

#[test]
pub fn parse_6() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#if x}{:elif}{:else}{/if}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'elif': expect expression".into(),
        range: (13, 13),
    });
}

#[test]
pub fn parse_7() {
    let (y, l) = (YFELO, LANG);
    let error = y.parse("{#if x}{:else}{:elif x}{/if}", l.as_ref(), &META_SYNTAX).unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "'elif' cannot come after 'else'".into(),
        range: (16, 20),
    });
}

#[test]
pub fn render_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#if true}
            Hello
        {:else}
            World
        {/if}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello");
}

#[test]
pub fn render_2() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#if false}
            Hello
        {:else}
            World
        {/if}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "World");
}

#[test]
pub fn render_3() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#if false}
            Hello
        {:elif true}
            World
        {:else}
            Yfelo
        {/if}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "World");
}

#[test]
pub fn render_4() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#if false}
            Hello
        {:elif false}
            World
        {:elif false}
            Yfelo
        {/if}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "");
}

#[test]
pub fn render_5() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.render("
        {#if false}
            Hello
        {:elif false}
            World
        {:elif true}
            Yfelo
        {/if}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Yfelo");
}
