use std::ffi::CStr;
use std::mem;
use std::os::raw::{c_char, c_void};

use crate::{Webview, WebviewWrapper};
use webview_sys as sys;

/// Extern function for C callback
///
/// The C library calls this function, which in turn executes the provided
/// Closure
pub extern "system" fn invoke_handler<'invoke, T>(webview: *mut sys::webview, arg: *const c_char) {
    unsafe {
        let wrapper = webview as *mut WebviewWrapper<'invoke, T>;
        let (func, userdata) = {
            let wrapper = &mut *wrapper;
            (wrapper.ext.external_invoke.as_mut().expect("no internal invoke set"), &mut wrapper.ext.userdata)
        };

        let cow = CStr::from_ptr(arg).to_string_lossy();
        let arg = cow.as_ref();

        func(&mut (*wrapper).inner, userdata, arg);
    }
}

/// Extern function for C callback
///
/// The C library calls this function, which in turn executes the provided
/// Closure
pub extern "system" fn dispatch_handler<'invoke, T>(webview: *mut sys::webview, args: *mut c_void) {
    unsafe {
        let wrapper = webview as *mut WebviewWrapper<'invoke, T>;
        let webview = &mut (*wrapper).inner;
        let userdata = &mut (*wrapper).ext.userdata;

        //The `Send` bound variant is used here so the function can be used for
        // dispatches from the main thread as well as any other thread.
        let func: &mut &mut (FnMut(&mut Webview, &mut T) + Send) = mem::transmute(args);
        func(webview, userdata);
    }
}
