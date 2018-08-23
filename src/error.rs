use std::{error, fmt};

use self::WebviewError::*;
use crate::ffi::CStrConversionError;

#[derive(Debug)]
pub enum WebviewError {
    MissingArgs,
    InvalidPath,
    InvalidStr(CStrConversionError),
    InvalidThread,
}

//TODO: Rewrite error messages
impl fmt::Display for WebviewError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            MissingArgs   => write!(f, "There were missing arguments required to create a Webview"),
            InvalidPath   => write!(f, "There was an invalid file path set for the Webview's initial content"),
            InvalidStr(_) => write!(f, "There was an attempt to send an invalid C string to the webview library"),
            InvalidThread => write!(f, "There was an attempt to create the Webview on a thread other than the main thread.\nIn order to disable this check, you can call `WebviewBuilder::disable_thread_check`"),
        }
    }
}

impl error::Error for WebviewError {}

impl From<CStrConversionError> for WebviewError {
    #[inline]
    fn from(error: CStrConversionError) -> Self {
        WebviewError::InvalidStr(error)
    }
}
