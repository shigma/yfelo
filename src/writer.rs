use std::any::Any;
use std::collections::HashMap;

use crate::directive::Directive;
use crate::interpreter::{Context, Interpreter};
use crate::Node;

pub struct Writer<'i> {
    pub lang: &'i dyn Interpreter,
    dirs: &'i HashMap<&'i str, Box<dyn Directive>>,
    output: String,
}

impl<'i> Writer<'i> {
    pub fn new(lang: &'i dyn Interpreter, dirs: &'i HashMap<&'i str, Box<dyn Directive>>) -> Self {
        Self {
            output: String::new(),
            lang,
            dirs,
        }
    }

    pub fn render_node(&mut self, node: &'i Node<'i>, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        match node {
            Node::Text(text) => self.output += text,
            Node::Expr(expr) => self.output += ctx.eval(expr)?.to_string()?.as_str(),
            Node::Element(element) => {
                let dir = self.dirs.get(element.name).unwrap();
                dir.render(self, element, ctx)?;
            },
        }
        Ok(())
    }

    pub fn render_layer(&mut self, nodes: &'i Vec<Node<'i>>, ctx: &dyn Context) -> Result<(), Box<dyn Any>> {
        for node in nodes {
            self.render_node(node, ctx)?;
        }
        Ok(())
    }
}
