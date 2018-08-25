use std::mem;
use std::os::raw::{c_char, c_int, c_void};

use private::webview_private;

mod private;

#[inline]
pub fn runtime_size_check() {
    unsafe {
        assert_eq!(
            mem::size_of::<webview>(),
            struct_webview_size(),
            "size of `webview` does not match C library."
        );
        assert_eq!(
            mem::size_of::<webview_private>(),
            struct_webview_priv_size(),
            "size of `webview_private` does not match C library."
        );
    }
}

#[allow(non_camel_case_types)]
pub type c_extern_callback_fn = extern "system" fn(*mut webview, *const c_char);

#[allow(non_camel_case_types)]
pub type c_webview_dispatch_fn = extern "system" fn(*mut webview, arg: *mut c_void);

#[allow(non_camel_case_types)]
#[repr(C)]
pub struct webview {
    pub(crate) url: *const c_char,
    pub(crate) title: *const c_char,
    pub(crate) width: c_int,
    pub(crate) height: c_int,
    pub(crate) resizable: c_int,
    pub(crate) debug: c_int,
    pub(crate) external_invoke_cb: c_extern_callback_fn,
    pub(crate) private: webview_private,
    pub(crate) userdata: *mut c_void,
}

/// webview_rs C wrapper functions
extern "C" {
    pub fn alloc_webview() -> *mut webview;
    pub fn free_webview(webview: *mut webview);
    pub fn struct_webview_size() -> usize;
    pub fn struct_webview_priv_size() -> usize;
    pub fn struct_webview_set_title(webview: *mut webview, title: *const c_char);
    pub fn struct_webview_set_url(webview: *mut webview, url: *const c_char);
    pub fn struct_webview_set_width(webview: *mut webview, width: c_int);
    pub fn struct_webview_set_height(webview: *mut webview, height: c_int);
    pub fn struct_webview_set_resizable(webview: *mut webview, resizable: c_int);
    pub fn struct_webview_set_debug(webview: *mut webview, debug: c_int);
    pub fn struct_webview_set_external_invoke_cb(
        webview: *mut webview,
        external_invoke_cb: Option<c_extern_callback_fn>,
    );
    pub fn struct_webview_set_userdata(webview: *mut webview, userdata: *mut c_void);
}

extern "C" {
    /// Creates simple webview with mandatory parameters only.
    pub fn webview(
        title: *const c_char,
        url: *const c_char,
        width: c_int,
        height: c_int,
        resizable: c_int,
    ) -> c_int;

    /// Initializes the webview struct (returns -1 if initialization fails)
    #[must_use]
    pub fn webview_init(webview: *mut webview) -> c_int;

    /// Run the webview main loop
    #[must_use]
    pub fn webview_loop(webview: *mut webview, blocking: c_int) -> c_int;

    /// Inject (evaluate) JS code
    #[must_use]
    pub fn webview_eval(webview: *mut webview, js: *const c_char) -> c_int;

    /// Inject css code
    #[must_use]
    pub fn webview_inject_css(webview: *mut webview, css: *const c_char) -> c_int;

    /// Set the title at runtime
    pub fn webview_set_title(webview: *mut webview, title: *const c_char) -> c_void;

    /// Set the fullscreen parameter at runtime
    pub fn webview_set_fullscreen(webview: *mut webview, fullscreen: c_int) -> c_void;

    /// Set the color at runtime
    pub fn webview_set_color(webview: *mut webview, red: u8, green: u8, blue: u8) -> c_void;

    ///
    pub fn webview_dialog(
        webview: *mut webview,
        dialog_type: c_int,
        flags: c_int,
        title: *const c_char,
        arg: *const c_char,
        result: *mut c_char,
        result_size: usize,
    ) -> c_void;

    /// Dispatch a callback from another thread
    pub fn webview_dispatch(
        webview: *mut webview,
        func: Option<c_webview_dispatch_fn>,
        arg: *mut c_void,
    ) -> c_void;

    /// Terminates the webview main loop
    pub fn webview_terminate(webview: *mut webview) -> c_void;

    /// Exits & deallocates the webview
    pub fn webview_exit(webview: *mut webview) -> c_void;

    ///
    pub fn webview_debug(format: *const c_char, ...) -> c_void;

    ///
    pub fn webview_print_log(log: *const c_char) -> c_void;
}

#[cfg(test)]
mod test {
    use std::mem;

    use super::*;

    #[test]
    fn struct_sizes() {
        unsafe {
            assert_eq!(mem::size_of::<webview>(), struct_webview_size());
            assert_eq!(
                mem::size_of::<webview_private>(),
                struct_webview_priv_size()
            );
        }
    }

    #[test]
    fn init_exit() {
        unsafe {
            let mut webview = mem::uninitialized();
            struct_webview_set_title(
                &mut webview as *mut _,
                "test".as_bytes().as_ptr() as *const c_char,
            );
            struct_webview_set_url(
                &mut webview as *mut _,
                "https://en.wikipedia.org/wiki/Main_Page"
                    .as_bytes()
                    .as_ptr() as *const c_char,
            );
            struct_webview_set_width(&mut webview as *mut _, 800);
            struct_webview_set_height(&mut webview as *mut _, 60);

            webview_init(&mut webview as *mut _);
            webview_exit(&mut webview as *mut _);
        }
    }
}
