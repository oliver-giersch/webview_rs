use std::{
    ffi::{CStr, CString, NulError, FromBytesWithNulError},
    mem,
    os::raw::{c_char, c_int, c_void},
};

use crate::callback;
use crate::userdata::Userdata;
use crate::Webview;
use webview_sys as sys;

pub type Result = std::result::Result<(), CStrError>;

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

impl From<c_int> for LoopResult {
    #[inline]
    fn from(result: i32) -> Self {
        match result {
            0 => LoopResult::Continue,
            _ => LoopResult::Exit,
        }
    }
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

#[inline]
pub unsafe fn struct_webview_new() -> sys::webview {
    mem::uninitialized()
}

#[inline]
pub unsafe fn struct_webview_set_title(
    webview: &mut sys::webview,
    title: impl Into<String>
) -> Result {
    let title_cstring = CString::new(title.into())?;
    sys::struct_webview_set_title(webview as *mut _, title_cstring.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn struct_webview_set_content(
    webview: &mut sys::webview,
    content: impl Into<String>,
) -> Result {
    let content_cstring = CString::new(content.into())?;
    sys::struct_webview_set_url(webview as *mut _, content_cstring.as_ptr());
    Ok(())
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

#[inline]
pub unsafe fn struct_webview_set_userdata<T: Userdata>(webview: &mut sys::webview, userdata: &T) {
    sys::struct_webview_set_userdata(webview as *mut _, userdata as *const _ as *mut c_void);
}

///
#[inline]
pub unsafe fn webview_simple(
    title: impl Into<String>,
    content: impl Into<String>,
    width: usize,
    height: usize,
    resizable: bool,
) -> Result {
    let title_cstring = CString::new(title.into())?;
    let content_cstring = CString::new(content.into())?;
    sys::webview(
        title_cstr.as_ptr(),
        content_cstr.as_ptr(),
        width as c_int,
        height as c_int,
        resizable as c_int,
    );
    Ok(())
}

/// TODO: Return result
#[inline]
pub unsafe fn webview_init(webview: &mut sys::webview) {
    sys::webview_init(webview as *mut _);
}

/// Executes the main loop for one iteration.
/// The result indicates whether another iterations should be run or the
/// webview has been terminated.
#[inline]
pub unsafe fn webview_loop(webview: &mut sys::webview, blocking: bool) -> LoopResult {
    LoopResult::from(sys::webview_loop(webview as *mut _, blocking as c_int))
}

/// TODO: Return Result
#[inline]
pub unsafe fn webview_eval(webview: &mut sys::webview, buffer: &mut String) -> Result {
    buffer.push('\0');
    let js_cstr = CStr::from_bytes_with_nul(js.as_bytes())?;
    sys::webview_eval(webview as *mut _, js_cstr.as_ptr());
    Ok(())
}

/// TODO: Return Result
#[inline]
pub unsafe fn webview_inject_css(
    webview: &mut sys::webview,
    css: &str,
) -> Result<(), FromBytesWithNulError> {
    let css_cstr = CStr::from_bytes_with_nul(css.as_bytes())?;
    sys::webview_eval(webview as *mut _, css_cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn webview_set_title(
    webview: &mut sys::webview,
    title: &str,
) -> Result<(), FromBytesWithNulError> {
    let cstr = CStr::from_bytes_with_nul(title.as_bytes())?;
    sys::webview_set_title(webview as *mut _, cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn webview_set_fullscreen(webview: &mut sys::webview, fullscreen: bool) {
    sys::webview_set_fullscreen(webview as *mut _, fullscreen as c_int);
}

#[inline]
pub unsafe fn webview_set_color(webview: &mut sys::webview, red: u8, green: u8, blue: u8) {
    sys::webview_set_color(webview as *mut _, red, green, blue);
}

#[inline]
pub unsafe fn webview_dialog(
    webview: &mut sys::webview,
    dialog_type: Dialog,
    flags: Flags,
    title: &str,
    arg: &str,
    result_buffer: &mut [u8],
) -> Result<(), FromBytesWithNulError> {
    let title_cstr = CStr::from_bytes_with_nul(title.as_bytes())?;
    let arg_cstr = CStr::from_bytes_with_nul(arg.as_bytes())?;
    let (result_ptr, result_size) = (result_buffer.as_mut_ptr(), result_buffer.len());

    sys::webview_dialog(
        webview as *mut _,
        dialog_type as c_int,
        flags.bits() as c_int,
        title_cstr.as_ptr(),
        arg_cstr.as_ptr(),
        result_ptr as *mut c_char,
        result_size,
    );
    Ok(())
}

#[inline]
pub unsafe fn webview_dispatch<T>(webview: &mut sys::webview, func: &dyn FnMut(&Webview<T>)) {
    let callback = &func as *const _ as *mut c_void;
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

#[inline]
pub unsafe fn webview_print_log(log: &str) -> Result<(), FromBytesWithNulError> {
    let cstr = CStr::from_bytes_with_nul(log.as_bytes())?;
    sys::webview_print_log(cstr.as_ptr());
    Ok(())
}

pub enum CStrError {
    FromBytesWithNul(FromBytesWithNulError),
    Nul(NulError),
}

impl From<FromBytesWithNulError> for CStrError {
    #[inline]
    fn from(err: FromBytesWithNulError) -> Self {
        CStrError(err)
    }
}

impl From<NulError> for CStrError {
    #[inline]
    fn from(err: NulError) -> Self {
        CStrError::Nul(err)
    }
}

mod conversion {

}

#[cfg(test)]
mod test {
    use std::mem;

    use super::*;

    #[test]
    fn simple() {
        unsafe {
            let mut webview = struct_webview_new();
            struct_webview_set_title(&mut webview, "Simple Test\0")
                .unwrap();
            struct_webview_set_content(&mut webview, "https://en.wikipedia.org/wiki/Main_Page\0")
                .unwrap();
            struct_webview_set_width(&mut webview, 800);
            struct_webview_set_height(&mut webview, 600);
            struct_webview_set_resizable(&mut webview, true);

            webview_init(&mut webview);
            webview_exit(&mut webview);
            assert!(true)
        }
    }
}