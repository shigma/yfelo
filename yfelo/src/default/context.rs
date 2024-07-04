use std::collections::HashMap;
use std::rc::Rc;

use dyn_std::Instance;
use yfelo_core::{factory, writer::render, ContextFactory, Definition};

use super::{Expr, Pattern, RuntimeError, Value};

pub struct Context {
    store: HashMap<String, Rc<Value>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            store: Default::default(),
        }
    }

    fn preapply(&self, params: &Vec<(Pattern, Option<Expr>)>, args: Vec<Value>) -> Result<Self, RuntimeError> {
        let mut inner = self.fork();
        if args.len() > params.len() {
            return Err(RuntimeError {
                message: format!("expect {} arguments, found {}", params.len(), args.len()),
            });
        }
        for (index, (pattern, default)) in params.iter().enumerate() {
            let value = match args.get(index) {
                Some(value) => value.clone(),
                None => match default {
                    Some(expr) => inner.eval(expr)?,
                    None => {
                        return Err(RuntimeError {
                            message: format!("missing required argument '{}'", pattern),
                        });
                    },
                },
            };
            inner.bind(pattern, value)?;
        }
        Ok(inner)
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
        // todo: try
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
                    Value::Ref(self.store[ident].clone())
                }
            },
            Expr::Array(vec, _) => {
                Value::Array(vec.iter().map(|expr| {
                    self.eval(expr).map(|v| Rc::new(v))
                }).collect::<Result<Vec<_>, _>>()?)
            },
            Expr::Apply(func, args, _) => {
                let func = self.eval(func)?;
                // todo: clone
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
        Context {
            store: self.store.clone(),
        }
    }

    fn bind(&mut self, pattern: &Pattern, value: Value) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Ident(ident, _) => {
                self.store.insert(ident.into(), Rc::new(value));
                Ok(())
            },
        }
    }

    fn def(&mut self, name: &str, params: Vec<(Pattern, Option<Expr>)>, definition: Definition) -> Result<(), RuntimeError> {
        self.store.insert(name.into(), Rc::new(Value::Lazy(params, definition)));
        Ok(())
    }

    fn apply(&self, name: &str, args: Vec<Expr>, init: &mut dyn FnMut(&mut dyn yfelo_core::Context) -> Result<String, Box<dyn yfelo_core::RuntimeError>>) -> Result<Value, RuntimeError> {
        Self::apply(&self, &self.store[name], args, init)
    }
}
