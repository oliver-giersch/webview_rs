#[derive(Debug, Clone)]
pub enum CStrConversionError {
    FromBytesWithNul(FromBytesWithNulError),
    Nul(NulError),
}

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

use std::borrow::Cow;
use std::ffi::{CStr, CString, FromBytesWithNulError, NulError};

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