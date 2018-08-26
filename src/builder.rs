use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::mem;
use std::sync::Arc;
use std::thread;

use webview_sys as sys;
use crate::{ExternalInvokeFn, Webview, WebviewHandle};
use crate::conversion::convert_to_cstring;
use crate::error::WebviewError;
use crate::ffi;
use crate::storage::StringStorage;

pub struct Builder<'title, 'content, T> {
    title: Option<Cow<'title, str>>,
    content: Option<Cow<'content, str>>,
    size: Option<(usize, usize)>,
    resizable: bool,
    debug: bool,
    external_invoke: Option<ExternalInvokeFn<T>>,
    userdata: Option<T>,
    thread_check: bool,
    buffer_size: usize,
}

impl<'title, 'content, T> Builder<'title, 'content, T> {
    #[inline]
    pub fn new() -> Self {
        sys::runtime_size_check();
        Default::default()
    }

    #[inline]
    pub fn set_size(mut self, width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0);
        self.size = Some((width, height));
        self
    }

    #[inline]
    pub fn set_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    #[inline]
    pub fn set_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    #[inline]
    pub fn set_external_invoke(mut self, func: impl FnMut(&Webview<T>, &str) + 'static) -> Self {
        self.external_invoke = Some(Box::new(func));
        self
    }

    #[inline]
    pub fn set_initial_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    #[inline]
    pub fn deactivate_thread_check(mut self) -> Self {
        self.thread_check = false;
        self
    }

    #[inline(never)]
    pub fn build(self) -> Result<WebviewHandle<T>, WebviewError> {
        if self.thread_check {
            if let Some("main") = thread::current().name() {
            } else {
                return Err(WebviewError::InvalidThread);
            }
        }

        let title = self.title.ok_or(WebviewError::Build)?;
        let content = self.content.ok_or(WebviewError::Build)?;
        let (width, height) = self.size.unwrap_or((800, 600));
        let debug = self.debug;

        let inner = unsafe {
            let storage = StringStorage::new(
                convert_to_cstring(title)?,
                convert_to_cstring(content)?,
                self.buffer_size,
            );

            let mut webview: sys::webview = mem::zeroed();
            ffi::struct_webview_set_title(&mut webview, &storage.title);
            ffi::struct_webview_set_content(&mut webview, &storage.content);
            ffi::struct_webview_set_width(&mut webview, width);
            ffi::struct_webview_set_height(&mut webview, height);
            ffi::struct_webview_set_resizable(&mut webview, self.resizable);
            ffi::struct_webview_set_debug(&mut webview, self.debug);

            if self.external_invoke.is_some() {
                ffi::struct_webview_set_external_invoke_cb::<T>(&mut webview);
            }

            Webview {
                webview,
                storage,
                external_invoke: self.external_invoke,
                userdata: self.userdata,
            }
        };

        let built = WebviewHandle::new(inner);

        unsafe {
            let inner = built.webview();
            ffi::webview_init(&mut inner.webview)?;
        }

        Ok(built)
    }
}