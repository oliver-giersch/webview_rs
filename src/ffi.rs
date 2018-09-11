//! TODO: ffi mod doc

use std::borrow::Cow;
use std::error;
use std::ffi::CStr;
use std::fmt;
use std::mem;
use std::os::raw::{c_char, c_int, c_void};

use crate::callback;
use crate::conversion::convert_to_cstring;
use crate::error::WebviewError;
use crate::Webview;
use webview_sys as sys;

type DispatchFn = sys::c_webview_dispatch_fn;
type InvokeFn = sys::c_extern_callback_fn;

/// Dialog options
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub enum Dialog {
    Open = 0,
    Save = 1,
    Alert = 2,
}

/// Loop result
///
/// Each iteration in `webview_loop` returns either a `Continue` result, or an
/// `Exit` result (for instance when `webview_terminate` has been called during
/// the last iteration)
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
            _ => LoopResult::Exit,
        }
    }
}

/// webview C library errors
#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum LibraryError {
    Init(i32),
    Eval(i32),
}

impl fmt::Display for LibraryError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            LibraryError::Init(val) => {
                write!(f, "failed to initialize webview (error code: {})", val)
            }
            LibraryError::Eval(val) => write!(f, "failed to evaluate js/css (error code: {})", val),
        }
    }
}

impl error::Error for LibraryError {}

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

/**
 ** The following functions are used to set the individual fields of the
 ** webview struct.
 **
 ** These functions are bound to external C functions, so any changes to
 ** the order of fields in the library don't necessarily constitute
 ** breaking changes.
 **/

/// Set title of the webview struct
///
/// #Notes
///
/// The webview struct does not take ownership of the supplied title string
#[inline]
pub unsafe fn struct_webview_set_title<'s>(
    webview: &mut sys::webview,
    title: impl Into<Cow<'s, CStr>>,
) {
    let title_cstr = title.into();
    let ptr = title_cstr.as_ptr();
    sys::struct_webview_set_title(webview as *mut _, ptr);
}

/// Set content of the webview struct
///
/// #Notes
///
/// The webview struct does not take ownership of the supplied content string
#[inline]
pub unsafe fn struct_webview_set_content<'s>(
    webview: &mut sys::webview,
    content: impl Into<Cow<'s, CStr>>,
) {
    let content_cstr = content.into();
    let ptr = content_cstr.as_ptr();
    sys::struct_webview_set_url(webview as *mut _, ptr);
}

/// Set the width of the webview struct
#[inline]
pub unsafe fn struct_webview_set_width(webview: &mut sys::webview, width: usize) {
    sys::struct_webview_set_width(webview as *mut _, width as c_int);
}

/// Set the height of the webview struct
#[inline]
pub unsafe fn struct_webview_set_height(webview: &mut sys::webview, height: usize) {
    sys::struct_webview_set_height(webview as *mut _, height as c_int);
}

/// Set the resizability attribute of the webview struct
#[inline]
pub unsafe fn struct_webview_set_resizable(webview: &mut sys::webview, resizable: bool) {
    sys::struct_webview_set_resizable(webview as *mut _, resizable as c_int);
}

/// Set the debug attribute of the webview struct
#[inline]
pub unsafe fn struct_webview_set_debug(webview: &mut sys::webview, debug: bool) {
    sys::struct_webview_set_debug(webview as *mut _, debug as c_int);
}

/// Set the function pointer for the external invoke callback
///
/// This is set to the `callback::invoke_handler` function.
#[inline]
pub unsafe fn struct_webview_set_external_invoke_cb<'invoke, T>(webview: &mut sys::webview) {
    sys::struct_webview_set_external_invoke_cb(
        webview as *mut _,
        Some(callback::invoke_handler::<T> as InvokeFn),
    );
}

/// Initializes the webview data structure
#[must_use]
#[inline]
pub unsafe fn webview_init(webview: &mut sys::webview) -> Result<(), LibraryError> {
    let result = sys::webview_init(webview as *mut _);
    match result {
        0 => Ok(()),
        c => Err(LibraryError::Init(c)),
    }
}

/// Executes the main loop for one iteration.
///
/// The result indicates whether another iteration should be run or the
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
        c => Err(WebviewError::from(LibraryError::Eval(c))),
    }
}

#[must_use]
#[inline]
pub unsafe fn webview_inject_css(
    webview: &mut sys::webview,
    buffer: &[u8],
) -> Result<(), WebviewError> {
    let css_cstr = CStr::from_bytes_with_nul(buffer)?;
    let result = sys::webview_inject_css(webview as *mut _, css_cstr.as_ptr());

    match result {
        0 => Ok(()),
        c => Err(WebviewError::from(LibraryError::Eval(c))),
    }
}

#[inline]
pub unsafe fn webview_set_title<'title>(
    webview: &mut sys::webview,
    title: impl Into<Cow<'title, str>>,
) -> Result<(), WebviewError> {
    let title_cstr = convert_to_cstring(title)?;
    sys::webview_set_title(webview as *mut _, title_cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn webview_set_fullscreen(webview: &mut sys::webview, fullscreen: bool) {
    sys::webview_set_fullscreen(webview as *mut _, fullscreen as c_int);
}

#[inline]
pub unsafe fn webview_set_color(
    webview: &mut sys::webview,
    red: u8,
    green: u8,
    blue: u8,
    alpha: u8,
) {
    sys::webview_set_color(webview as *mut _, red, green, blue, alpha);
}

#[inline]
pub unsafe fn webview_dialog<'title, 'arg>(
    webview: &mut sys::webview,
    dialog_type: Dialog,
    flags: Flags,
    title: impl Into<Cow<'title, str>>,
    arg: impl Into<Cow<'arg, str>>,
    result_buffer: &mut [u8],
) -> Result<(), WebviewError> {
    let title_cstr = convert_to_cstring(title)?;
    let arg_cstr = convert_to_cstring(arg)?;
    let (ptr, size) = (result_buffer.as_mut_ptr(), result_buffer.len());

    sys::webview_dialog(
        webview as *mut _,
        dialog_type as c_int,
        flags.bits() as c_int,
        title_cstr.as_ptr(),
        arg_cstr.as_ptr(),
        ptr as *mut c_char,
        size,
    );
    Ok(())
}

#[inline]
pub unsafe fn webview_dispatch<'invoke, T>(
    webview: &mut sys::webview,
    func: &dyn FnMut(&mut Webview, &mut T),
) {
    let callback: *mut c_void = mem::transmute(&func);
    sys::webview_dispatch(
        webview as *mut _,
        Some(callback::dispatch_handler::<T> as DispatchFn),
        callback,
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
