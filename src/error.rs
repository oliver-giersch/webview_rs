use std::error::Error;
use std::ffi::{FromBytesWithNulError, NulError};
use std::fmt;

use self::WebviewError::*;
use crate::conversion::CStrConversionError;
use crate::ffi::LibraryError;

#[derive(Debug)]
pub enum WebviewError {
    Build,
    DispatchFailed,
    Library(LibraryError),
    InvalidPath,
    InvalidStr(CStrConversionError),
    InvalidThread,
}

impl fmt::Display for WebviewError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Build => write!(f, "failed to to build webview due to missing required arguments"),
            DispatchFailed => write!(f, "failed to dispatch callback from thread (main thread handle no longer exists)"),
            Library(ref err) => write!(f, "webview C library: {}", err.description()),
            InvalidPath => unimplemented!(), //TODO: Write error message
            InvalidStr(ref err) => write!(f, "string conversion error: {}", err.description()),
            InvalidThread => write!(
                f, "failed to start webview: Attempt to run on thread other than `main` \
                (check can be disabled by calling `Builder::deactivate_thread_check`)"
            ),
        }
    }
}

impl Error for WebviewError {}

impl From<LibraryError> for WebviewError {
    #[inline]
    fn from(error: LibraryError) -> Self {
        WebviewError::Library(error)
    }
}

impl From<CStrConversionError> for WebviewError {
    #[inline]
    fn from(error: CStrConversionError) -> Self {
        WebviewError::InvalidStr(error)
    }
}

impl From<FromBytesWithNulError> for WebviewError {
    #[inline]
    fn from(error: FromBytesWithNulError) -> Self {
        WebviewError::InvalidStr(CStrConversionError::from(error))
    }
}

impl From<NulError> for WebviewError {
    #[inline]
    fn from(error: NulError) -> Self {
        WebviewError::InvalidStr(CStrConversionError::from(error))
    }
}
