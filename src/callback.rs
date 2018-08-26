use std::ffi::CStr;
use std::os::raw::{c_char, c_void};

use webview_sys as sys;
use crate::Webview;

/// Extern function for C callback
///
/// The C library calls this function, which in turn executes the provided
/// Closure
pub extern "system" fn invoke_handler<T>(webview: *mut sys::webview, arg: *const c_char) {
    unsafe {
        let webview = webview as *mut Webview<T>;
        let webview = &mut *webview;

        let func = webview.external_invoke.expect("no external invoke set");

        let cow = CStr::from_ptr(arg).to_string_lossy();
        let arg = cow.as_ref();

        func(webview, arg);
    }
}

/// Extern function for C callback
///
/// The C library calls this function, which in turn executes the provided
/// Closure
pub extern "system" fn dispatch_handler<T>(webview: *mut sys::webview, args: *mut c_void) {
    unsafe {
        let webview = webview as *mut Webview<T>;
        let webview = &mut *webview;

        //The `Send` bound variant is used here so the function can be used for
        // dispatches from the main thread as well as any other thread.
        let ptr: *mut &mut (FnMut(&mut Webview<T>) + Send) = args as _;
        let boxed = Box::from_raw(ptr);
        boxed(webview);
    }
}
