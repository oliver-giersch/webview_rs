use std::borrow::Cow;
use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_char, c_int, c_void};

use webview_sys as sys;
use crate::Webview;use crate::callback;
use crate::conversion::{CStrConversionError, convert_to_cstring};
use crate::error::WebviewError;

type DispatchFn = sys::c_webview_dispatch_fn;
type InvokeFn = sys::c_extern_callback_fn;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub enum Dialog {
    Open = 0,
    Save = 1,
    Alert = 2,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum LoopResult {
    Continue,
    Exit,
}

impl From<i32> for LoopResult {
    #[inline]
    fn from(result: i32) -> Self {
        match result {
            0 => LoopResult::Continue,
            _ => LoopResult::Exit
        }
    }
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum LibraryError {
    Init(i32),
    Eval(i32)
}

bitflags! {
    pub struct Flags: i32 {
        const File      = 0b0000;
        const Directory = 0b0001;
        const Info      = 0b0010;
        const Warning   = 0b0100;
        const Error     = 0b0110;
        const AlertMask = 0b0110;
    }
}

/// # Struct field setters
///
/// The following functions wrap external C function calls, which set the individual fields of the
/// webview struct pre initialization.
/// These setters are outsourced to C code in order to prevent any breaking changes caused by
/// reordering of fields within the library.
///
/// The runtime sanity checks can detect differences in size and alignment of the C library structs
/// and the Rust re-implementations, but not differences in field ordering/arrangement.

#[inline]
pub unsafe fn struct_webview_set_title<'s>(
    webview: &mut sys::webview,
    title: impl Into<Cow<'s, CStr>>,
) {
    let title_cstr = title.into();
    let ptr = title_cstr.as_ptr();
    sys::struct_webview_set_title(webview as *mut _, ptr);
}

#[inline]
pub unsafe fn struct_webview_set_content<'s>(
    webview: &mut sys::webview,
    content: impl Into<Cow<'s, CStr>>,
) {
    let content_cstr = content.into();
    let ptr = content_cstr.as_ptr();
    sys::struct_webview_set_url(webview as *mut _, ptr);
}

#[inline]
pub unsafe fn struct_webview_set_width(webview: &mut sys::webview, width: usize) {
    sys::struct_webview_set_width(webview as *mut _, width as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_height(webview: &mut sys::webview, height: usize) {
    sys::struct_webview_set_height(webview as *mut _, height as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_resizable(webview: &mut sys::webview, resizable: bool) {
    sys::struct_webview_set_resizable(webview as *mut _, resizable as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_debug(webview: &mut sys::webview, debug: bool) {
    sys::struct_webview_set_debug(webview as *mut _, debug as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_external_invoke_cb<T>(webview: &mut sys::webview) {
    sys::struct_webview_set_external_invoke_cb(
        webview as *mut _,
        Some(callback::invoke_handler::<T> as InvokeFn),
    );
}

/// # Rust wrappers
/// The following functions provide Rust wrappers using Rust types and idioms to call the appropriate
/// (raw) extern C library functions.

#[inline]
pub unsafe fn webview_simple<'title, 'content>(
    title: impl Into<Cow<'title, str>>,
    content: impl Into<Cow<'content, str>>,
    width: usize,
    height: usize,
    resizable: bool,
) -> Result<(), WebviewError> {
    let title_cstr = convert_to_cstring(title)?;
    let content_cstr = convert_to_cstring(content)?;

    let result = sys::webview(
        title_cstr.as_ptr(),
        content_cstr.as_ptr(),
        width as c_int,
        height as c_int,
        resizable as c_int,
    );

    match result {
        0 => Ok(()),
        c => Err(WebviewError::from(LibraryError::Init(c)))
    }
}

#[must_use]
#[inline]
pub unsafe fn webview_init(webview: &mut sys::webview) -> Result<(), LibraryError> {
    let result = sys::webview_init(webview as *mut _);
    match result {
        0 => Ok(()),
        c => Err(LibraryError::Init(c))
    }
}

/// Executes the main loop for one iteration.
/// The result indicates whether another iterations should be run or the
/// webview has been terminated.
#[must_use]
#[inline]
pub unsafe fn webview_loop(webview: &mut sys::webview, blocking: bool) -> LoopResult {
    let result = sys::webview_loop(webview as *mut _, blocking as c_int);
    LoopResult::from(result)
}

#[must_use]
#[inline]
pub unsafe fn webview_eval(webview: &mut sys::webview, buffer: &[u8]) -> Result<(), WebviewError> {
    let js_cstr = CStr::from_bytes_with_nul(buffer)?;
    let result = sys::webview_eval(webview as *mut _, js_cstr.as_ptr());

    match result {
        0 => Ok(()),
        c => Err(WebviewError::from(LibraryError::Eval(c)))
    }
}

#[must_use]
#[inline]
pub unsafe fn webview_inject_css(webview: &mut sys::webview, buffer: &[u8]) -> Result<(), WebviewError> {
    let css_cstr = CStr::from_bytes_with_nul(buffer)?;
    let result = sys::webview_inject_css(webview as *mut _, css_cstr.as_ptr());

    match result {
        0 => Ok(()),
        c => Err(WebviewError::from(LibraryError::Eval(c)))
    }
}

//...set_title, set_fullscreen, set_color, dialog

#[inline]
pub unsafe fn webview_dispatch<T>(webview: &mut sys::webview, func: &dyn FnMut(&mut Webview<T>)) {
    let callback: *mut c_void = mem::transmute(&func);
    sys::webview_dispatch(
        webview as *mut _,
        Some(callback::dispatch_handler::<T> as DispatchFn),
        callback
    );
}

#[inline]
pub unsafe fn webview_terminate(webview: &mut sys::webview) {
    sys::webview_terminate(webview as *mut _);
}

#[inline]
pub unsafe fn webview_exit(webview: &mut sys::webview) {
    sys::webview_exit(webview as *mut _);
}

//...debug, print_log