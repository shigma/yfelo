use std::collections::{BTreeMap, HashMap};

use dyn_std::Instance;
use yfelo_core::{factory, writer::render, Definiton};

use super::{Expr, Pattern, RuntimeError, Value};

pub struct Context {
    inner: Value,
    f_store: HashMap<String, Box<dyn Fn(Vec<Value>) -> Result<Value, RuntimeError>>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            inner: Value::Object(BTreeMap::new()),
            f_store: HashMap::new(),
        }
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
                    self.eval(expr).unwrap()
                }).collect())
            },
            // Expr::Apply(func, args) => {
            //     let func = self.eval(func).unwrap();
            //     let args = args.iter().map(|expr| {
            //         self.eval(expr).unwrap()
            //     }).collect();
            //     Value::Null
            // },
            Expr::Unary(op, expr) => {
                let value = self.eval(expr).unwrap();
                op.eval(value)?
            },
            Expr::Binary(lhs, op, rhs) => {
                let lhs = self.eval(lhs).unwrap();
                let rhs = self.eval(rhs).unwrap();
                op.eval(lhs, rhs)?
            },
            _ => unimplemented!(),
        })
    }

    fn fork(&self) -> Self {
        Context {
            inner: self.inner.clone(),
            f_store: HashMap::new(), // fixme clone
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

    fn def(&mut self, name: &str, params: Vec<(Pattern, Option<Expr>)>, v: Definiton) -> Result<(), RuntimeError> {
        let inner = self.fork();
        self.f_store.insert(name.into(), Box::new(move |args| {
            let mut inner = inner.fork();
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
            match &v {
                Definiton::Inline(expr) => {
                    inner.eval(&expr.as_any().downcast_ref::<Instance<Expr, ()>>().unwrap().0)
                },
                Definiton::Block(nodes) => {
                    match render(&mut Instance::new(inner), nodes) {
                        Ok(value) => Ok(Value::String(value)),
                        Err(e) => Err(e.as_any_box().downcast::<Instance<RuntimeError, ()>>().unwrap().0),
                    }
                },
            }
        }));
        Ok(())
    }

    fn apply(&self, name: &str, args: Vec<Value>) -> Result<Value, RuntimeError> {
        let Some(f) = self.f_store.get(name) else {
            return Err(RuntimeError {});
        };
        f(args)
    }
}
