use std::fmt::{Display, Formatter};

#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct Error {
    msg: String,
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
        })
    }
}

macro_rules! impl_error {
    ($ty:path, $pre:literal) => {
        impl From<$ty> for Error {
            fn from(value: $ty) -> Error {
                Error {
                    msg: format!("{}: {}", $pre, value),
                }
            }
        }
    };
}

impl_error!(std::ffi::FromBytesWithNulError, "FFI");
impl_error!(std::ffi::FromBytesUntilNulError, "FFI");
impl_error!(std::ffi::NulError, "FFI");
impl_error!(windows::core::Error, "WindowsErr");
impl_error!(std::io::Error, "IOError");
impl_error!(std::string::FromUtf16Error, "UTF16Error");
