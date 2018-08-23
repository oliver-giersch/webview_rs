use std::borrow::Cow;
use std::marker::PhantomData;
use std::path::Path;
use std::thread;

use crate::error::WebviewError;
use crate::ffi;
use crate::userdata::Userdata;
use crate::Webview;

use webview_sys::runtime_size_check;

pub struct WebviewBuilder<'title, 'content, T = ()> {
    title:                   Option<Cow<'title, str>>,
    content:                 Option<Cow<'content, str>>,
    width:                   Option<usize>,
    height:                  Option<usize>,
    resizeable:              bool,
    debug:                   bool,
    external_invoke:         Option<Box<dyn FnMut(&Webview<T>, &str)>>,
    userdata:                Option<T>,
    deactivate_thread_check: bool,
    buffer_size:             usize,
    error:                   Option<WebviewError>,
}

impl<'title, 'content, T> WebviewBuilder<'title, 'content, T> {
    #[inline]
    pub fn new() -> Self {
        runtime_size_check();
        Default::default()
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
    pub fn set_width(mut self, width: usize) -> Self {
        assert!(width > 0);
        self.width = Some(width);
        self
    }

    #[inline]
    pub fn set_height(mut self, height: usize) -> Self {
        assert!(height > 0);
        self.height = Some(height);
        self
    }

    #[inline]
    pub fn set_resizable(mut self, resizable: bool) -> Self {
        self.resizeable = resizable;
        self
    }

    #[inline]
    pub fn set_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    #[inline]
    //TODO: Find out why 'static
    pub fn set_external_invoke(mut self, func: impl FnMut(&Webview<T>, &str) + 'static) -> Self {
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
        self.deactivate_thread_check = true;
        self
    }

    #[inline(never)]
    pub fn build(self) -> Result<Webview<T>, WebviewError> {
        if let Some(error) = self.error {
            return Err(error);
        }

        if self.deactivate_thread_check {
            if let Some("main") = thread::current().name() {
            } else {
                return Err(WebviewError::InvalidThread);
            }
        }

        let title = self.title.ok_or(WebviewError::MissingArgs)?
            .into_owned();
        let content = self.content.ok_or(WebviewError::MissingArgs)?
            .into_owned();
        let width = self.width.unwrap_or(800);
        let height = self.height.unwrap_or(600);
        let resizable = self.resizeable;
        let debug = self.debug;
        let has_external_invoke = self.external_invoke.is_some();

        let webview = unsafe { ffi::struct_webview_new() };
        let built = Webview::new(
            webview,
            self.userdata,
            self.external_invoke,
            self.buffer_size,
        );

        unsafe {
            let webview = built.inner_webview();

            ffi::struct_webview_set_title(webview, title)?;
            ffi::struct_webview_set_content(webview, content)?;
            ffi::struct_webview_set_width(webview, width);
            ffi::struct_webview_set_height(webview, height);
            ffi::struct_webview_set_resizable(webview, resizable);
            ffi::struct_webview_set_debug(webview, debug);

            if has_external_invoke {
                ffi::struct_webview_set_external_invoke_cb::<T>(webview);
            }

            ffi::webview_init(webview);
        }

        Ok(built)
    }
}

impl<'title, 'content, T> WebviewBuilder<'title, 'content, T>
where
    T: Userdata,
{
    #[inline]
    pub fn set_userdata(mut self, userdata: T) -> Self {
        self.userdata = Some(userdata);
        self
    }
}

//TODO Battle borrowck
/*impl<'title, 'content, T> WebviewBuilder<'title, 'content, T> {
    #[inline]
    pub fn set_content_file<C>(mut self, path: C) -> Self
    where
        C: Into<Cow<'content, Path>>
    {
        let path = path.into();
        let string = path.to_string_lossy();
        self.content = Some(string);
        self
    }
}*/

impl<'title, 'content, T> Default for WebviewBuilder<'title, 'content, T> {
    #[inline]
    fn default() -> Self {
        Self {
            title:                   None,
            content:                 None,
            width:                   None,
            height:                  None,
            resizeable:              true,
            debug:                   false,
            external_invoke:         None,
            userdata:                None,
            deactivate_thread_check: false,
            buffer_size:             0,
            error:                   None,
        }
    }
}

pub enum Content<'content, C>
where
    C: Into<Cow<'content, str>>,
{
    Url(C),
    File(C),
    Html(C),
    __Hidden(PhantomData<&'content str>),
}

impl<'content, C> Into<Cow<'content, str>> for Content<'content, C>
where
    C: Into<Cow<'content, str>>,
{
    #[inline]
    fn into(self) -> Cow<'content, str> {
        match self {
            Content::Url(content) => into_url(content),
            Content::File(content) => into_file_path(content),
            Content::Html(content) => into_html(content),
            Content::__Hidden(_) => panic!("enum variant for internal use only"),
        }
    }
}

fn into_url<'s>(content: impl Into<Cow<'s, str>>) -> Cow<'s, str> {
    let content = content.into();
    match content {
        Cow::Borrowed(string) => {
            if string_starts_with_any(string, &["http://", "https://"]) {
                Cow::from(string)
            } else {
                Cow::from(format!("http://{}", string))
            }
        }
        Cow::Owned(mut string) => {
            if string_starts_with_any(&string, &["http://", "https://"]) {
                Cow::from(string)
            } else {
                string.insert_str(0, "http://");
                Cow::from(string)
            }
        }
    }
}

fn into_file_path<'s>(content: impl Into<Cow<'s, str>>) -> Cow<'s, str> {
    let content = content.into();
    match content {
        Cow::Borrowed(string) => {
            if string.starts_with("file:///") {
                Cow::from(string)
            } else {
                Cow::from(format!("file:///{}", string))
            }
        }
        Cow::Owned(mut string) => {
            if string.starts_with("file:///") {
                Cow::from(string)
            } else {
                string.insert_str(0, "file:///");
                Cow::from(string)
            }
        }
    }
}

fn into_html<'s>(content: impl Into<Cow<'s, str>>) -> Cow<'s, str> {
    let content = content.into();
    match content {
        Cow::Borrowed(string) => {
            if string.starts_with("data:text/html,") {
                Cow::from(string)
            } else {
                Cow::from(format!("data:text/html,{}", string))
            }
        }
        Cow::Owned(mut string) => {
            if string.starts_with("data:text/html,") {
                Cow::from(string)
            } else {
                string.insert_str(0, "data:text/html,");
                Cow::from(string)
            }
        }
    }
}

fn string_starts_with_any(string: &str, any: &[&str]) -> bool {
    any.iter().any(|&contain| string.starts_with(contain))
}
