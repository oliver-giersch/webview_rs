use std::borrow::Cow;
use std::marker::PhantomData;
use std::path::Path;
use std::thread;

use crate::Webview;
use crate::error::WebviewError;
use crate::ffi;
use crate::userdata::Userdata;

use webview_ffi::runtime_size_check;

pub struct WebviewBuilder<'title, 'content, C, E, T>
where
    C: Into<Cow<'content, str>> + 'content,
    E: FnMut(&Webview<T, E>, &str),
    T: Userdata
{
    title: Option<&'title str>,
    content: Option<Content<'content, C>>,
    width: Option<usize>,
    height: Option<usize>,
    resizeable: bool,
    debug: bool,
    external_invoke: Option<E>,
    userdata: Option<T>,
    deactivate_thread_check: bool,
    buffer_size: usize,
    error: Option<WebviewError>,
}

impl<'title, 'content, C, E, T> WebviewBuilder<'title, 'content, C, E, T>
where
    C: Into<Cow<'content, str>> + 'content,
    E: FnMut(&Webview<T, E>, &str),
    T: Userdata
{
    pub fn new() -> Self {
        runtime_size_check();
        Default::default()
    }

    pub fn set_title(mut self, title: &'title impl AsRef<str>) -> Self {
        self.title = Some(title.as_ref());
        self
    }

    pub fn set_content_http(self, http: C) -> Self {
        self.set_content(Content::Http(http))
    }

    pub fn set_content_https(self, https: C) -> Self {
        self.set_content(Content::Https(https))
    }

    pub fn set_content_html(self, html: C) -> Self {
        self.set_content(Content::Html(html))
    }

    pub fn set_content_raw(self, raw: C) -> Self {
        self.set_content(Content::Raw(raw))
    }

    //pub fn set_content(mut self, content: impl Into<Content<'content, C>>) -> Self {
    pub fn set_content(mut self, content: Content<'content, C>) -> Self {
        //let content = content.into();
        self.content = Some(content);
        self
    }

    pub fn set_width(mut self, width: usize) -> Self {
        assert!(width > 0);
        self.width = Some(width);
        self
    }

    pub fn set_height(mut self, height: usize) -> Self {
        assert!(height > 0);
        self.height = Some(height);
        self
    }

    pub fn set_resizable(mut self, resizable: bool) -> Self {
        self.resizeable = resizable;
        self
    }

    pub fn set_debug(mut self, debug: bool) -> Self {
        self.debug = debug;
        self
    }

    pub fn set_external_invoke(mut self, func: E) -> Self {
        self.external_invoke = Some(func);
        self
    }

    pub fn set_userdata(mut self, userdata: T) -> Self {
        self.userdata = Some(userdata);
        self
    }

    pub fn set_initial_buffer_size(mut self, buffer_size: usize) -> Self {
        self.buffer_size = buffer_size;
        self
    }

    pub fn deactivate_thread_check(mut self) -> Self {
        self.deactivate_thread_check = true;
        self
    }

    pub fn build(self) -> Result<Webview<T, E>, WebviewError> {
        if let Some(error) = self.error {
            return Err(error);
        }

        if self.deactivate_thread_check {
            if let Some("main") = thread::current().name() {} else {
                return Err(WebviewError::InvalidThread);
            }
        }

        let title = self.title.ok_or(WebviewError::MissingArgs)?;
        //TODO
        let _content = self.content.ok_or(WebviewError::MissingArgs)?;
        let width = self.width.unwrap_or(800);
        let height = self.width.unwrap_or(600);
        let resizable = self.resizeable;
        let debug = self.debug;
        let has_external_invoke = self.external_invoke.is_some();

        let webview = unsafe { ffi::struct_webview_new() };
        let built = Webview::new(
            webview,
            self.userdata,
            self.external_invoke,
            self.buffer_size
        );

        unsafe {
            let webview = built.inner_webview();
            ffi::struct_webview_set_title(webview, title)?;
            //TODO: Content
            ffi::struct_webview_set_width(webview, width);
            ffi::struct_webview_set_height(webview, height);
            ffi::struct_webview_set_resizable(webview, resizable);
            ffi::struct_webview_set_debug(webview, debug);

            if has_external_invoke {
                ffi::struct_webview_set_external_invoke_cb::<T, E>(webview);
            }

            ffi::webview_init(webview);
        }

        Ok(built)
    }
}

impl<'title, 'content, E, T> WebviewBuilder<'title, 'content, &'content str, E, T>
where
    E: FnMut(&Webview<T, E>, &str),
    T: Userdata
{
    pub fn set_content_file(mut self, path: &'content impl AsRef<Path>) -> Self {
        match path.as_ref().to_str() {
            Some(path) => self.set_content(Content::File(path)),
            None => {
                self.error = Some(WebviewError::InvalidPath);
                self
            }
        }
    }

}

impl<'title, 'content, C, E, T> Default for WebviewBuilder<'title, 'content, C, E, T>
where
    C: Into<Cow<'content, str>> + 'content,
    E: FnMut(&Webview<T, E>, &str),
    T: Userdata
{
    #[inline]
    fn default() -> Self {
        Self {
            title: None,
            content: None,
            width: None,
            height: None,
            resizeable: true,
            debug: false,
            external_invoke: None,
            userdata: None,
            deactivate_thread_check: false,
            buffer_size: 0,
            error: None,
        }
    }
}

pub enum Content<'c, C>
where
    C: Into<Cow<'c, str>> + 'c
{
    Http(C),
    Https(C),
    File(C),
    Html(C),
    Raw(C),
    #[doc(hidden)]
    __NonExhaustive(PhantomData<&'c str>),
}

impl<'content, C> Content<'content, C>
where
    C: Into<Cow<'content, str>> + 'content
{
    #[inline]
    fn format(self) -> Cow<'content, str> {
        match self {
            Content::Http(content)  => prepend_str("http://", content),
            Content::Https(content) => prepend_str("https://", content),
            Content::File(content)  => prepend_str("file:///", content),
            Content::Html(content)  => prepend_str("data:text/html,", content),
            Content::Raw(content)   => content.into(),
            _                       => panic!("attempted to format internal enum variant"),
        }
    }
}

#[inline]
fn prepend_str<'content, C>(prepend: &str, content: C) -> Cow<'content, str>
where
    C: Into<Cow<'content, str>> + 'content
{
    let mut cow = content.into();
    cow.to_mut().insert_str(0, prepend);

    cow
}