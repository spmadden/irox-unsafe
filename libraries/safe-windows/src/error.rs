use std::ffi::CStr;
use std::fmt::{Display, Formatter};
#[cfg(windows)]
use windows::Win32::Foundation::WIN32_ERROR;
#[cfg(windows)]
use windows::Win32::Networking::WinSock::WSA_ERROR;

pub const WSA_INVALID_HANDLE: i32 = 6;
pub const WSA_INVALID_PARAMETER: i32 = 87;
pub const WSA_E_OPERATION_ABORTED: i32 = 995;
pub const WSA_IO_INCOMPLETE: i32 = 996;
pub const WSA_E_INTR: i32 = 10004;
pub const WSA_E_FAULT: i32 = 10014;
pub const WSA_E_INVAL: i32 = 10022;
pub const WSA_E_WOULDBLOCK: i32 = 10035;
pub const WSA_E_INPROGRESS: i32 = 10036;
pub const WSA_E_NOTSOCK: i32 = 10038;
pub const WSA_E_MSGSIZE: i32 = 10040;
pub const WSA_E_OPNOTSUPP: i32 = 10045;
pub const WSA_E_NETDOWN: i32 = 10050;
pub const WSA_E_NETRESET: i32 = 10052;
pub const WSA_E_CONNABORTED: i32 = 10053;
pub const WSA_E_CONNRESET: i32 = 10054;
pub const WSA_E_NOBUFS: i32 = 10055;
pub const WSA_E_NOTCONN: i32 = 10057;
pub const WSA_E_SHUTDOWN: i32 = 10058;
pub const WSA_NOT_INITIALIZED: i32 = 10093;
#[derive(Debug, Copy, Clone)]
pub enum WinsockErr {
    InvalidHandle,
    InvalidParameter,
}

#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub enum ErrorType {
    NotFound,
    NoConsole,
    Network,
    IOIncomplete,
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
    pub fn no_console<T>() -> Result<T, Error> {
        Err(Error {
            msg: "Console not allocated".to_string(),
            err_type: ErrorType::NoConsole,
        })
    }
    pub fn notfound<T>() -> Result<T, Error> {
        Err(Error {
            msg: "Element not found".to_string(),
            err_type: ErrorType::NotFound,
        })
    }
    pub fn msg(&self) -> &str {
        &self.msg
    }
    pub fn err_type(&self) -> ErrorType {
        self.err_type
    }
    pub fn is_notfound(&self) -> bool {
        self.err_type == ErrorType::NotFound
    }

    pub fn is_ioincomplete(&self) -> bool {
        self.err_type == ErrorType::IOIncomplete
    }

    #[cfg(windows)]
    pub fn win32<T>(err: WIN32_ERROR) -> Result<T, Error> {
        use windows::core::PSTR;
        use windows::Win32::System::Diagnostics::Debug::{
            FormatMessageA, FORMAT_MESSAGE_FROM_SYSTEM,
        };
        let flags = FORMAT_MESSAGE_FROM_SYSTEM;
        let source = None;
        let messageid = err.0;
        let languageid = 1; // sublang_default

        let mut buf = [0u8; 1024];
        let buffer = PSTR(buf.as_mut_ptr());

        let args = None;
        let msg = unsafe {
            let res = FormatMessageA(flags, source, messageid, languageid, buffer, 1024, args);
            if res > 0 {
                CStr::from_bytes_until_nul(&buf)
                    .unwrap_or_default()
                    .to_string_lossy()
            } else {
                Default::default()
            }
        };
        Error::code(err.0, msg.trim())
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
impl_error!(irox::bits::Error, "BitsError");
impl_error!(std::string::FromUtf16Error, "UTF16Error");

#[cfg(windows)]
impl_error!(windows::core::Error, "WindowsErr");

#[cfg(windows)]
impl From<WIN32_ERROR> for Error {
    fn from(value: WIN32_ERROR) -> Self {
        Error {
            msg: format!("Win32 Error: {value:?}"),
            err_type: ErrorType::Other,
        }
    }
}

#[cfg(windows)]
impl From<WSA_ERROR> for Error {
    fn from(value: WSA_ERROR) -> Self {
        match value.0 {
            WSA_NOT_INITIALIZED => Error {
                err_type: ErrorType::Network,
                msg: "WSANOTINITIALIZED: A successful WSAStartup call must occur before using this function".to_string()
            },
            WSA_E_NETDOWN => Error {
                err_type: ErrorType::Network,
                msg: "WSAENETDOWN: The network subsystem has failed".to_string()
            },
            WSA_E_NOTSOCK => Error {
                err_type: ErrorType::Network,
                msg: "WSAENOTSOCK: The descriptor is not a socket".to_string()
            },
            WSA_INVALID_HANDLE => Error {
                err_type: ErrorType::Network,
                msg: "WSA_INVALID_HANDLE: The hEvent parameter of the overlapped structure does not contain a valid object handle".to_string()
            },
            WSA_INVALID_PARAMETER => Error {
                err_type: ErrorType::Network,
                msg: "WSA_INVALID_PARAMETER: One of the parameters is unacceptable".to_string()
            },
            WSA_IO_INCOMPLETE => Error {
                err_type: ErrorType::IOIncomplete,
                msg: "WSA_IO_INCOMPLETE: the fWait parameter is false and the operation has not completed".to_string()
            },
            WSA_E_FAULT => Error {
                err_type: ErrorType::Network,
                msg: "WSAEFAULT: one or more of the pointers are not valid".to_string()
            },
            e => Error {
                err_type: ErrorType::Other,
                msg: format!("Unknown WSA Error code: {e}")
            }
        }
    }
}
