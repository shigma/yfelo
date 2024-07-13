use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::{Header, Yfelo};
use yfelo::default::{Context, Expr, Language, Pattern};

const HEADER: Lazy<Header> = Lazy::new(|| {
    let yfelo = Box::leak(Box::new(Yfelo::new()));
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo.prepare("{@yfelo}", false).unwrap()
});

#[test]
pub fn basic_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {@def world = 'yfelo'}
        Hello, {world}!
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}

#[test]
pub fn def_inline_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {@def foo = 'bar'}
        {@def bar = 'foo'}
        {foo + bar}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "barfoo");
}

#[test]
pub fn def_block_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#def text}
            Hello, world!
        {/def}
        {@apply text}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, world!");
}

#[test]
pub fn def_block_2() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#def text(world)}
            Hello, {world}!
        {/def}
        {@apply text('yfelo')}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}

#[test]
pub fn def_default_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = HEADER.render("
        {#def text(world = 'yfelo')}
            Hello, {world}!
        {/def}
        {@apply text}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}
