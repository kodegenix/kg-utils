#![feature(allocator_api, try_from, test)]

#[cfg(test)]
extern crate test;
extern crate serde;


pub mod collections;

#[macro_use]
pub mod ws;









use std::ops::{Deref, DerefMut};

pub struct PrettyPrinter<'a, 'b: 'a> {
    padding: &'a str,
    fmt: &'a mut std::fmt::Formatter<'b>,
    on_newline: bool,
}

impl<'a, 'b> PrettyPrinter<'a, 'b> {
    pub fn new(fmt: &'a mut std::fmt::Formatter<'b>, padding: &'a str) -> PrettyPrinter<'a, 'b> {
        PrettyPrinter {
            padding: padding,
            fmt: fmt,
            on_newline: false,
        }
    }

    pub fn fmt(&'a mut self) -> &'a mut std::fmt::Formatter<'b> {
        self.fmt
    }
}

impl<'a, 'b> Deref for PrettyPrinter<'a, 'b> {
    type Target = std::fmt::Formatter<'b>;

    fn deref(&self) -> &Self::Target {
        self.fmt
    }
}

impl<'a, 'b> DerefMut for PrettyPrinter<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.fmt
    }
}

impl<'a, 'b> std::fmt::Write for PrettyPrinter<'a, 'b> {
    fn write_str(&mut self, mut s: &str) -> std::fmt::Result {
        while !s.is_empty() {
            if self.on_newline {
                self.fmt.write_str(self.padding)?;
            }

            let split = match s.find('\n') {
                Some(pos) => {
                    self.on_newline = true;
                    pos + 1
                }
                None => {
                    self.on_newline = false;
                    s.len()
                }
            };
            self.fmt.write_str(&s[..split])?;
            s = &s[split..];
        }

        Ok(())
    }
}


pub struct ListDisplay<'a, T: std::fmt::Display + 'a>(pub &'a [T]);

impl<'a, T: std::fmt::Display + 'a> std::fmt::Display for ListDisplay<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut i = self.0.iter().peekable();
        while let Some(e) = i.next() {
            e.fmt(f)?;
            if i.peek().is_some() {
                write!(f, ", ")?;
            }
        }
        Ok(())
    }
}
