use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::{Header, SyntaxError, Yfelo};
use yfelo::default::{Context, Expr, Language, Pattern};

const HEADER: Lazy<Header> = Lazy::new(|| {
    let yfelo = Box::leak(Box::new(Yfelo::new()));
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo.prepare("{@yfelo}", false).unwrap()
});

#[test]
pub fn parse_1() {
    let error = HEADER.parse("{#if}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'if': expect expression".into(),
        range: (4, 4),
    });
}

#[test]
pub fn parse_2() {
    let error = HEADER.parse("{#if x x}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (7, 7),
    });
}

#[test]
pub fn parse_3() {
    let error = HEADER.parse("{@if}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "directive 'if' should not be empty".into(),
        range: (2, 4),
    });
}

#[test]
pub fn parse_4() {
    let error = HEADER.parse("{#if x}{:else y}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid tag syntax: expect '}'".into(),
        range: (14, 14),
    });
}

#[test]
pub fn parse_5() {
    let error = HEADER.parse("{#if x}{:else}{:else}{/if}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "'else' cannot come after 'else'".into(),
        range: (16, 20),
    });
}

#[test]
pub fn parse_6() {
    let error = HEADER.parse("{#if x}{:elif}{:else}{/if}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "invalid syntax for directive 'elif': expect expression".into(),
        range: (13, 13),
    });
}

#[test]
pub fn parse_7() {
    let error = HEADER.parse("{#if x}{:else}{:elif x}{/if}").unwrap_err();
    assert_eq!(error, SyntaxError {
        message: "'elif' cannot come after 'else'".into(),
        range: (16, 20),
    });
}

#[test]
pub fn render_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#if true}
            Hello
        {:else}
            World
        {/if}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello");
}

#[test]
pub fn render_2() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#if false}
            Hello
        {:else}
            World
        {/if}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "World");
}

#[test]
pub fn render_3() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#if false}
            Hello
        {:elif true}
            World
        {:else}
            Yfelo
        {/if}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "World");
}

#[test]
pub fn render_4() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#if false}
            Hello
        {:elif false}
            World
        {:elif false}
            Yfelo
        {/if}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "");
}

#[test]
pub fn render_5() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#if false}
            Hello
        {:elif false}
            World
        {:elif true}
            Yfelo
        {/if}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Yfelo");
}
