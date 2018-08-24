#![feature(crate_in_paths)]

#[macro_use]
extern crate bitflags;
extern crate webview_sys;

use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::mem;
use std::ops::Deref;
use std::sync::Arc;

pub use builder::{Content, WebviewBuilder};
pub use userdata::Userdata;

use crate::error::WebviewError;
use crate::ffi::{LoopResult, StringStorage};
use webview_sys::webview;

mod builder;
mod callback;
mod error;
mod ffi;
mod userdata;

/// # Thread Safety
///
/// The Webview struct is not thread-safe:
///
/// unsafe impl<T, E> !Send for Webview<T, E>
/// unsafe impl<T, E> !Sync for Webview<T, E>
///
/// To create a thread-safe handle for a Webview, call the consuming function
/// `webview_rs::Webview::dispatch_handles`.
/// This function creates two handles, one for the main thread which can be
/// used exactly like a normal Webview struct, and one which can be sent to
/// another thread.
///
/// This `ThreadHandle` can be cloned and is only able to call dispatch
/// closures, which are called in a thread-safe way.
pub struct Webview<T = ()> {
    inner: UnsafeCell<WebviewInner<T>>,
}

impl<T> From<WebviewInner<T>> for Webview<T> {
    #[inline]
    fn from(inner: WebviewInner<T>) -> Self {
        Self {
            inner: UnsafeCell::new(inner),
        }
    }
}

#[repr(C)]
struct WebviewInner<T = ()> {
    webview:         webview,
    userdata:        Option<T>,
    external_invoke: Option<Box<dyn FnMut(&Webview<T>, &str)>>,
    storage:         StringStorage,
}

impl<T> Webview<T> {
    #[inline]
    fn new(
        webview: webview,
        userdata: Option<T>,
        external_invoke: Option<Box<dyn FnMut(&Webview<T>, &str)>>,
        buffer_size: usize,
    ) -> Self {
        Self {
            inner: UnsafeCell::new(WebviewInner {
                webview,
                userdata,
                external_invoke,
                eval_buffer: String::with_capacity(buffer_size),
            }),
        }
    }

    #[inline]
    pub fn run(&self) {
        loop {
            unsafe {
                if let LoopResult::Exit = ffi::webview_loop(self.inner_webview(), true) {
                    //TODO: customize blocking
                    break;
                }
            }
        }
    }

    #[inline]
    pub fn run_with_init(&self, init: impl FnOnce()) {
        init();

        self.run();
    }

    #[inline]
    pub fn eval(&self, js: &str) -> Result<(), WebviewError> {
        let buffer = self.eval_buffer();
        buffer.clear();
        buffer.push_str(js);

        unsafe { ffi::webview_eval(self.inner_webview(), buffer)? };
        Ok(())
    }

    #[inline]
    pub fn eval_fn(&self, function: &str, args: &[&str]) -> Result<(), WebviewError> {
        let buffer = self.eval_buffer();
        buffer.clear();
        buffer.push_str(function);
        buffer.push('(');

        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            buffer.push_str(arg);
            if iter.peek().is_some() {
                buffer.push(',');
            }
        }

        buffer.push_str(");");

        unsafe { ffi::webview_eval(self.inner_webview(), buffer)? };
        Ok(())
    }

    #[inline]
    pub fn inject_css(&self, css: &str) -> Result<(), WebviewError> {
        let buffer = self.eval_buffer();
        buffer.clear();
        buffer.push_str(css);

        unsafe { ffi::webview_inject_css(self.inner_webview(), buffer)? };
        Ok(())
    }

    #[inline]
    pub fn set_title<'s>(&self, title: impl Into<Cow<'s, str>>) -> Result<(), WebviewError> {
        unsafe {
            ffi::webview_set_title(self.inner_webview(), title)?;
            Ok(())
        }
    }

    #[inline]
    pub fn set_fullscreen(&self, fullscreen: bool) {
        unsafe { ffi::webview_set_fullscreen(self.inner_webview(), fullscreen) };
    }

    #[inline]
    pub fn set_color(&self, color: impl Into<[u8; 3]>) {
        let color = color.into();
        unsafe { ffi::webview_set_color(self.inner_webview(), color[0], color[1], color[2]) };
    }

    #[inline]
    pub fn dialog(&self) {
        unimplemented!()
    }

    #[inline]
    pub fn dispatch(&self, func: impl FnMut(&Webview<T>)) {
        unsafe { ffi::webview_dispatch(self.inner_webview(), &func) };
    }

    #[inline]
    pub fn terminate(&self) {
        unsafe { ffi::webview_terminate(self.inner_webview()) };
    }

    #[inline]
    pub fn thread_handles(self) -> (MainHandle<T>, ThreadHandle<T>) {
        let inner = unsafe { mem::replace(&mut *self.inner.get(), mem::uninitialized()) };
        mem::forget(self);

        let main = Arc::new(Webview::from(inner));
        let thread = Arc::clone(&main);

        (MainHandle::new(main), ThreadHandle::new(thread))
    }

    #[inline]
    fn inner_webview(&self) -> &mut webview {
        unsafe {
            let inner = &mut *self.inner.get();
            &mut inner.webview
        }
    }

    #[inline]
    fn external_invoke(&self) -> &mut dyn FnMut(&Webview<T>, &str) {
        unsafe {
            let inner = &mut *self.inner.get();
            inner
                .external_invoke
                .as_mut()
                .expect("no external invoke callback is set")
                .as_mut()
        }
    }

    #[inline]
    fn eval_buffer(&self) -> &mut String {
        unsafe { &mut (*self.inner.get()).eval_buffer as &mut String }
    }
}

