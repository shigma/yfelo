use std::marker::PhantomData;

use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::{MetaSyntax, Yfelo};
use yfelo::default::{Context, Expr, Language, Pattern, Value};

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
    let world: Box<dyn yfelo::Pattern> = Box::new(Instance::new(Pattern::from("world")));
    ctx.bind(&world, Box::new(Instance::new(Value::from("yfelo")))).unwrap();
    let output = y.run("Hello, {world}!", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}

#[test]
pub fn if_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let foo: Box<dyn yfelo::Pattern> = Box::new(Instance::new(Pattern::from("foo")));
    let bar: Box<dyn yfelo::Pattern> = Box::new(Instance::new(Pattern::from("bar")));
    ctx.bind(&foo, Box::new(Instance::new(Value::from(true)))).unwrap();
    ctx.bind(&bar, Box::new(Instance::new(Value::from(false)))).unwrap();
    let output = y.run("
        {#if foo}Hello{/if}, {#if bar}world{/if}!
    ", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, !");
}
