use std::cell::RefCell;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::rc::{Rc, Weak};

use dyn_std::Instance;
use yfelo_core::{factory, writer::render, ContextFactory, Definition};

use super::{Expr, Pattern, RuntimeError, Value};

#[derive(Default)]
struct ContextInner {
    parent: Weak<ContextInner>,
    store: RefCell<HashMap<String, Rc<Value>>>,
}

impl ContextInner {
    fn get(self: &Rc<Self>, key: &str) -> Value {
        let mut this = Some(self.clone());
        while let Some(inner) = this {
            if let Some(v) = inner.store.borrow().get(key) {
                return Value::Ref(v.clone())
            }
            this = inner.parent.upgrade();
        }
        Value::Null
    }

    fn set(&self, key: String, value: Value) -> Result<(), RuntimeError> {
        match self.store.borrow_mut().entry(key) {
            Entry::Vacant(entry) => {
                entry.insert(Rc::new(value));
                Ok(())
            },
            Entry::Occupied(entry) => Err(RuntimeError {
                message: format!("'{}' is already defined", entry.key()),
            }),
        }
    }
}

#[derive(Default)]
pub struct Context(Rc<ContextInner>);

impl Context {
    pub fn new() -> Self {
        Self::default()
    }

    fn preapply(&self, params: &Vec<(Pattern, Option<Expr>)>, args: Vec<Value>) -> Result<Self, RuntimeError> {
        let mut ctx = self.fork();
        if args.len() > params.len() {
            return Err(RuntimeError {
                message: format!("expect {} arguments, found {}", params.len(), args.len()),
            });
        }
        let args = args.into_iter().map(Some).chain(std::iter::repeat(None));
        for ((pattern, default), arg) in params.iter().zip(args) {
            let value = match arg {
                Some(value) => value,
                None => match default {
                    Some(expr) => ctx.eval(expr)?,
                    None => {
                        return Err(RuntimeError {
                            message: format!("missing required argument '{}'", pattern),
                        });
                    },
                },
            };
            ctx.bind(pattern, value)?;
        }
        Ok(ctx)
    }

    fn postapply(self, definition: &Definition) -> Result<Value, RuntimeError> {
        match definition {
            Definition::Inline(expr) => {
                self.eval(&expr.as_any().downcast_ref::<Instance<Expr, ()>>().unwrap().0)
            },
            Definition::Block(nodes) => {
                match render(&mut Instance::new(self), &nodes) {
                    Ok(value) => Ok(Value::String(value)),
                    Err(e) => Err(e.as_any_box().downcast::<Instance<RuntimeError, ()>>().unwrap().0),
                }
            },
        }
    }

    fn apply(&self, f: &Value, args: Vec<Expr>, init: &mut dyn FnMut(&mut dyn yfelo_core::Context) -> Result<String, Box<dyn yfelo_core::RuntimeError>>) -> Result<Value, RuntimeError> {
        let Value::Lazy(params, definition) = f else {
            return Err(RuntimeError {
                message: format!("expect function, found {}", f.type_name()),
            });
        };
        let args = args.iter()
            .map(|expr| self.eval(expr))
            .collect::<Result<Vec<_>, _>>()?;
        let mut inst = Instance::new(self.preapply(params, args)?);
        init(&mut inst).map_err(|e| {
            e.as_any_box().downcast::<Instance<RuntimeError, ()>>().unwrap().0
        })?;
        inst.0.postapply(definition)
    }
}

impl factory::Context<Expr, Pattern, Value, RuntimeError> for Context {
    fn eval(&self, expr: &Expr) -> Result<Value, RuntimeError> {
        Ok(match expr {
            Expr::Number(n, _) => Value::Number(*n),
            Expr::String(s, _) => Value::String(s.clone()),
            Expr::Ident(ident, _) => {
                if ident == "true" {
                    Value::Bool(true)
                } else if ident == "false" {
                    Value::Bool(false)
                } else if ident == "null" {
                    Value::Null
                } else {
                    self.0.get(ident)
                }
            },
            Expr::Array(vec, _) => {
                Value::Array(vec.iter().map(|expr| {
                    self.eval(expr).map(|v| Rc::new(v))
                }).collect::<Result<Vec<_>, _>>()?)
            },
            Expr::Apply(func, args, _) => {
                let func = self.eval(func)?;
                // fixme
                self.apply(&func, args.clone(), &mut |_| Ok(String::new()))?
            },
            Expr::Unary(op, expr, _) => {
                let value = self.eval(expr)?;
                op.eval(value)?
            },
            Expr::Binary(lhs, op, rhs, _) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                op.eval(lhs, rhs)?
            },
        })
    }

    fn fork(&self) -> Self {
        Self(Rc::new(ContextInner {
            parent: Rc::downgrade(&self.0),
            store: Default::default(),
        }))
    }

    fn bind(&mut self, pattern: &Pattern, value: Value) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Ident(ident, _) => {
                self.0.set(ident.into(), value)?;
                Ok(())
            },
        }
    }

    fn def(&mut self, name: &str, params: Vec<(Pattern, Option<Expr>)>, definition: Definition) -> Result<(), RuntimeError> {
        self.0.set(name.into(), Value::Lazy(params, definition))?;
        Ok(())
    }

    fn apply(&self, name: &str, args: Vec<Expr>, init: &mut dyn FnMut(&mut dyn yfelo_core::Context) -> Result<String, Box<dyn yfelo_core::RuntimeError>>) -> Result<Value, RuntimeError> {
        Self::apply(&self, &self.0.get(name), args, init)
    }
}
