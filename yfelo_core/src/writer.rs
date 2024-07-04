use crate::directive::Node;
use crate::language::{Context, RuntimeError};

pub fn render(ctx: &mut dyn Context, nodes: &[Node]) -> Result<String, Box<dyn RuntimeError>> {
    let mut output = String::new();
    for node in nodes {
        match node {
            Node::Text(text) => output += text,
            Node::Expr(expr) => {
                output += ctx.eval(expr)?.to_string()?.as_str()
            },
            Node::Element(element) => {
                output += &element.directive.render(ctx, &element.nodes, &element.branches)?;
            },
        }
    }
    Ok(output)
}
