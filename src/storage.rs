use std::borrow::Cow;
use std::ffi::{CStr, CString};

#[derive(Debug, Clone)]
#[repr(C)]
pub struct StringStorage {
    pub title: CString,
    pub content: CString,
    /// The eval buffer needs to be a string so that push/clear operations can be used.
    /// Nul terminator has to be manually added.
    pub eval_buffer: String,
}

impl StringStorage {
    #[inline]
    pub fn new<'title, 'content>(
        title: impl Into<Cow<'title, CStr>>,
        content: impl Into<Cow<'content, CStr>>,
        buffer_size: usize,
    ) -> Self {
        Self {
            title: title.into().into_owned(),
            content: content.into().into_owned(),
            eval_buffer: String::with_capacity(buffer_size),
        }
    }

    #[inline]
    pub fn nul_terminated_buffer(&mut self) -> &[u8] {
        if !self.eval_buffer.is_empty() && !self.eval_buffer.ends_with('\0') {
            self.eval_buffer.push('\0');
        }

        self.eval_buffer.as_bytes()
    }
}
