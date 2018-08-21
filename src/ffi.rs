use std::{
    ffi::{CStr, FromBytesWithNulError},
    mem,
    os::raw::{c_char, c_int, c_void},
};

use crate::Webview;
use crate::callback;
use crate::userdata::Userdata;
use webview_ffi as ffi;

type DispatchFn = ffi::c_webview_dispatch_fn;
type InvokeFn = ffi::c_extern_callback_fn;

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
#[repr(C)]
pub enum Dialog {
    Open  = 0,
    Save  = 1,
    Alert = 2,
}

#[derive(Debug, Clone, Copy, PartialOrd, PartialEq)]
pub enum LoopResult {
    Continue,
    Exit
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
pub unsafe fn struct_webview_new() -> ffi::webview {
    mem::uninitialized()
}

#[inline]
pub unsafe fn struct_webview_set_title(webview: &mut ffi::webview, title: &str) -> Result<(), FromBytesWithNulError> {
    let title_cstr = CStr::from_bytes_with_nul(title.as_bytes())?;
    ffi::struct_webview_set_title(webview as *mut _, title_cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn struct_webview_set_url(webview: &mut ffi::webview, url: &str) -> Result<(), FromBytesWithNulError> {
    let url_cstr = CStr::from_bytes_with_nul(url.as_bytes())?;
    ffi::struct_webview_set_url(webview as *mut _, url_cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn struct_webview_set_width(webview: &mut ffi::webview, width: usize) {
    ffi::struct_webview_set_width(webview as *mut _, width as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_height(webview: &mut ffi::webview, height: usize) {
    ffi::struct_webview_set_width(webview as *mut _, height as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_resizable(webview: &mut ffi::webview, resizable: bool) {
    ffi::struct_webview_set_resizable(webview as *mut _, resizable as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_debug(webview: &mut ffi::webview, debug: bool) {
    ffi::struct_webview_set_debug(webview as *mut _, debug as c_int);
}

#[inline]
pub unsafe fn struct_webview_set_external_invoke_cb<T, E>(webview: &mut ffi::webview)
where
    T: Userdata,
    E: FnMut(&Webview<T, E>, &str)
{
    ffi::struct_webview_set_external_invoke_cb(
        webview as *mut _,
        //Some(mem::transmute(callback::invoke_handler::<T, E>)),
        Some(callback::invoke_handler::<T, E> as InvokeFn)
    );
}

#[inline]
pub unsafe fn struct_webview_set_userdata<T: Userdata>(webview: &mut ffi::webview, userdata: &T) {
    ffi::struct_webview_set_userdata(webview as *mut _, userdata as *const _ as *mut c_void);
}

///
#[inline]
pub unsafe fn webview_simple(
    title: &str,
    content: &str,
    width: usize,
    height: usize,
    resizable: bool
) -> Result<(), FromBytesWithNulError>
{
    let title_cstr = CStr::from_bytes_with_nul(title.as_bytes())?;
    let content_cstr = CStr::from_bytes_with_nul(content.as_bytes())?;
    ffi::webview(
        title_cstr.as_ptr(),
        content_cstr.as_ptr(),
        width as c_int,
        height as c_int,
        resizable as c_int
    );
    Ok(())
}

/// TODO: Return result
#[inline]
pub unsafe fn webview_init(webview: &mut ffi::webview) {
    ffi::webview_init(webview as *mut _);
}

/// Executes the main loop for one iteration.
/// The result indicates whether another iterations should be run or the webview has been
/// terminated.
#[inline]
pub unsafe fn webview_loop(webview: &mut ffi::webview, blocking: bool) -> LoopResult {
    LoopResult::from(ffi::webview_loop(webview as *mut _, blocking as c_int))
}

/// TODO: Return Result
#[inline]
pub unsafe fn webview_eval(webview: &mut ffi::webview, js: &str) -> Result<(), FromBytesWithNulError>{
    let js_cstr = CStr::from_bytes_with_nul(js.as_bytes())?;
    ffi::webview_eval(webview as *mut _, js_cstr.as_ptr());
    Ok(())
}

/// TODO: Return Result
#[inline]
pub unsafe fn webview_inject_css(webview: &mut ffi::webview, css: &str) -> Result<(), FromBytesWithNulError>{
    let css_cstr = CStr::from_bytes_with_nul(css.as_bytes())?;
    ffi::webview_eval(webview as *mut _, css_cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn webview_set_title(webview: &mut ffi::webview, title: &str) -> Result<(), FromBytesWithNulError> {
    let cstr = CStr::from_bytes_with_nul(title.as_bytes())?;
    ffi::webview_set_title(webview as *mut _, cstr.as_ptr());
    Ok(())
}

#[inline]
pub unsafe fn webview_set_fullscreen(webview: &mut ffi::webview, fullscreen: bool) {
    ffi::webview_set_fullscreen(webview as *mut _, fullscreen as c_int);
}

#[inline]
pub unsafe fn webview_set_color(webview: &mut ffi::webview, red: u8, green: u8, blue: u8) {
    ffi::webview_set_color(
        webview as *mut _,
        red,
        green,
        blue,
    );
}

#[inline]
pub unsafe fn webview_dialog(
    webview: &mut ffi::webview,
    dialog_type: Dialog,
    flags: Flags,
    title: &str,
    arg: &str,
    result_buffer: &mut [u8]
) -> Result<(), FromBytesWithNulError>
{
    let title_cstr = CStr::from_bytes_with_nul(title.as_bytes())?;
    let arg_cstr = CStr::from_bytes_with_nul(arg.as_bytes())?;
    let (result_ptr, result_size) = (result_buffer.as_mut_ptr(), result_buffer.len());

    ffi::webview_dialog(
        webview as *mut _,
        dialog_type as c_int,
        flags.bits() as c_int,
        title_cstr.as_ptr(),
        arg_cstr.as_ptr(),
        result_ptr as *mut c_char,
        result_size
    );
    Ok(())
}

#[inline]
pub unsafe fn webview_dispatch<T, E>(webview: &mut ffi::webview, func: &dyn FnMut(&Webview<T, E>))
where
    T: Userdata,
    E: FnMut(&Webview<T, E>, &str)
{
    let callback = &func as *const _ as *mut c_void;
    ffi::webview_dispatch(
        webview as *mut _,
        Some(callback::dispatch_handler::<T, E> as DispatchFn),
        callback
    );
}

#[inline]
pub unsafe fn webview_terminate(webview: &mut ffi::webview) {
    ffi::webview_terminate(webview as *mut _);
}

#[inline]
pub unsafe fn webview_exit(webview: &mut ffi::webview) {
    ffi::webview_exit(webview as *mut _);
}

#[inline]
pub unsafe fn webview_print_log(log: &str) -> Result<(), FromBytesWithNulError> {
    let cstr = CStr::from_bytes_with_nul(log.as_bytes())?;
    ffi::webview_print_log(cstr.as_ptr());
    Ok(())
}