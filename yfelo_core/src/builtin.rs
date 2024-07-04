use std::fmt::Debug;

use dyn_std::Instance;

use crate::directive::{DirectiveFactory as Directive, Node};
use crate::language::{Context, Expr, Pattern, RuntimeError, SyntaxError};
use crate::reader::{Reader, TagInfo};
use crate::writer::render;
use crate::{Definition, Element};

/// No-op directive.
/// 
/// ### Example
/// ```yfelo
/// {@stub}
/// ```
/// 
/// ### Example
/// ```yfelo
/// {#stub}
///   ...
/// {/stub}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Stub;

impl Directive for Stub {
    fn open(_: &mut Reader, _: &TagInfo) -> Result<Self, SyntaxError> {
        Ok(Self)
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        render(ctx, nodes)
    }
}

/// Conditional directive.
/// 
/// ### Example
/// ```yfelo
/// {#if EXPR}
///     ...
/// {/if}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct If {
    expr: Box<dyn Expr>,
}

impl Directive for If {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let expr = reader.parse_expr()?;
        Ok(Self { expr })
    }

    fn branch(tags: &[TagInfo], info: &TagInfo) -> Result<(), SyntaxError> {
        if let Some(prev) = tags.last() {
            if prev.name == "else" {
                return Err(SyntaxError {
                    message: format!("'{}' cannot come after 'else'", info.name),
                    range: info.range,
                })
            }
        }
        Ok(())
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], branches: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        if ctx.eval(&self.expr)?.as_bool()? {
            return render(ctx, nodes);
        }
        for branch in branches {
            if let Some(instance) = branch.directive.as_any().downcast_ref::<Instance<If, ()>>() {
                if ctx.eval(&instance.0.expr)?.as_bool()? {
                    return render(ctx, &branch.children);
                }
            } else if let Some(_) = branch.directive.as_any().downcast_ref::<Instance<Stub, ()>>() {
                return render(ctx, &branch.children);
            } else {
                panic!("unexpected directive instance: {:?}", branch.directive)
            }
        }
        Ok(String::new())
    }
}

/// Loop directive.
/// 
/// ### Example
/// ```yfelo
/// {#for PAT in EXPR}
///     ...
/// {/for}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct For {
    vpat: Box<dyn Pattern>,
    kpat: Option<Box<dyn Pattern>>,
    expr: Box<dyn Expr>,
}

impl Directive for For {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        info.expect_children()?;
        let vpat = reader.parse_pattern()?;
        let kpat = match reader.parse_punct(",") {
            Ok(_) => Some(reader.parse_pattern()?),
            Err(_) => None,
        };
        reader.parse_keyword("in")?;
        let expr = reader.parse_expr()?;
        Ok(Self { vpat, kpat, expr })
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let entries = ctx.eval(&self.expr)?.as_entries()?;
        let mut output = String::new();
        for entry in entries {
            let mut inner = ctx.fork();
            inner.bind(&self.vpat, entry.0)?;
            if let Some(kpat) = &self.kpat {
                inner.bind(&kpat, entry.1)?;
            }
            output += &render(inner.as_mut(), nodes)?;
        }
        Ok(output)
    }
}

/// Definition directive.
/// 
/// ### Example
/// Bind the result of `EXPR` to `PAT`.
/// 
/// ```yfelo
/// {@def PAT = EXPR}
/// ```
/// 
/// ### Example
/// Define a function `NAME` with `PARAMS`.
/// 
/// ```yfelo
/// {#def NAME(PARAMS)}
///     ...
/// {/def}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum Def {
    Var(DefVar),
    Fn(DefFn),
}

/// Variable definition. Must have an inline expression.
/// 
/// ```yfelo
/// {@def PAT = EXPR}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DefVar {
    pat: Box<dyn Pattern>,
    expr: Box<dyn Expr>,
}

impl DefVar {
    fn render(&self, ctx: &mut dyn Context, _: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let value = ctx.eval(&self.expr)?;
        ctx.bind(&self.pat, value)?;
        Ok(String::new())
    }
}

