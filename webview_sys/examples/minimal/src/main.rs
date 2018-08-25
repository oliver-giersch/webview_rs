extern crate webview_sys as sys;

use std::ffi::CStr;
use std::mem;

static TITLE: &'static str = "Minimal webview example\0";
static URL: &'static str = "https://en.m.wikipedia.org/wiki/Main_Page\0";

fn main() {
    unsafe {
        let mut webview = struct
    }
}

fn main_alt() {
    unsafe {
        /*let mut webview: webview = mem::uninitialized();
        struct_webview_set_title(
            &mut webview as *mut _,
            ffi::CStr::from_bytes_with_nul_unchecked(TITLE.as_bytes()).as_ptr()
        );
        struct_webview_set_url(
            &mut webview as *mut _,
            ffi::CStr::from_bytes_with_nul_unchecked(URL.as_bytes()).as_ptr()
        );
        struct_webview_set_width(
            &mut webview as *mut _,
            800
        );
        struct_webview_set_height(
            &mut webview as *mut _,
            600
        );
        struct_webview_set_resizable(
            &mut webview as *mut _,
            1
        );

        let result = webview_init(&mut webview as *mut _,);
        assert_eq!(0, result);

        loop {
            if webview_loop(&mut webview as *mut _, 1) == 1 {
                break;
            }
        }*/
        sys::webview(
            CStr::from_bytes_with_nul_unchecked(TITLE.as_bytes()).as_ptr(),
            CStr::from_bytes_with_nul_unchecked(URL.as_bytes()).as_ptr(),
            800,
            600,
            1
        );
    }
}
