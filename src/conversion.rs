use std::borrow::Cow;
use std::error::Error;
use std::ffi::{CStr, CString, FromBytesWithNulError, NulError};
use std::fmt;

#[derive(Debug, Clone)]
pub enum CStrConversionError {
    FromBytesWithNul(FromBytesWithNulError),
    Nul(NulError),
}

impl fmt::Display for CStrConversionError {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            CStrConversionError::FromBytesWithNul(ref err) => write!(f, "{}", err.description()),
            CStrConversionError::Nul(ref err) => write!(f, "{}", err.description()),
        }
    }
}

impl Error for CStrConversionError {}

impl From<FromBytesWithNulError> for CStrConversionError {
    #[inline]
    fn from(err: FromBytesWithNulError) -> Self {
        CStrConversionError::FromBytesWithNul(err)
    }
}

impl From<NulError> for CStrConversionError {
    #[inline]
    fn from(err: NulError) -> Self {
        CStrConversionError::Nul(err)
    }
}

pub fn convert_to_cstring<'s>(
    string: impl Into<Cow<'s, str>>,
) -> Result<Cow<'s, CStr>, CStrConversionError> {
    match string.into() {
        Cow::Borrowed(ref string) => {
            if string.ends_with('\0') {
                let cstr = CStr::from_bytes_with_nul(string.as_bytes())?;
                Ok(Cow::from(cstr))
            } else {
                let mut buffer = String::with_capacity(string.len() + 1);
                buffer.push_str(string);

                let cstring = CString::new(buffer)?;
                Ok(Cow::from(cstring))
            }
        }
        Cow::Owned(string) => {
            let cstring = CString::new(string)?;
            Ok(Cow::from(cstring))
        }
    }
}
