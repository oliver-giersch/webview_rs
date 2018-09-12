//! Module for handling conversion from Rust strings to C strings.
//!
//! Copy-on-write (Cow) type is used extensively to avoid unnecessary
//! allocations. Since C strings have to be nul-terminated (unlike Rust
//! strings), new allocations and reallocations of strings.
//! The `convert_to_cstring` function provides functionality to convert
//! strings which avoids unnecessary allocations where possible and ensures
//! space efficient allocations where required.

use std::borrow::Cow;
use std::error::Error;
use std::ffi::{CStr, CString, FromBytesWithNulError, NulError};
use std::fmt;

/// Possible errors due to conversion to C strings
///
/// Errors occur in case of strings with internal 0-bytes.
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

/// Converts either an owned (`String`) or a referenced (`&str`) string to the
/// equivalent C string
///
/// To avoid unnecessary allocations, `&str` types will be converted to
/// `&CStr`, as long as they are nul-terminated. Otherwise the allocated
/// `String` will be allocated with capacity (string.len() + 1).
/// Owned strings are not reallocated but appended by a nul-byte if neccessary.
///
/// #Errors
///
/// Returns a `CStrConversionError` when a supplied string contains internal
/// nul-bytes.
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

#[cfg(test)]
mod test {
    use std::borrow::Cow;

    use crate::conversion::*;

    #[test]
    fn nul_terminated_str() {
        match convert_to_cstring("nul-terminated\0").unwrap() {
            Cow::Borrowed(ref cstring) => assert_eq!("nul-terminated", cstring.to_str().unwrap()),
            _ => panic!("unnecessary allocation"),
        };
    }

    #[test]
    fn non_nul_terminated_str() {
        let string = "regular rust string";
        match convert_to_cstring(string).unwrap() {
            Cow::Owned(owned) => {
                assert_eq!(owned.to_str().unwrap(), string);
                assert_eq!(string.as_bytes().len() + 1, owned.into_bytes().capacity());
            }
            _ => panic!("non nul-terminated string requires allocation"),
        };
    }

    #[test]
    fn owned_string() {
        let string = String::from("owned string");
        match convert_to_cstring(string).unwrap() {
            Cow::Owned(owned) => assert_eq!(owned.to_str().unwrap(), "owned string"),
            _ => unreachable!(),
        };
    }

    #[test]
    fn conversion_err() {
        assert!(convert_to_cstring("internal \0 bytes").is_err());
        assert!(convert_to_cstring(String::from("internal \0 bytes")).is_err());
    }
}