impl<T> Webview<T>
where
    T: Userdata,
{
    #[inline]
    pub fn userdata(&self) -> Option<&T> {
        unsafe {
            let inner = &*self.inner.get();
            inner.userdata.as_ref()
        }
    }

    #[inline]
    pub fn userdata_mut(&self) -> Option<&mut T> {
        unsafe {
            let inner = &mut *self.inner.get();
            inner.userdata.as_mut()
        }
    }
}

impl<T> Drop for Webview<T> {
    #[inline]
    fn drop(&mut self) {
        unsafe { ffi::webview_exit(self.inner_webview()) };
    }
}

pub struct MainHandle<T> {
    inner: Arc<Webview<T>>,
}

impl<T> MainHandle<T> {
    #[inline]
    fn new(webview: Arc<Webview<T>>) -> Self {
        Self { inner: webview }
    }

    #[inline]
    pub fn run(&self) {
        self.inner.run();
    }

    #[inline]
    pub fn run_with_init(&self, init: impl FnOnce()) {
        self.inner.run_with_init(init);
    }

    #[inline]
    pub fn eval(&self, js: &str) -> Result<(), WebviewError> {
        self.inner.eval(js)
    }

    #[inline]
    pub fn eval_fn(&self, function: &str, args: &[&str]) -> Result<(), WebviewError> {
        self.inner.eval_fn(function, args)
    }

    #[inline]
    pub fn inject_css(&self, css: &str) -> Result<(), WebviewError> {
        self.inner.inject_css(css)
    }

    #[inline]
    pub fn set_title(&self, title: &str) -> Result<(), WebviewError> {
        self.inner.set_title(title)
    }

    #[inline]
    pub fn set_fullscreen(&self, fullscreen: bool) {
        self.inner.set_fullscreen(fullscreen);
    }

    #[inline]
    pub fn set_color(&self, color: impl Into<[u8; 3]>) {
        self.inner.set_color(color);
    }

    #[inline]
    pub fn dialog(&self) {
        self.inner.dialog();
    }

    #[inline]
    pub fn dispatch(&self, func: impl FnMut(&Webview<T>)) {
        self.inner.dispatch(func);
    }

    #[inline]
    pub fn terminate(&self) {
        self.inner.terminate();
    }
}

impl<T> MainHandle<T>
where
    T: Userdata,
{
    #[inline]
    pub fn userdata(&self) -> Option<&T> {
        self.inner.userdata()
    }

    #[inline]
    pub fn userdata_mut(&self) -> Option<&mut T> {
        self.inner.userdata_mut()
    }
}

unsafe impl<T> Send for ThreadHandle<T> where T: Send {}
unsafe impl<T> Sync for ThreadHandle<T> where T: Sync {}

#[derive(Clone)]
pub struct ThreadHandle<T> {
    inner: Arc<Webview<T>>,
}

impl<T> ThreadHandle<T> {
    #[inline]
    fn new(webview: Arc<Webview<T>>) -> Self {
        Self { inner: webview }
    }

    #[inline]
    pub fn dispatch(&self, func: impl FnMut(&Webview<T>) + Send) {
        let webview = Arc::deref(&self.inner);
        unsafe { ffi::webview_dispatch(webview.inner_webview(), &func) };
    }
}