/// Function definition, which can be further divided into two types:
/// 
/// 1. With inline expression.
/// ```yfelo
/// {@def NAME(...) = EXPR}
/// ```
/// 
/// 2. With block content (parentheses are optional).
/// ```yfelo
/// {#def NAME(...)}
///     ...
/// {/def}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct DefFn {
    ident: String,
    params: Vec<(Box<dyn Pattern>, Option<Box<dyn Expr>>)>,
    expr: Option<Box<dyn Expr>>,
}

impl DefFn {
    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let Self { expr, .. } = self.clone();
        ctx.def(&self.ident, self.params.clone(), match &expr {
            Some(expr) => Definition::Inline(expr.clone()),
            None => Definition::Block(nodes.into()),
        })?;
        Ok(String::new())
    }
}

impl Def {
    fn as_ident(pat: Box<dyn Pattern>) -> Result<String, SyntaxError> {
        pat.into_ident().ok_or_else(|| SyntaxError {
            message: "expected identifier".into(),
            range: (0, 0), // fixme
        })
    }
}

impl Directive for Def {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        let pat = reader.parse_pattern()?;
        Ok(if let Ok(_) = reader.parse_punct("(") {
            let ident = Self::as_ident(pat)?;
            let mut params = vec![];
            loop {
                if let Ok(_) = reader.parse_punct(")") {
                    break;
                }
                let pat = reader.parse_pattern()?;
                let expr = if let Ok(_) = reader.parse_punct("=") {
                    Some(reader.parse_expr()?)
                } else {
                    None
                };
                params.push((pat, expr));
                if let Ok(_) = reader.parse_punct(",") {
                    continue;
                }
                reader.parse_punct(")")?;
                break;
            }
            let expr = if let Ok(_) = reader.parse_punct("=") {
                info.expect_empty()?;
                Some(reader.parse_expr()?)
            } else {
                info.expect_children()?;
                None
            };
            Self::Fn(DefFn { ident, params, expr })
        } else {
            if let Ok(_) = reader.parse_punct("=") {
                info.expect_empty()?;
                let expr = reader.parse_expr()?;
                Self::Var(DefVar { pat, expr })
            } else {
                let ident = Self::as_ident(pat)?;
                info.expect_children()?;
                Self::Fn(DefFn { ident, expr: None, params: vec![] })
            }
        })
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], branches: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        match self {
            Self::Var(def) => def.render(ctx, nodes, branches),
            Self::Fn(def) => def.render(ctx, nodes, branches),
        }
    }
}

/// Application directive.
/// 
/// ### Example
/// Apply function `NAME` with `PARAMS`.
/// 
/// ```yfelo
/// {@apply NAME(PARAMS)}
/// ```
#[derive(Debug, Clone, PartialEq)]
pub struct Apply {
    name: String,
    args: Vec<Box<dyn Expr>>,
}

impl Directive for Apply {
    fn open(reader: &mut Reader, info: &TagInfo) -> Result<Self, SyntaxError> {
        let name = reader.parse_ident()?.0.into();
        let args = if let Ok(_) = reader.parse_punct("(") {
            let mut args = vec![];
            loop {
                if let Ok(_) = reader.parse_punct(")") {
                    break;
                }
                args.push(reader.parse_expr()?);
                if let Ok(_) = reader.parse_punct(",") {
                    continue;
                }
                reader.parse_punct(")")?;
                break;
            }
            args
        } else {
            vec![]
        };
        info.expect_empty()?;
        Ok(Self { name, args })
    }

    fn render(&self, ctx: &mut dyn Context, nodes: &[Node], _: &[Element]) -> Result<String, Box<dyn RuntimeError>> {
        let value = ctx.apply(&self.name, self.args.clone(), &mut |ctx| {
            render(ctx, nodes)
        })?;
        Ok(value.to_string()?)
    }
}
