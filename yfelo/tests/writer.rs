use once_cell::sync::Lazy;
use yfelo::{MetaSyntax, Yfelo};
use yfelo::default::{Context, Language, Pattern, Value};

const YFELO: Lazy<Yfelo> = Lazy::new(|| Yfelo::new());

const LANG: Lazy<Box<dyn yfelo::Language>> = Lazy::new(|| Box::new(Language));

const META_SYNTAX: MetaSyntax = MetaSyntax {
    left: "{",
    right: "}",
};

#[test]
pub fn basic_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Context::new());
    ctx.bind(&Pattern::from("world"), Box::new(Value::from("yfelo"))).unwrap();
    let output = y.run("Hello, {world}!", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}

#[test]
pub fn if_1() {
    let (y, l) = (YFELO, LANG);
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Context::new());
    ctx.bind(&Pattern::from("foo"), Box::new(Value::from(true))).unwrap();
    ctx.bind(&Pattern::from("bar"), Box::new(Value::from(false))).unwrap();
    let output = y.run("{#if foo}Hello{/if}, {#if bar}world{/if}!", l.as_ref(), &META_SYNTAX, ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, !");
}
