use crate::directive::Node;
use crate::language::{Context, Language, RuntimeError};

pub struct Writer<'i> {
    pub lang: &'i dyn Language,
    output: String,
}

impl<'i> Writer<'i> {
    pub fn new(lang: &'i dyn Language) -> Self {
        Self {
            output: String::new(),
            lang,
        }
    }

    pub fn render(&mut self, nodes: &'i Vec<Node<'i>>, ctx: &dyn Context) -> Result<(), Box<dyn RuntimeError>> {
        for node in nodes {
            match node {
                Node::Text(text) => self.output += text,
                Node::Expr(expr) => self.output += ctx.eval(expr.as_ref())?.to_string()?.as_str(),
                Node::Element(element) => {
                    element.directive.render(self, &element.children, ctx)?;
                },
            }
        }
        Ok(())
    }

    pub fn run(mut self, nodes: &'i Vec<Node<'i>>, ctx: &dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        self.render(nodes, ctx)?;
        Ok(self.output)
    }
}
