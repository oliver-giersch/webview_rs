use std::borrow::Cow;
use std::ffi::{CStr, CString};
use std::fmt::Write;

pub enum Arg<'s> {
    Int(usize),
    Float(f64),
    Str(&'s str),
}

#[derive(Debug, Clone)]
pub struct StringBuffers {
    pub title:   CString,
    pub content: CString,
    pub buffer:  EvalBuffer,
}

impl StringBuffers {
    #[inline]
    pub fn new<'title, 'content>(
        title: impl Into<Cow<'title, CStr>>,
        content: impl Into<Cow<'content, CStr>>,
        buffer_size: usize,
    ) -> Self {
        Self {
            title:   title.into().into_owned(),
            content: content.into().into_owned(),
            buffer:  EvalBuffer::new(buffer_size),
        }
    }
}

#[derive(Debug, Clone)]
pub struct EvalBuffer {
    inner: String,
}

impl EvalBuffer {
    #[inline]
    fn new(buffer_size: usize) -> Self {
        Self {
            inner: String::with_capacity(buffer_size),
        }
    }

    #[inline]
    pub fn push_str(&mut self, string: &str) {
        self.inner.push_str(string);
    }

    #[inline]
    pub fn push(&mut self, ch: char) {
        self.inner.push(ch);
    }

    #[inline]
    pub fn push_arg<'s>(&mut self, arg: &Arg<'s>) {
        match arg {
            Arg::Int(val) => write!(&mut self.inner, "{}", val).unwrap(),
            Arg::Float(val) => write!(&mut self.inner, "{}", val).unwrap(),
            Arg::Str(ref string) => write!(&mut self.inner, "'{}'", string).unwrap(),
        };
    }

    #[inline]
    pub fn clear(&mut self) {
        self.inner.clear();
    }

    #[inline]
    pub fn nul_terminated(&mut self) -> &[u8] {
        if !self.inner.is_empty() && !self.inner.ends_with('\0') {
            self.inner.push('\0');
        }

        self.inner.as_bytes()
    }
}
