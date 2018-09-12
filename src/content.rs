use std::borrow::Cow;
use std::marker::PhantomData;

/// Content Variants
///
/// The initial content for a webview can be either a URL, a filepath or HTML
/// markup.
pub enum Content<'content, C>
where
    C: Into<Cow<'content, str>>,
{
    Url(C),
    File(C),
    Html(C),
    #[doc(hidden)]
    __Hidden(PhantomData<&'content str>),
}

impl<'content, C> Into<Cow<'content, str>> for Content<'content, C>
where
    C: Into<Cow<'content, str>>,
{
    /// Conversion from Content into a string (using the copy-on-write type)
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

/// Convert Content string into correctly formatted URL.
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
