use std::borrow::Cow;
use std::mem;
//use std::path::Path; TODO: add set_content for Path
use std::thread;

use crate::content::Content;
use crate::conversion::convert_to_cstring;
use crate::error::WebviewError;
use crate::eval::StringBuffers;
use crate::ffi;
use crate::{Extension, ExternalInvokeFnBox, Webview, WebviewHandle, WebviewWrapper};
use webview_sys as sys;

pub struct Builder<'title, 'content, 'invoke, T> {
    title:           Option<Cow<'title, str>>,
    content:         Option<Cow<'content, str>>,
    size:            Option<(usize, usize)>,
    resizable:       bool,
    debug:           bool,
    external_invoke: Option<ExternalInvokeFnBox<'invoke, T>>,
    userdata:        T,
    thread_check:    bool,
    buffer_size:     usize,
}

impl<'title, 'content, 'invoke> Builder<'title, 'content, 'invoke, ()> {
    #[inline]
    pub fn without_userdata() -> Self {
        sys::runtime_size_check();
        Builder {
            title:           None,
            content:         None,
            size:            None,
            resizable:       true,
            debug:           false,
            external_invoke: None,
            userdata:        (),
            thread_check:    true,
            buffer_size:     0,
        }
    }
}

impl<'title, 'content, 'invoke, T> Builder<'title, 'content, 'invoke, T> {
    #[inline]
    pub fn with_userdata(userdata: T) -> Builder<'title, 'content, 'invoke, T> {
        sys::runtime_size_check();
        Builder {
            title: None,
            content: None,
            size: None,
            resizable: true,
            debug: false,
            external_invoke: None,
            userdata,
            thread_check: true,
            buffer_size: 0,
        }
    }

    #[inline]
    pub fn set_title(mut self, title: impl Into<Cow<'title, str>>) -> Self {
        self.title = Some(title.into());
        self
    }

    #[inline]
    pub fn set_content_url(self, url: impl Into<Cow<'content, str>>) -> Self {
        self.set_content(Content::Url(url))
    }

    #[inline]
    pub fn set_content_html(self, html: impl Into<Cow<'content, str>>) -> Self {
        self.set_content(Content::Html(html))
    }

    #[inline]
    pub fn set_content<C>(mut self, content: impl Into<Content<'content, C>>) -> Self
    where
        C: Into<Cow<'content, str>>,
    {
        self.content = Some(content.into().into());
        self
    }

    #[inline]
    pub fn set_size(mut self, width: usize, height: usize) -> Self {
        assert!(width > 0 && height > 0);
        self.size = Some((width, height));
        self
    }

    #[inline]
    pub fn set_resizable(mut self, resizable: bool) -> Self {
        self.resizable = resizable;
        self
    }

    #[inline]
    pub fn set_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    #[inline]
    pub fn set_external_invoke(
        mut self,
        func: impl FnMut(&mut Webview, &mut T, &str) + 'invoke,
    ) -> Self {
        self.external_invoke = Some(Box::new(func));
        self
    }

    #[inline]
    pub fn set_initial_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    #[inline]
    pub fn deactivate_thread_check(mut self) -> Self {
        self.thread_check = false;
        self
    }

    #[inline(never)]
    pub fn build(self) -> Result<WebviewHandle<'invoke, T>, WebviewError> {
        if self.thread_check && !is_main_thread() {
            return Err(WebviewError::InvalidThread);
        }

        let title = self.title.ok_or(WebviewError::Build)?;
        let content = self.content.ok_or(WebviewError::Build)?;
        let (width, height) = self.size.unwrap_or((800, 600));

        let inner = unsafe {
            let buffers = StringBuffers::new(
                convert_to_cstring(title)?,
                convert_to_cstring(content)?,
                self.buffer_size,
            );

            let mut webview: sys::webview = mem::zeroed();
            ffi::struct_webview_set_title(&mut webview, &buffers.title);
            ffi::struct_webview_set_content(&mut webview, &buffers.content);
            ffi::struct_webview_set_width(&mut webview, width);
            ffi::struct_webview_set_height(&mut webview, height);
            ffi::struct_webview_set_resizable(&mut webview, self.resizable);
            ffi::struct_webview_set_debug(&mut webview, self.debug);

            if self.external_invoke.is_some() {
                ffi::struct_webview_set_external_invoke_cb::<T>(&mut webview);
            }

            Webview { webview, buffers }
        };

        let mut built = WebviewHandle::new(WebviewWrapper {
            inner,
            ext: Extension {
                external_invoke: self.external_invoke,
                userdata:        self.userdata,
            },
        });

        unsafe {
            let inner = built.webview_mut();
            ffi::webview_init(&mut inner.webview)?;
        }

        Ok(built)
    }
}

#[inline]
fn is_main_thread() -> bool {
    thread::current()
        .name()
        .map_or(false, |name| name == "main")
}
