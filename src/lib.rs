//! TODO: Crate doc
//! 

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

//TODOS
//TODO: Make builder more ergonomic
//TODO: Make userdata more ergonomic

/// Outer wrapper
/// 
/// A wrapper struct around the C library struct and the associated string buffers.
/// The `ext` contains further (optional) Rust specific representations for userdata
/// and the external invoke callback for calling Rust code from Javascript.
#[repr(C)]
pub struct WebviewWrapper<'invoke, T> {
    inner: Webview,
    ext: Extension<'invoke, T>
}

/// Type alias for a boxed internal invoke callback.
type ExternalInvokeFnBox<'invoke, T> = Box<FnMut(&mut Webview, &mut T, &str) + 'invoke>;

#[repr(C)]
struct Extension<'invoke, T> {
    external_invoke: Option<ExternalInvokeFnBox<'invoke, T>>,
    userdata: T,
}

#[repr(C)]
pub struct Webview {
    webview: sys::webview,
    storage: StringStorage,
}

impl Webview {
    /// Evaluate a string as JavaScript code and execute it.
    ///
    ///
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
    pub fn dispatch<T>(&mut self, func: impl FnMut(&mut Webview, &mut T)) {
        unsafe { ffi::webview_dispatch(&mut self.webview, &func) };
    }

    #[inline]
    pub fn terminate(&mut self) {
        unsafe { ffi::webview_terminate(&mut self.webview) };
    }
}

impl Drop for Webview {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::webview_exit(&mut self.webview) };
    }
}

/// unsafe impl<T> !Send for WebviewHandle<T> {}
/// unsafe impl<T> !Sync for WebviewHandle<T> {}

pub struct WebviewHandle<'invoke, T = ()> {
    inner: Arc<UnsafeCell<WebviewWrapper<'invoke, T>>>,
}

impl<'invoke, T> WebviewHandle<'invoke, T> {
    #[inline]
    fn new(inner: WebviewWrapper<'invoke, T>) -> Self {
        Self {
            inner: Arc::new(UnsafeCell::new(inner)),
        }
    }

    #[inline]
    pub fn run(&mut self, blocking: bool) {
        use ffi::LoopResult::Continue;
        while let Continue = unsafe { ffi::webview_loop(&mut self.webview_mut().webview, blocking) } {}
    }

    #[inline]
    pub fn eval(&mut self, js: &str) -> Result<(), WebviewError> {
        self.webview_mut().eval(js)
    }

    #[inline]
    pub fn eval_fn(&mut self, function: &str, args: &[&str]) -> Result<(), WebviewError> {
        self.webview_mut().eval_fn(function, args)
    }

    #[inline]
    pub fn inject_css(&mut self, css: &str) -> Result<(), WebviewError> {
        self.webview_mut().inject_css(css)
    }

    #[inline]
    pub fn set_title<'title>(
        &mut self,
        title: impl Into<Cow<'title, str>>,
    ) -> Result<(), WebviewError> {
        self.webview_mut().set_title(title)
    }

    #[inline]
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        self.webview_mut().set_fullscreen(fullscreen);
    }

    #[inline]
    pub fn set_color(&mut self, color: impl Into<[u8; 4]>) {
        self.webview_mut().set_color(color);
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
        self.webview_mut().dialog(dialog, flags, title, arg, result_buffer)
    }

    #[inline]
    pub fn dispatch(&mut self, func: impl FnMut(&mut Webview, &mut T)) {
        self.webview_mut().dispatch(func);
    }

    #[inline]
    pub fn terminate(&mut self) {
        self.webview_mut().terminate();
    }

    #[inline]
    pub fn userdata(&self) -> &T {
        &self.extension().userdata
    }

    #[inline]
    pub fn userdata_mut(&mut self) -> &T {
        &mut self.extension_mut().userdata
    }

    #[inline]
    pub fn thread_handle(&self) -> ThreadHandle<'invoke, T> {
        ThreadHandle {
            inner: Arc::downgrade(&self.inner),
        }
    }

    #[inline]
    fn webview(&self) -> &Webview {
        unsafe { &(*self.inner.get()).inner }
    }

    #[inline]
    fn webview_mut(&mut self) -> &mut Webview {
        unsafe { &mut (*self.inner.get()).inner }
    }

    #[inline]
    fn extension(&self) -> &Extension<'invoke, T> {
        unsafe { &mut (*self.inner.get()).ext }
    }

    #[inline]
    fn extension_mut(&mut self) -> &mut Extension<'invoke, T> {
        unsafe { &mut (*self.inner.get()).ext }
    }
}

unsafe impl<'invoke, T> Send for ThreadHandle<'invoke, T> where T: Send {}
unsafe impl<'invoke, T> Sync for ThreadHandle<'invoke, T> where T: Sync {}

#[derive(Clone)]
pub struct ThreadHandle<'invoke, T> {
    inner: Weak<UnsafeCell<WebviewWrapper<'invoke, T>>>,
}

impl<'invoke, T> ThreadHandle<'invoke, T> {
    #[inline]
    pub fn try_dispatch(
        &self,
        func: impl FnMut(&mut Webview, &mut T) + Send,
    ) -> Result<(), WebviewError> {
        match self.inner.upgrade() {
            Some(ref cell) => {
                let webview = unsafe { &mut (*cell.get()).inner };
                webview.dispatch(func);
                Ok(())
            }
            None => Err(WebviewError::DispatchFailed),
        }
    }
}
