use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_char, c_void};

use crate::Webview;
use webview_sys::webview;

/// Extern function for C callback
///
/// The C library calls this function, which in turn executes the provided
/// Closure
pub extern "system" fn invoke_handler<T>(webview: *mut webview, arg: *const c_char) {
    unsafe {
        let webview_ptr: *mut Webview<T> = mem::transmute(webview);
        let webview = &*webview_ptr;

        let func = webview.external_invoke();

        let cow = CStr::from_ptr(arg).to_string_lossy();
        let arg = cow.as_ref();

        func(webview, arg);
    }
}

/// Extern function for C callback
///
/// The C library calls this function, which in turn executes the provided
/// Closure
pub extern "system" fn dispatch_handler<T>(webview: *mut webview, args: *mut c_void) {
    unsafe {
        let webview_ptr: *mut Webview<T> = mem::transmute(webview);
        let webview = &*webview_ptr;

        //The `Send` bound variant is used here so the function can be used for
        // dispatches from the main thread as well as any other thread.
        let func: &mut &mut (FnMut(&Webview<T>) + Send) = mem::transmute(args);
        func(webview);
    }
}
