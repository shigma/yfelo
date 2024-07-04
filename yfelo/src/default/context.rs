use dyn_std::Instance;
use yfelo_core::{factory, writer::render, ContextFactory, Definition};

use super::{Expr, Pattern, RuntimeError, Value};

pub struct Context {
    inner: Value,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Value::Object(Default::default()),
        }
    }

    fn preapply(&self, params: &Vec<(Pattern, Option<Expr>)>, args: Vec<Value>) -> Result<Self, RuntimeError> {
        let mut inner = self.fork();
        for (index, (pattern, default)) in params.iter().enumerate() {
            let value = match args.get(index) {
                Some(value) => value.clone(),
                None => match default {
                    Some(expr) => inner.eval(expr)?,
                    None => {
                        return Err(RuntimeError {});
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
        let Value::Abs(params, definition) = f else {
            return Err(RuntimeError {});
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
            Expr::Number(n) => Value::Number(n.clone()),
            Expr::String(s) => Value::String(s.clone()),
            Expr::Ident(ident) => {
                if ident == "true" {
                    Value::Bool(true)
                } else if ident == "false" {
                    Value::Bool(false)
                } else if ident == "null" {
                    Value::Null
                } else {
                    self.inner[ident].clone()
                }
            },
            Expr::Array(vec) => {
                Value::Array(vec.iter().map(|expr| {
                    self.eval(expr)
                }).collect::<Result<Vec<_>, _>>()?)
            },
            Expr::Apply(func, args) => {
                let func = self.eval(func)?;
                // todo: clone
                self.apply(&func, args.clone(), &mut |_| Ok(String::new()))?
            },
            Expr::Unary(op, expr) => {
                let value = self.eval(expr)?;
                op.eval(value)?
            },
            Expr::Binary(lhs, op, rhs) => {
                let lhs = self.eval(lhs)?;
                let rhs = self.eval(rhs)?;
                op.eval(lhs, rhs)?
            },
        })
    }

    fn fork(&self) -> Self {
        Context {
            inner: self.inner.clone(),
        }
    }

    fn bind(&mut self, pattern: &Pattern, value: Value) -> Result<(), RuntimeError> {
        match pattern {
            Pattern::Ident(ident) => {
                self.inner[ident] = value;
                Ok(())
            },
        }
    }

    fn def(&mut self, name: &str, params: Vec<(Pattern, Option<Expr>)>, definition: Definition) -> Result<(), RuntimeError> {
        self.inner[name.into()] = Value::Abs(params, definition);
        Ok(())
    }

    fn apply(&self, name: &str, args: Vec<Expr>, init: &mut dyn FnMut(&mut dyn yfelo_core::Context) -> Result<String, Box<dyn yfelo_core::RuntimeError>>) -> Result<Value, RuntimeError> {
        Self::apply(&self, &self.inner[name], args, init)
    }
}
