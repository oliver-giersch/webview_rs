#![feature(crate_in_paths)]

#[macro_use]
extern crate bitflags;
extern crate webview_sys;

use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::sync::{Arc, Weak};

pub use crate::builder::Builder;
pub use crate::content::Content;
pub use crate::ffi::{Dialog, Flags};

use webview_sys as sys;
use crate::error::WebviewError;
use crate::storage::StringStorage;

mod builder;
mod callback;
mod content;
mod conversion;
mod error;
mod ffi;
mod storage;

/// Doc
pub fn webview<'title, 'content>(
    title: impl Into<Cow<'title, str>>,
    content: impl Into<Cow<'content, str>>,
    width: usize,
    height: usize,
    resizable: bool,
) -> Result<(), WebviewError> {
    unsafe {
        ffi::webview_simple(title, content, width, height, resizable)
    }
}

#[repr(C)]
pub struct Webview<'invoke, T> {
    webview: sys::webview,
    storage: StringStorage,
    external_invoke: Option<Box<FnMut(&mut Webview<'invoke, T>, &str) + 'invoke>>,
    userdata: Option<T>,
}

impl<'invoke, T> Webview<'invoke, T> {
    #[inline]
    pub fn eval(&mut self, js: &str) -> Result<(), WebviewError> {
        self.storage.eval_buffer.clear();
        self.storage.eval_buffer.push_str(js);

        unsafe { ffi::webview_eval(&mut self.webview, self.storage.nul_terminated_buffer())? };
        Ok(())
    }

    #[inline]
    pub fn eval_fn(&mut self, function: &str, args: &[&str]) -> Result<(), WebviewError> {
        self.storage.eval_buffer.clear();
        self.storage.eval_buffer.push_str(function);
        self.storage.eval_buffer.push('(');

        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            self.storage.eval_buffer.push_str(arg);
            if iter.peek().is_some() {
                self.storage.eval_buffer.push(',');
            }
        }
        self.storage.eval_buffer.push_str(");");

        unsafe { ffi::webview_eval(&mut self.webview, self.storage.nul_terminated_buffer())? };
        Ok(())
    }

    #[inline]
    pub fn inject_css(&mut self, css: &str) -> Result<(), WebviewError> {
        self.storage.eval_buffer.clear();
        self.storage.eval_buffer.push_str(css);

        unsafe { ffi::webview_inject_css(&mut self.webview, self.storage.nul_terminated_buffer())? };
        Ok(())
    }

    #[inline]
    pub fn set_title<'title>(
        &mut self,
        title: impl Into<Cow<'title, str>>,
    ) -> Result<(), WebviewError> {
        unsafe { ffi::webview_set_title(&mut self.webview, title) }
    }

    #[inline]
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe { ffi::webview_set_fullscreen(&mut self.webview, fullscreen) };
    }

    #[inline]
    pub fn set_color(&mut self, color: impl Into<[u8; 4]>) {
        let color = color.into();
        unsafe {
            ffi::webview_set_color(&mut self.webview, color[0], color[1], color[2], color[3])
        };
    }

    #[inline]
    pub fn dialog<'title, 'arg>(
        &mut self,
        dialog: Dialog,
        flags: Flags,
        title: impl Into<Cow<'title, str>>,
        arg: impl Into<Cow<'arg, str>>,
        result_buffer: &mut [u8],
    ) -> Result<(), WebviewError> {
        unsafe { ffi::webview_dialog(&mut self.webview, dialog, flags, title, arg, result_buffer) }
    }

    #[inline]
    pub fn dispatch(&mut self, func: impl FnMut(&mut Webview<'invoke, T>)) {
        //TODO: Check if it works without box as well
        let func = Box::leak(Box::new(func));
        unsafe { ffi::webview_dispatch(&mut self.webview, func) };
    }

    #[inline]
    pub fn terminate(&mut self) {
        unsafe { ffi::webview_terminate(&mut self.webview) };
    }

    #[inline]
    pub fn userdata(&self) -> Option<&T> {
        self.userdata.as_ref()
    }
}

