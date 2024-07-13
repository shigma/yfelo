use std::borrow::Cow;
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
            '#' | ':' => Ok(()),
            _ => Err(SyntaxError {
                message: format!("directive '{}' should not be empty", self.name),
                range: self.range,
            }),
        }
    }

    pub fn expect_empty(&self) -> Result<(), SyntaxError> {
        match self.mark {
            '@' => Ok(()),
            _ => Err(SyntaxError {
                message: format!("directive '{}' should be empty", self.name),
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
    pub stack: Vec<(Element, TagInfo<'i>, Vec<TagInfo<'i>>)>,
}

impl<'i> Reader<'i> {
    pub fn new(input: &'i str, offset: usize, meta: &'i MetaSyntax, lang: &'i dyn Language, dirs: &'i HashMap<String, Box<dyn Directive>>) -> Self {
        Self {
            input,
            offset,
            meta,
            lang,
            dirs,
            stack: vec![(Element::new(Box::new(Instance::new(Stub))), TagInfo {
                name: "",
                range: (0, 0),
                mark: '\0',
            }, vec![])],
        }
    }

    fn push_node(&mut self, node: Node) {
        let mut element = &mut self.stack.last_mut().unwrap().0;
        if let Some(branch) = element.branches.last_mut() {
            element = branch;
        }
        element.nodes.push(node)
    }

    fn push_text(&mut self, mut text: &'i str) {
        if text.len() == 0 {
            return
        }
        let remain = text.trim_ascii_start();
        let trimmed = &text[..text.len() - remain.len()];
        if trimmed.contains('\n') {
            text = remain;
            if text.len() == 0 {
                return
            }
        }
        let remain = text.trim_ascii_end();
        let trimmed = &text[remain.len()..];
        if trimmed.contains('\n') {
            text = remain;
            if text.len() == 0 {
                return
            }
        }
        self.push_node(Node::Text(text.into()));
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
        match self.lang.parse_expr(self.input, self.offset) {
            Ok((expr, offset)) => {
                self.skip(offset);
                self.trim_start();
                Ok(expr)
            },
            Err(_) => Err(SyntaxError {
                message: format!("expect expression"),
                range: (self.offset, self.offset),
            }),
        }
    }

    pub fn parse_pattern(&mut self) -> Result<Box<dyn Pattern>, SyntaxError> {
        match self.lang.parse_pattern(self.input, self.offset) {
            Ok((expr, offset)) => {
                self.skip(offset);
                self.trim_start();
                Ok(expr)
            },
            Err(_) => Err(SyntaxError {
                message: format!("expect pattern"),
                range: (self.offset, self.offset),
            }),
        }
    }

    pub fn parse_ident(&mut self) -> Result<(&'i str, (usize, usize)), SyntaxError> {
        let pos = self.input
            .find(|c: char| !c.is_ascii_alphanumeric())
            .unwrap_or(self.input.len());
        if pos == 0 {
            return Err(SyntaxError {
                message: format!("expect identifier"),
                range: (self.offset, self.offset),
            });
        }
        let ident = &self.input[..pos];
        let range = (self.offset, self.offset + pos);
        self.skip(pos);
        self.trim_start();
        Ok((ident, range))
    }

    pub fn parse_punct(&mut self, punct: &str) -> Result<(), SyntaxError> {
        if self.input.starts_with(punct) {
            self.skip(punct.len());
            self.trim_start();
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected punctuation '{}'", punct),
                range: (self.offset, self.offset),
            })
        }
    }

    pub fn parse_keyword(&mut self, keyword: &str) -> Result<(), SyntaxError> {
        if self.input.starts_with(keyword) && !self.input[keyword.len()..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
            self.skip(keyword.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expect keyword '{}'", keyword),
                range: (self.offset, self.offset),
            })
        }
    }

    pub fn tag_close(&mut self) -> Result<(), SyntaxError> {
        if self.input.starts_with(&self.meta.right) {
            self.skip(self.meta.right.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: "invalid tag syntax: expect '}'".into(),
                range: (self.offset, self.offset),
            })
        }
    }

    fn dir_open(&mut self, info: &TagInfo) -> Result<Box<dyn Directive>, SyntaxError> {
        let (element, parent, tags) = self.stack.last().unwrap();
        let name = if info.mark == ':' {
            Cow::Owned(format!("{}:{}", parent.name, info.name))
        } else {
            Cow::Borrowed(info.name)
        };
        let Some(factory) = self.dirs.get(&*name) else {
            return Err(SyntaxError {
                message: format!("unknown directive '{}'", name),
                range: info.range,
            });
        };
        if info.mark == ':' {
            element.directive.branch(tags, info)?;
        }
        factory.open(self, &info).map_err(|mut error| {
            // directive may produce error with tag info
            if error.range != info.range {
                error.message = format!("invalid syntax for directive '{}': {}", info.name, error.message);
            }
            error
        })
    }

    fn expect_parent(&mut self, info: &TagInfo) -> Result<(), SyntaxError> {
        if self.stack.len() == 1 {
            Err(SyntaxError {
                message: format!("unmatched tag name '{}'", info.name),
                range: info.range,
            })
        } else {
            Ok(())
        }
    }

    fn directive(&mut self, mark: char) -> Result<(), SyntaxError> {
        let (name, range) = self.parse_ident().map_err(|mut error| {
            error.message = format!("invalid tag syntax: missing directive name");
            error
        })?;
        let info = TagInfo { name, range, mark };
        match mark {
            '#' => {
                let directive = self.dir_open(&info)?;
                self.stack.push((Element::new(directive), info, vec![]));
            },
            '@' => {
                let directive = self.dir_open(&info)?;
                self.push_node(Node::Element(Element::new(directive)));
            },
            ':' => {
                self.expect_parent(&info)?;
                let directive = self.dir_open(&info)?;
                let (element, _, tags) = self.stack.last_mut().unwrap();
                element.branches.push(Element::new(directive));
                tags.push(info);
            },
            '/' => {
                self.expect_parent(&info)?;
                let (mut element, parent, _) = self.stack.pop().unwrap();
                if parent.name != name {
                    return Err(SyntaxError {
                        message: format!("unmatched tag name: expect '{}', found '{}'", parent.name, name),
                        range,
                    });
                }
                element.directive.close(self, &info)?;
                self.push_node(Node::Element(element));
            },
            _ => unreachable!(),
        }
        Ok(())
    }

    pub fn run(mut self) -> Result<Vec<Node>, SyntaxError> {
        while let Some(pos) = self.input.find(&self.meta.left) {
            self.push_text(&self.input[..pos]);
            self.skip(pos + 1);
            if let Some(c @ ('#' | '/' | '@' | ':')) = self.input.chars().nth(0) {
                self.skip(1);
                self.directive(c)?;
                self.tag_close()?;
            } else {
                let expr = self.parse_expr()?;
                self.tag_close()?;
                self.push_node(Node::Expr(expr));
            }
        }
        self.push_text(self.input);
        let (element, info, _) = self.stack.pop().unwrap();
        if self.stack.len() > 0 {
            return Err(SyntaxError {
                message: format!("unmatched tag name '{}'", info.name),
                range: info.range,
            });
        }
        Ok(element.nodes)
    }
}
