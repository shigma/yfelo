use std::collections::HashMap;

use dyn_std::Instance;

use crate::builtin::Stub;
use crate::directive::Directive;
use crate::language::{Expr, Language, Pattern, SyntaxError};
use crate::{Element, MetaSyntax, Node};

#[derive(Debug, Clone, PartialEq)]
pub struct TagInfo<'i> {
    pub name: &'i str,
    pub range: (usize, usize),
    pub mark: char,
}

impl<'i> TagInfo<'i> {
    pub fn expect_children(&self) -> Result<(), SyntaxError> {
        match self.mark {
            '#' => Ok(()),
            _ => Err(SyntaxError {
                message: format!("directive {} should not be empty", self.name),
                range: self.range,
            }),
        }
    }

    pub fn expect_empty(&self) -> Result<(), SyntaxError> {
        match self.mark {
            '@' => Ok(()),
            _ => Err(SyntaxError {
                message: format!("directive {} should be empty", self.name),
                range: self.range,
            }),
        }
    }
}

pub struct Reader<'i>{
    pub lang: &'i dyn Language,
    input: &'i str,
    offset: usize,
    dirs: &'i HashMap<String, Box<dyn Directive>>,
    meta: &'i MetaSyntax<'i>,
    stack: Vec<(Element<'i>, TagInfo<'i>)>,
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
                directive: Box::new(Instance::new(Stub)),
                children: vec![],
            }, TagInfo {
                name: "",
                range: (0, 0),
                mark: '\0',
            })],
        }
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

    fn directive(&mut self, mark: char) -> Result<(), SyntaxError> {
        let pos = self.input
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(self.input.len());
        let name = &self.input[..pos];
        self.skip(pos);
        let range = (self.offset - name.len(), self.offset);
        let Some(directive) = self.dirs.get(name) else {
            return Err(SyntaxError {
                message: format!("unknown directive '{}'", name),
                range,
            });
        };
        let info = TagInfo { name, range, mark };
        match mark {
            '#' => {
                let directive = directive.open(self, &info)?;
                self.stack.push((Element {
                    directive,
                    children: vec![],
                }, info));
            },
            '/' => {
                let (mut element, parent) = self.stack.pop().unwrap();
                println!("{:?} {:?}", parent, info);
                if parent.name != name {
                    let suffix = if self.stack.len() > 0 {
                        format!(": expect '{}', found '{}'", parent.name, name)
                    } else {
                        format!(" '{}'", name)
                    };
                    return Err(SyntaxError {
                        message: format!("unmatched tag name{}", suffix),
                        range,
                    });
                }
                element.directive.close(self, &info)?;
                self.push_node(Node::Element(element));
            },
            '@' => {
                let directive = directive.open(self, &info)?;
                self.push_node(Node::Element(Element {
                    directive,
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
                self.directive(c)?;
                self.tag_close()?;
            } else {
                let expr = self.parse_expr()?;
                self.tag_close()?;
                self.push_node(Node::Expr(expr));
            }
        }
        if self.input.len() > 0 {
            self.push_node(Node::Text(self.input));
        }
        let (element, info) = self.stack.pop().unwrap();
        if self.stack.len() > 0 {
            return Err(SyntaxError {
                message: format!("unmatched tag name '{}'", info.name),
                range: info.range,
            });
        }
        Ok(element.children)
    }
}