impl<'invoke, T> Drop for Webview<'invoke, T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::webview_exit(&mut self.webview) };
    }
}

/// unsafe impl<T> !Send for WebviewHandle<T> {}
/// unsafe impl<T> !Sync for WebviewHandle<T> {}

pub struct WebviewHandle<'invoke, T = ()> {
    inner: Arc<UnsafeCell<Webview<'invoke, T>>>,
}

impl<'invoke, T> WebviewHandle<'invoke, T> {
    #[inline]
    fn new(inner: Webview<'invoke, T>) -> Self {
        Self {
            inner: Arc::new(UnsafeCell::new(inner)),
        }
    }

    #[inline]
    pub fn run(&mut self, blocking: bool) {
        use ffi::LoopResult::Continue;
        let inner = unsafe { &mut *self.inner.get() };
        while let Continue = unsafe { ffi::webview_loop(&mut inner.webview, blocking) } {}
    }

    #[inline]
    pub fn eval(&mut self, js: &str) -> Result<(), WebviewError> {
        self.webview().eval(js)
    }

    #[inline]
    pub fn eval_fn(&mut self, function: &str, args: &[&str]) -> Result<(), WebviewError> {
        self.webview().eval_fn(function, args)
    }

    #[inline]
    pub fn inject_css(&mut self, css: &str) -> Result<(), WebviewError> {
        self.webview().inject_css(css)
    }

    #[inline]
    pub fn set_title<'title>(
        &mut self,
        title: impl Into<Cow<'title, str>>,
    ) -> Result<(), WebviewError> {
        self.webview().set_title(title)
    }

    #[inline]
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.webview().set_fullscreen(fullscreen);
    }

    #[inline]
    pub fn set_color(&mut self, color: impl Into<[u8; 4]>) {
        self.webview().set_color(color);
    }

    #[inline]
    pub fn dialog<'title, 'arg>(
        &mut self,
        dialog: Dialog,
        flags: Flags,
        title: impl Into<Cow<'title, str>>,
        arg: impl Into<Cow<'arg, str>>,
        result_buffer: &mut [u8],
    ) -> Result<(), WebviewError> {
        self.webview().dialog(dialog, flags, title, arg, result_buffer)
    }

    #[inline]
    pub fn dispatch(&mut self, func: impl FnMut(&mut Webview<'invoke, T>)) {
        self.webview().dispatch(func);
    }

    #[inline]
    pub fn terminate(&mut self) {
        self.webview().terminate();
    }

    #[inline]
    pub fn userdata(&mut self) -> Option<&mut T> {
        self.webview().userdata.as_mut()
    }

    #[inline]
    pub fn thread_handle(&self) -> ThreadHandle<'invoke, T> {
        ThreadHandle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    #[inline]
    fn webview(&mut self) -> &mut Webview<'invoke, T> {
        unsafe { &mut *self.inner.get() }
    }
}

unsafe impl<'invoke, T> Send for ThreadHandle<'invoke, T> where T: Send {}
unsafe impl<'invoke, T> Sync for ThreadHandle<'invoke, T> where T: Sync {}

#[derive(Clone)]
pub struct ThreadHandle<'invoke, T> {
    inner: Weak<UnsafeCell<Webview<'invoke, T>>>,
}

impl<'invoke, T> ThreadHandle<'invoke, T> {
    #[inline]
    pub fn try_dispatch(
        &self,
        func: impl FnMut(&mut Webview<'invoke, T>) + Send,
    ) -> Result<(), WebviewError> {
        match self.inner.upgrade() {
            Some(ref cell) => {
                let webview = unsafe { &mut *cell.get() };
                webview.dispatch(func);
                Ok(())
            }
            None => Err(WebviewError::DispatchFailed),
        }
    }
}
