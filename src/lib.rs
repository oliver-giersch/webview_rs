//! Webview RS
//!
//! This crate is a library which wraps around the cross-platform GUI C library webview.
//! The library uses ... under Linux, ... under Windows and ... under MacOS, which are
//! required dependencies for compiling the crate.
//!
//! The webview library has the following features:
//!     - build a HTML frontend with CSS and Javascript and a native backend
//!     - evaluate JS or inject CSS at runtime
//!     - invoke native (e.g. Rust) code from Javascript
//!     - display arbitrary websites
//!     - dispatch Javascript code for runtime evaluation safely from
//!       multiple threads
//!
//! The invocation and evaluation functionality is somewhat limited (to simple function calls
//! with comparatively simple arguments).


#![feature(crate_in_paths)]

#[macro_use]
extern crate bitflags;
extern crate webview_sys;

use std::borrow::Cow;
use std::cell::UnsafeCell;
use std::sync::{Arc, Weak};

pub use crate::builder::Builder;
pub use crate::content::Content;
pub use crate::eval::{Arg, EvalBuffer, StringBuffers};
pub use crate::ffi::{Dialog, Flags};

use crate::error::WebviewError;
use webview_sys as sys;

mod builder;
mod callback;
mod content;
mod conversion;
mod error;
mod eval;
mod ffi;

/// Type alias for a boxed internal invoke callback.
type ExternalInvokeFnBox<'invoke, T> = Box<FnMut(&mut Webview, &mut T, &str) + 'invoke>;
type Result = std::result::Result<(), WebviewError>;

/// Outer wrapper
///
/// A wrapper struct around the inner wrapper. The `ext` contains further
/// (optional) Rust specific representations for userdata and the external
/// invoke callback for calling Rust code from Javascript.
#[repr(C)]
pub struct WebviewWrapper<'invoke, T> {
    inner: Webview,
    ext:   Extension<'invoke, T>,
}

/// Inner wrapper
///
/// A wrapper struct for the actual C library struct and the associated string
/// buffers
#[repr(C)]
pub struct Webview {
    webview: sys::webview,
    buffers: StringBuffers,
}

struct Extension<'invoke, T> {
    external_invoke: Option<ExternalInvokeFnBox<'invoke, T>>,
    userdata:        T,
}

impl Webview {
    /// Evaluate a string as Javascript code and execute it.
    ///
    /// #Errors
    ///
    /// Attempting to evaluate invalid JS code does *not* necessarily lead
    /// to an error!
    /// Set the debug attribute in the `Builder` and watch for platform
    /// specific error messages in the console.
    #[inline]
    pub fn eval(&mut self, js: &str) -> Result {
        self.buffers.buffer.clear();

        self.buffers.buffer.clear();
        self.buffers.buffer.push_str(js);

        unsafe { ffi::webview_eval(&mut self.webview, self.buffers.buffer.nul_terminated())? };
        Ok(())
    }

    /// Evaluate a single Javascript function with simple arguments
    #[inline]
    pub fn eval_fn<'s>(&mut self, function: &str, args: &[Arg<'s>]) -> Result {
        self.buffers.buffer.clear();
        self.buffers.buffer.push_str(function);
        self.buffers.buffer.push('(');

        let mut iter = args.iter().peekable();
        while let Some(arg) = iter.next() {
            self.buffers.buffer.push_arg(arg);
            if iter.peek().is_some() {
                self.buffers.buffer.push(',');
            }
        }
        self.buffers.buffer.push_str(");");

        unsafe { ffi::webview_eval(&mut self.webview, self.buffers.buffer.nul_terminated())? };
        Ok(())
    }

    /// Inject CSS in string format at runtime
    ///
    ///
    #[inline]
    pub fn inject_css(&mut self, css: &str) -> Result {
        self.buffers.buffer.clear();
        self.buffers.buffer.push_str(css);

        unsafe {
            ffi::webview_inject_css(&mut self.webview, self.buffers.buffer.nul_terminated())?
        };
        Ok(())
    }

    /// Set the webview window title
    #[inline]
    pub fn set_title<'title>(&mut self, title: impl Into<Cow<'title, str>>) -> Result {
        unsafe { ffi::webview_set_title(&mut self.webview, title) }
    }

    /// Set the webview window to fullscreen/windowed
    #[inline]
    pub fn set_fullscreen(&mut self, fullscreen: bool) {
        unsafe { ffi::webview_set_fullscreen(&mut self.webview, fullscreen) };
    }

    /// Set the webview window color (paints over any content)
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
    ) -> Result {
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

pub struct WebviewHandle<'invoke, T> {
    inner: Arc<UnsafeCell<WebviewWrapper<'invoke, T>>>,
}

impl<'invoke, T> WebviewHandle<'invoke, T> {
    #[inline]
    fn new(inner: WebviewWrapper<'invoke, T>) -> Self {
        Self {
            inner: Arc::new(UnsafeCell::new(inner)),
        }
    }

    /// Start the webview event loop
    ///
    /// The loop iterates until either the webview window is closed or a
    /// call to `terminate` is made.
    #[inline]
    pub fn run(&mut self, blocking: bool) {
        use ffi::LoopResult::Exit;
        loop {
            let result = unsafe { ffi::webview_loop(&mut self.webview_mut().webview, blocking) };
            if let Exit = result {
                break;
            }
        }
    }

    #[inline]
    pub fn eval(&mut self, js: &str) -> Result {
        self.webview_mut().eval(js)
    }

    #[inline]
    pub fn eval_fn<'s>(&mut self, function: &str, args: &[Arg<'s>]) -> Result {
        self.webview_mut().eval_fn(function, args)
    }

    #[inline]
    pub fn inject_css(&mut self, css: &str) -> Result {
        self.webview_mut().inject_css(css)
    }

    #[inline]
    pub fn set_title<'title>(&mut self, title: impl Into<Cow<'title, str>>) -> Result {
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
    ) -> Result {
        self.webview_mut()
            .dialog(dialog, flags, title, arg, result_buffer)
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

/// A handle that is safe to share between threads
///
/// The thread handle contains a weak reference to the webview contained within
/// the main handle, so it can only be used to dispatch function calls while
/// the main handle exists.
#[derive(Clone)]
pub struct ThreadHandle<'invoke, T> {
    inner: Weak<UnsafeCell<WebviewWrapper<'invoke, T>>>,
}

impl<'invoke, T> ThreadHandle<'invoke, T> {
    /// Attempt to dispatch a function call
    ///
    /// The specified function is queued for execution on the main thread in a
    /// thread safe manner and asynchronously executed.
    ///
    /// # Errors
    ///
    /// Since the `ThreadHandle` only has a weak reference to the webview, a
    /// `WebviewError::DispatchFailed` is returned, if the dispatch is attempted
    /// when the main handle no longer exists.
    #[inline]
    pub fn try_dispatch(&self, func: impl FnMut(&mut Webview, &mut T) + Send) -> Result {
        self.inner
            .upgrade()
            .map(|cell| {
                let webview = unsafe { &mut (*cell.get()).inner };
                webview.dispatch(func);
            }).ok_or(WebviewError::DispatchFailed)
    }
}
