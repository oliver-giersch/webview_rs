use std::error;
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

//TODO: Rewrite error messages
impl fmt::Display for WebviewError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            Build => unimplemented!(),
            DispatchFailed => unimplemented!(),
            Library(ref err) => unimplemented!(),
            InvalidPath => unimplemented!(),
            InvalidStr(ref err) => unimplemented!(),
            InvalidThread => unimplemented!(),
        }
    }
}

impl error::Error for WebviewError {}

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
