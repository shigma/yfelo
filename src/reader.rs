use std::any::Any;
use std::collections::HashMap;

use crate::directive::Directive;
use crate::error::SyntaxError;
use crate::interpreter::Interpreter;
use crate::{Element, MetaSyntax, Node};

pub struct Reader<'i>{
    pub source: &'i str,
    pub lang: &'i dyn Interpreter,
    offset: usize,
    dirs: &'i HashMap<&'i str, Box<dyn Directive>>,
    meta: &'i MetaSyntax,
    stack: Vec<(Element<'i>, (usize, usize))>,
}

impl<'i> Reader<'i> {
    pub fn new(source: &'i str, meta: &'i MetaSyntax, lang: &'i dyn Interpreter, dirs: &'i HashMap<&'i str, Box<dyn Directive>>) -> Self {
        Self {
            source,
            offset: 0,
            meta,
            lang,
            dirs,
            stack: vec![(Element {
                name: "",
                meta: Box::new(()),
                children: Some(vec![]),
            }, (0, 0))],
        }
    }

    fn push_layer(&mut self, element: Element<'i>, range: (usize, usize)) {
        self.stack.push((element, range));
    }

    fn push_node(&mut self, node: Node<'i>) {
       self.stack.last_mut().unwrap().0.children.as_mut().unwrap().push(node)
    }

    pub fn skip(&mut self, offset: usize) {
        self.source = &self.source[offset..];
        self.offset += offset;
    }

    pub fn trim_start(&mut self) {
        let old_len = self.source.len();
        self.source = self.source.trim_start();
        self.offset += old_len - self.source.len();
    }

    pub fn parse_expr(&mut self) -> Result<Box<dyn Any>, SyntaxError> {
        self.lang.parse_expr(self)
    }

    pub fn parse_pattern(&mut self) -> Result<Box<dyn Any>, SyntaxError> {
        self.lang.parse_pattern(self)
    }

    pub fn parse_keyword(&mut self, keyword: &str) -> Result<(), SyntaxError> {
        if self.source.starts_with(keyword) && !self.source[keyword.len()..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
            self.skip(keyword.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected keyword {}", keyword),
                range: (self.offset, self.offset + 1),
            })
        }
    }

    pub fn tag_close(&mut self) -> Result<(), SyntaxError> {
        if self.source.starts_with(&self.meta.right) {
            self.skip(self.meta.right.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected close tag {}", self.meta.right),
                range: (self.offset, self.offset + 1),
            })
        }
    }

    fn tag_open(&mut self, c: char) -> Result<(), SyntaxError> {
        let pos = self.source
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(self.source.len());
        let name = &self.source[..pos];
        self.skip(pos);
        let range = (self.offset - name.len(), self.offset);
        let Some(dir) = self.dirs.get(name) else {
            return Err(SyntaxError {
                message: format!("unknown directive '{}'", name),
                range,
            });
        };
        match c {
            '#' => {
                let meta = dir.parse(self)?;
                self.push_layer(Element {
                    name,
                    meta,
                    children: Some(vec![]),
                }, range);
            },
            '/' => {
                let element = self.stack.pop().unwrap().0;
                if element.name != name {
                    return Err(SyntaxError {
                        message: format!("unmatched tag name"),
                        range,
                    });
                }
                self.push_node(Node::Element(element));
            },
            '@' => {
                let meta = dir.parse(self)?;
                self.push_node(Node::Element(Element {
                    name,
                    meta,
                    children: None,
                }));
            },
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn parse(mut self) -> Result<Vec<Node<'i>>, SyntaxError> {
        while let Some(pos) = self.source.find(&self.meta.left) {
            if pos > 0 {
                self.push_node(Node::Text(&self.source[..pos]));
            }
            self.skip(pos + 1);
            if let Some(c @ ('#' | '/' | '@')) = self.source.chars().nth(0) {
                self.skip(1);
                self.tag_open(c)?;
            } else {
                let expr = self.parse_expr()?;
                self.push_node(Node::Expr(expr));
            }
        }
        if self.source.len() > 0 {
            self.push_node(Node::Text(self.source));
        }
        if self.stack.len() > 1 {
            return Err(SyntaxError {
                message: format!("unmatched tag name"),
                range: self.stack.last().unwrap().1,
            });
        }
        Ok(self.stack.pop().unwrap().0.children.unwrap())
    }
}
