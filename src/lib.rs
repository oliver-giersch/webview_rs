#![feature(crate_in_paths)]

#[macro_use]
extern crate bitflags;
extern crate webview_sys;

use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::sync::{Arc, Weak};

use webview_sys as sys;
use crate::error::WebviewError;
use crate::storage::StringStorage;

mod builder;
mod callback;
mod conversion;
mod error;
mod ffi;
mod storage;

type ExternalInvokeFn<'invoke, T> = Box<dyn FnMut(&mut Webview<T>, &str) + 'invoke>;

#[repr(C)]
pub struct Webview<'invoke, T> {
    webview: sys::webview,
    storage: StringStorage,
    external_invoke: Option<ExternalInvokeFn<'invoke, T>>,
    userdata: Option<T>,
}

impl<'invoke, T> Webview<'invoke, T> {
    #[inline]
    pub fn eval(&mut self, js: &str) -> Result<(), WebviewError> {
        self.storage.eval_buffer.clear();
        self.storage.eval_buffer.push_str(js);

        unsafe {
            ffi::webview_eval(&mut self.webview, self.storage.nul_terminated_buffer())?
        };
        Ok(())
    }

    pub fn eval_unbuffered(&mut self, js: &str) -> Result<(), WebviewError> {
        unsafe { ffi::webview_eval(&mut self.webview, js.as_bytes()) };
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

        unsafe {
            ffi::webview_eval(&mut self.webview, self.storage.nul_terminated_buffer())?
        };
        Ok(())
    }

    #[inline]
    pub fn inject_css(&mut self, css: &str) -> Result<(), WebviewError> {
        self.storage.eval_buffer.clear();
        self.storage.eval_buffer.push_str(css);

        unsafe {
            ffi::webview_eval(&mut self.webview, self.storage.nul_terminated_buffer())?
        };
        Ok(())
    }

    #[inline]
    pub fn set_title<'title>(&mut self, title: impl Into<Cow<'title, str>>) -> Result<(), WebviewError> {
        unimplemented!()
    }

    #[inline]
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        unimplemented!()
    }

    #[inline]
    pub fn set_color(&mut self, color: impl Into<[u8; 4]>) {
        unimplemented!()
    }

    #[inline]
    pub fn dialog(&mut self) {
        //TODO: Missing arguments
        unimplemented!()
    }

    #[inline]
    pub fn dispatch(&mut self, func: impl FnMut(&mut Webview<'invoke, T>)) {
        let func = Box::leak(Box::new(func));
        unsafe { ffi::webview_dispatch(&mut self.webview, func) };
    }

    #[inline]
    pub fn terminate(&mut self) {
        unsafe { ffi::webview_terminate(&mut self.webview) };
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

pub struct WebviewHandle<'invoke, T> {
    inner: Arc<UnsafeCell<Webview<'invoke, T>>>
}

impl<'invoke, T> WebviewHandle<'invoke, T> {
    #[inline]
    fn new(inner: Webview<'invoke, T>) -> Self {
        Self {
            inner: Arc::new(UnsafeCell::new(inner)),
        }
    }

    #[inline]
    pub fn run(&self, blocking: bool) {
        use ffi::LoopResult::Continue;
        let inner = unsafe { &mut *self.inner.get() };
        while let Continue = unsafe { ffi::webview_loop(&mut inner.webview, blocking) } {}
    }

    #[inline]
    pub fn eval(&self, js: &str) -> Result<(), WebviewError> {
        self.webview().eval(js)
    }

    //...

    #[inline]
    pub fn thread_handle(&self) -> ThreadHandle<'invoke, T> {
        ThreadHandle {
            inner: Arc::downgrade(&self.inner)
        }
    }

    #[inline]
    fn webview(&self) -> &mut Webview<'invoke, T> {
        unsafe { &mut *self.inner.get() }
    }
}

unsafe impl<'invoke, T> Send for ThreadHandle<'invoke, T> where T: Send {}
unsafe impl<'invoke, T> Sync for ThreadHandle<'invoke, T> where T: Sync {}

#[derive(Clone)]
pub struct ThreadHandle<'invoke, T> {
    inner: Weak<UnsafeCell<Webview<'invoke, T>>>
}

impl<'invoke, T> ThreadHandle<'invoke, T> {
    #[inline]
    pub fn try_dispatch(&self, func: impl FnMut(&mut Webview<'invoke, T>) + Send) -> Result<(), WebviewError> {
        match self.inner.upgrade() {
            Some(ref cell) => {
                let webview = unsafe { &mut *cell.get() };
                webview.dispatch(func);
                Ok(())
            },
            None => Err(WebviewError::DispatchFailed)
        }
    }
}

