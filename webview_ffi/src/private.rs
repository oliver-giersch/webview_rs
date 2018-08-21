#[cfg(any(target_os = "linux", target_os = "bsd"))]
pub use self::linux::webview_private;
#[cfg(target_os = "windows")]
pub use self::windows::webview_private;
#[cfg(target_os = "macos")]
pub use self::macos::webview_private;

#[cfg(any(target_os = "linux", target_os = "bsd"))]
mod linux {
    use std::os::raw::c_int;

    enum GtkWidget {}
    enum GAsyncQueue {}

    #[allow(non_camel_case_types)]
    #[repr(C)]
    pub struct webview_private {
        window: *mut GtkWidget,
        scroller: *mut GtkWidget,
        webview: *mut GtkWidget,
        inspector_window: *mut GtkWidget,
        queue: *mut GAsyncQueue,
        read: c_int,
        js_busy: c_int,
        should_exit: c_int,
    }
}

#[cfg(target_os = "windows")]
mod windows {
    use std::os::raw::{BOOL, DWORD, LONG};

    enum Opaque {}
    enum IOleObject {}

    type HWND = *mut Opaque;

    #[allow(non_camel_case_types)]
    #[repr(C)]
    pub struct webview_private {
        hwnd: HWND,
        browser: *mut *mut IOleObject,
        is_fullscreen: BOOL,
        saved_style: DWORD,
        saved_ex_style: DWORD,
        saved_rect: RECT,
    }

    #[allow(non_camel_case_types)]
    #[repr(C)]
    struct RECT {
        left: LONG,
        top: LONG,
        right: LONG,
        bottom: LONG,
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use std::os::raw::c_int;

    enum Opaque {}

    #[allow(non_camel_case_types)]
    type id = *mut Opaque;

    #[allow(non_camel_case_types)]
    #[repr(C)]
    pub struct webview_private {
        pool: id,
        window: id,
        webview: id,
        window_delegate: id,
        should_exit: c_int,
    }
}