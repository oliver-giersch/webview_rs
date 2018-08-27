use std::error;
use std::fmt;

use self::WebviewError::*;
use crate::conversion::CStrConversionError;
use crate::ffi::{FFIError, LibraryError};

#[derive(Debug)]
pub enum WebviewError {
    Build,
    DispatchFailed,
    Internal(LibraryError),
    InvalidPath,
    InvalidStr(CStrConversionError),
    InvalidThread,
}

//TODO: Rewrite error messages
impl fmt::Display for WebviewError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Build => unimplemented!(),
            DispatchFailed => unimplemented!(),
            Internal(err) => unimplemented!(),
            InvalidPath => unimplemented!(),
            InvalidStr(err) => unimplemented!(),
            InvalidThread => unimplemented!(),
        }
    }
}

impl error::Error for WebviewError {}

impl From<LibraryError> for WebviewError {
    #[inline]
    fn from(error: LibraryError) -> Self {
        WebviewError::Internal(error)
    }
}

impl From<CStrConversionError> for WebviewError {
    #[inline]
    fn from(error: CStrConversionError) -> Self {
        WebviewError::InvalidStr(error)
    }
}


