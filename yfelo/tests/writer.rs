use dyn_std::Instance;
use once_cell::sync::Lazy;
use yfelo::Yfelo;
use yfelo::default::{Context, Expr, Language, Pattern};

const YFELO: Lazy<Yfelo> = Lazy::new(|| {
    let mut yfelo = Yfelo::new();
    yfelo.add_language::<Expr, Pattern, Language>("default");
    yfelo
});

fn render(input: &str, ctx: &mut dyn yfelo::Context) -> Result<String, yfelo::Error> {
    YFELO.render(&(String::from("{@yfelo}\n") + input), ctx)
}

#[test]
pub fn basic_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = render("
        {@def world = 'yfelo'}
        Hello, {world}!
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}

#[test]
pub fn def_inline_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = render("
        {@def foo = 'bar'}
        {@def bar = 'foo'}
        {foo + bar}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "barfoo");
}

#[test]
pub fn def_block_1() {
    let mut ctx: Box<dyn yfelo::Context> = Box::new(Instance::new(Context::new()));
    let output = render("
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
    let output = render("
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
    let output = render("
        {#def text(world = 'yfelo')}
            Hello, {world}!
        {/def}
        {@apply text}
    ", ctx.as_mut()).unwrap();
    assert_eq!(output, "Hello, yfelo!");
}
