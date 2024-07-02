use crate::directive::Node;
use crate::language::{Context, Language, RuntimeError};

pub struct Writer<'i> {
    pub lang: &'i dyn Language,
}

impl<'i> Writer<'i> {
    pub fn new(lang: &'i dyn Language) -> Self {
        Self {
            lang,
        }
    }

    pub fn render(&self, nodes: &'i Vec<Node<'i>>, ctx: &mut dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        let mut output = String::new();
        for node in nodes {
            match node {
                Node::Text(text) => output += text,
                Node::Expr(expr) => output += ctx.eval(expr)?.to_string()?.as_str(),
                Node::Element(element) => {
                    output += &element.directive.render(self, &element.children, ctx)?;
                },
            }
        }
        Ok(output)
    }

    pub fn run(self, nodes: &'i Vec<Node<'i>>, ctx: &mut dyn Context) -> Result<String, Box<dyn RuntimeError>> {
        self.render(nodes, ctx)
    }
}
