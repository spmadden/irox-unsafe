use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum ErrorType {
    NotFound,
    #[default]
    Other,
}

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Error {
    msg: String,
    err_type: ErrorType,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: {}", self.msg)
    }
}

impl std::error::Error for Error {}

impl Error {
    pub fn code<T>(code: u32, message: &str) -> Result<T, Error> {
        Err(Error {
            msg: format!("Error({code}): {message}"),
            err_type: ErrorType::Other,
        })
    }
    pub fn notfound<T>() -> Result<T, Error> {
        Err(Error {
            msg: "Element not found".to_string(),
            err_type: ErrorType::NotFound,
        })
    }
    pub fn is_notfound(&self) -> bool {
        self.err_type == ErrorType::NotFound
    }
}

macro_rules! impl_error {
    ($ty:path, $pre:literal) => {
        impl From<$ty> for Error {
            fn from(value: $ty) -> Error {
                Error {
                    msg: format!("{}: {}", $pre, value),
                    err_type: ErrorType::Other,
                }
            }
        }
    };
}

impl_error!(std::ffi::FromBytesWithNulError, "FFI");
impl_error!(std::ffi::FromBytesUntilNulError, "FFI");
impl_error!(std::ffi::NulError, "FFI");
impl_error!(std::io::Error, "IOError");
impl_error!(std::string::FromUtf16Error, "UTF16Error");

#[cfg(windows)]
impl_error!(windows::core::Error, "WindowsErr");
