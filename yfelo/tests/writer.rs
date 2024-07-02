use std::marker::PhantomData;

use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::{MetaSyntax, Yfelo};
use yfelo::default::{Context, Expr, Language, Pattern};

const YFELO: Lazy<Yfelo> = Lazy::new(|| Yfelo::new());

const LANG: Lazy<Box<dyn yfelo::Language>> = Lazy::new(|| Box::new(PhantomData::<(Language, Expr, Pattern)>));

const META_SYNTAX: MetaSyntax = MetaSyntax {
    left: "{",
    right: "}",
};

#[test]
pub fn basic_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.run("
        {@def world = 'yfelo'}
        Hello, {world}!
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}

#[test]
pub fn if_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.run("
        {@def foo = true}
        {@def bar = false}
        {#if foo}Hello{/if}, {#if bar}world{/if}!
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, !");
}

#[test]
pub fn def_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = y.run("
        {#def text}
            Hello, world!
        {/def}
        {@apply text}
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, world!");
}
