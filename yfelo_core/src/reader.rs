use std::collections::HashMap;

use crate::builtin::StubMeta;
use crate::directive::Directive;
use crate::language::{Expr, Language, Pattern, SyntaxError};
use crate::{Element, MetaSyntax, Node};

pub struct Reader<'i>{
    pub lang: &'i dyn Language,
    input: &'i str,
    offset: usize,
    dirs: &'i HashMap<String, Box<dyn Directive>>,
    meta: &'i MetaSyntax<'i>,
    stack: Vec<(Element<'i>, (usize, usize))>,
}

impl<'i> Reader<'i> {
    pub fn new(source: &'i str, meta: &'i MetaSyntax, lang: &'i dyn Language, dirs: &'i HashMap<String, Box<dyn Directive>>) -> Self {
        Self {
            input: source,
            offset: 0,
            meta,
            lang,
            dirs,
            stack: vec![(Element {
                name: "",
                meta: Box::new(StubMeta),
                children: vec![],
            }, (0, 0))],
        }
    }

    fn push_layer(&mut self, element: Element<'i>, range: (usize, usize)) {
        self.stack.push((element, range));
    }

    fn push_node(&mut self, node: Node<'i>) {
       self.stack.last_mut().unwrap().0.children.push(node)
    }

    pub fn skip(&mut self, offset: usize) {
        self.input = &self.input[offset..];
        self.offset += offset;
    }

    pub fn trim_start(&mut self) {
        let old_len = self.input.len();
        self.input = self.input.trim_start();
        self.offset += old_len - self.input.len();
    }

    pub fn parse_expr(&mut self) -> Result<Box<dyn Expr>, SyntaxError> {
        let (expr, offset) = self.lang.parse_expr(self.input)?;
        self.skip(offset);
        self.trim_start();
        Ok(expr)
    }

    pub fn parse_pattern(&mut self) -> Result<Box<dyn Pattern>, SyntaxError> {
        let (expr, offset) = self.lang.parse_pattern(self.input)?;
        self.skip(offset);
        self.trim_start();
        Ok(expr)
    }

    pub fn parse_punct(&mut self, punct: &str) -> Result<(), SyntaxError> {
        if self.input.starts_with(punct) {
            self.skip(punct.len());
            self.trim_start();
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected punctuation {}", punct),
                range: (self.offset, self.offset + 1),
            })
        }
    }

    pub fn parse_keyword(&mut self, keyword: &str) -> Result<(), SyntaxError> {
        if self.input.starts_with(keyword) && !self.input[keyword.len()..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
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
        if self.input.starts_with(&self.meta.right) {
            self.skip(self.meta.right.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("invalid tag syntax"),
                range: (self.offset, self.offset + 1),
            })
        }
    }

    fn tag_open(&mut self, c: char) -> Result<(), SyntaxError> {
        let pos = self.input
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(self.input.len());
        let name = &self.input[..pos];
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
                let meta = dir.parse_open(self)?;
                self.push_layer(Element {
                    name,
                    meta,
                    children: vec![],
                }, range);
            },
            '/' => {
                self.tag_close()?;
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
                let meta = dir.parse_open(self)?;
                self.push_node(Node::Element(Element {
                    name,
                    meta,
                    children: vec![],
                }));
            },
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn run(mut self) -> Result<Vec<Node<'i>>, SyntaxError> {
        while let Some(pos) = self.input.find(&self.meta.left) {
            if pos > 0 {
                self.push_node(Node::Text(&self.input[..pos]));
            }
            self.skip(pos + 1);
            if let Some(c @ ('#' | '/' | '@')) = self.input.chars().nth(0) {
                self.skip(1);
                self.tag_open(c)?;
            } else {
                let expr = self.parse_expr()?;
                self.tag_close()?;
                self.push_node(Node::Expr(expr));
            }
        }
        if self.input.len() > 0 {
            self.push_node(Node::Text(self.input));
        }
        if self.stack.len() > 1 {
            return Err(SyntaxError {
                message: format!("unmatched tag name"),
                range: self.stack.last().unwrap().1,
            });
        }
        Ok(self.stack.pop().unwrap().0.children)
    }
}
