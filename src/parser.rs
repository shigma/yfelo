use std::any::Any;

use crate::{error::SyntaxError, Interpreter};

pub struct Input<'i>{
    pub source: &'i str,
    pub offset: usize,
    pub close: String,
    pub lang: &'i dyn Interpreter,
}

impl<'i> Input<'i> {
    pub fn shift(&mut self, offset: usize) {
        let old_len = self.source.len();
        self.source = self.source[offset..].trim_start();
        self.offset += old_len - self.source.len();
    }

    pub fn expect_expr(&mut self) -> Result<Box<dyn Any>, SyntaxError> {
        self.lang.parse_expr(self)
    }

    pub fn expect_pattern(&mut self) -> Result<Box<dyn Any>, SyntaxError> {
        self.lang.parse_pattern(self)
    }

    pub fn expect_directive(&mut self) -> Option<(char, &str)> {
        if let Some(c @ ('#' | '/' | ':' | '@')) = self.source.chars().nth(0) {
            self.source = &self.source[1..];
            self.offset += 1;
            let pos = self.source
                .find(|c: char| !c.is_ascii_alphanumeric())
                .unwrap_or(self.source.len());
            let name = &self.source[..pos];
            self.source = &self.source[pos..];
            self.offset += pos;
            Some((c, name))
        } else {
            None
        }
    }

    pub fn expect_keyword(&mut self, keyword: &str) -> Result<(), SyntaxError> {
        if self.source.starts_with(keyword) && !self.source[keyword.len()..].starts_with(|c: char| c.is_ascii_alphanumeric()) {
            self.shift(keyword.len());
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected keyword {}", keyword),
                range: (self.offset, self.offset + 1),
            })
        }
    }

    pub fn expect_close(&mut self) -> Result<(), SyntaxError> {
        if self.source.starts_with(&self.close) {
            self.source = &self.source[self.close.len()..];
            self.offset += self.close.len();
            Ok(())
        } else {
            Err(SyntaxError {
                message: format!("expected close tag {}", self.close),
                range: (self.offset, self.offset + 1),
            })
        }
    }
}
