// SPDX-License-Identifier: MIT
// Copyright 2024 IROX Contributors
//

//!
//! Error Numbers (ERRNOs)

use crate::time::EnumName;
use core::fmt::{Debug, Formatter};

#[derive(Copy, Clone, Eq, PartialEq, EnumName)]
pub enum Errno {
    /// Operation not permitted
    EPERM,
    /// No such file or directory
    ENOENT,
    /// No such process
    ESRCH,
    /// Interrupted system call
    EINTR,
    /// I/O Error
    EIO,
    /// No such device or address
    ENXIO,
    /// Argument list too long
    E2BIG,
    /// Exec format error
    ENOEXEC,
    /// Bad file number
    EBADF,
    /// No child processes
    ECHILD,
    /// Try again
    EGAGAIN,
    /// Out of memory
    ENOMEM,
    /// Permission denied
    EACCES,
    /// Bad address
    EFAULT,
    /// Block device required
    ENOTBLK,
    /// Device or resource busy
    EBUSY,
    /// File exists
    EEXIST,
    /// Cross-device link
    EXDEV,
    /// No such device
    ENODEV,
    /// Not a directory
    ENOTDIR,
    /// Is a directory
    EISDIR,
    /// Invalid argument
    EINVAL,
    /// File table overflow
    ENFILE,
    /// Too many open files
    EMFILE,
    /// Not a typewriter
    ENOTTY,
    /// Text file busy
    ETXTBSY,
    /// File too large
    EFBIG,
    /// No space left on device
    ENOSPC,
    /// Illegal seek
    ESPIPE,
    /// Read-only file system
    EROFS,
    /// Too many links
    EMLINK,
    /// Broken pipe
    EPIPE,
    /// Math argument out of domain of func
    EDOM,
    /// Math result not representable
    ERANGE,

    UNK(i64),
}

impl Debug for Errno {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(
            f,
            "{:?}({}): {}",
            self.name(),
            self.code(),
            self.description()
        )
    }
}

impl Errno {
    pub fn code(&self) -> i64 {
        match self {
            Errno::EPERM => -1,
            Errno::ENOENT => -2,
            Errno::ESRCH => -3,
            Errno::EINTR => -4,
            Errno::EIO => -5,
            Errno::ENXIO => -6,
            Errno::E2BIG => -7,
            Errno::ENOEXEC => -8,
            Errno::EBADF => -9,
            Errno::ECHILD => -10,
            Errno::EGAGAIN => -11,
            Errno::ENOMEM => -12,
            Errno::EACCES => -13,
            Errno::EFAULT => -14,
            Errno::ENOTBLK => -15,
            Errno::EBUSY => -16,
            Errno::EEXIST => -17,
            Errno::EXDEV => -18,
            Errno::ENODEV => -19,
            Errno::ENOTDIR => -20,
            Errno::EISDIR => -21,
            Errno::EINVAL => -22,
            Errno::ENFILE => -23,
            Errno::EMFILE => -24,
            Errno::ENOTTY => -25,
            Errno::ETXTBSY => -26,
            Errno::EFBIG => -27,
            Errno::ENOSPC => -28,
            Errno::ESPIPE => -29,
            Errno::EROFS => -30,
            Errno::EMLINK => -31,
            Errno::EPIPE => -32,
            Errno::EDOM => -33,
            Errno::ERANGE => -34,
            Errno::UNK(e) => *e,
        }
    }
    pub fn description(&self) -> &'static str {
        match self {
            Errno::EPERM => "Operation not permitted",

            Errno::ENOENT => "No such file or directory",
            Errno::ESRCH => "No such process",
            Errno::EINTR => "Interrupted system call",
            Errno::EIO => "I/O Error",
            Errno::ENXIO => "No such device or address",
            Errno::E2BIG => "Argument list too long",
            Errno::ENOEXEC => "Exec format error",
            Errno::EBADF => "Bad file number",
            Errno::ECHILD => "No child processes",
            Errno::EGAGAIN => "Try again",
            Errno::ENOMEM => "Out of memory",
            Errno::EACCES => "Permission denied",
            Errno::EFAULT => "Bad address",
            Errno::ENOTBLK => "Block device required",
            Errno::EBUSY => "Device or resource busy",
            Errno::EEXIST => "File exists",
            Errno::EXDEV => "Cross-device list",
            Errno::ENODEV => "No such device",
            Errno::ENOTDIR => "Not a directory",
            Errno::EISDIR => "Is a directory",
            Errno::EINVAL => "Invalid argument",
            Errno::ENFILE => "File table overflow",
            Errno::EMFILE => "Too many open files",
            Errno::ENOTTY => "Not a typewriter",
            Errno::ETXTBSY => "Text file bsy",
            Errno::EFBIG => "File too large",
            Errno::ENOSPC => "No space left on device",
            Errno::ESPIPE => "Illegal seek",
            Errno::EROFS => "Read-only file system",
            Errno::EMLINK => "Too many links",
            Errno::EPIPE => "Broken pipe",
            Errno::EDOM => "Math argument out of domain of func",
            Errno::ERANGE => "Math result not representable",
            Errno::UNK(_) => "Unknown error code",
        }
    }
}

impl From<i64> for Errno {
    fn from(value: i64) -> Self {
        match value {
            -1 | 1 => Errno::EPERM,
            -2 | 2 => Errno::ENOENT,
            -3 | 3 => Errno::ESRCH,
            -4 | 4 => Errno::EINTR,
            -5 | 5 => Errno::EIO,
            -6 | 6 => Errno::ENXIO,
            -7 | 7 => Errno::E2BIG,
            -8 | 8 => Errno::ENOEXEC,
            -9 | 9 => Errno::EBADF,
            -10 | 10 => Errno::ECHILD,
            -11 | 11 => Errno::EGAGAIN,
            -12 | 12 => Errno::ENOMEM,
            -13 | 13 => Errno::EACCES,
            -14 | 14 => Errno::EFAULT,
            -15 | 15 => Errno::ENOTBLK,
            -16 | 16 => Errno::EBUSY,
            -17 | 17 => Errno::EEXIST,
            -18 | 18 => Errno::EXDEV,
            -19 | 19 => Errno::ENODEV,
            -20 | 20 => Errno::ENOTDIR,
            -21 | 21 => Errno::EISDIR,
            -22 | 22 => Errno::EINVAL,
            -23 | 23 => Errno::ENFILE,
            -24 | 24 => Errno::EMFILE,
            -25 | 25 => Errno::ENOTTY,
            -26 | 26 => Errno::ETXTBSY,
            -27 | 27 => Errno::EFBIG,
            -28 | 28 => Errno::ENOSPC,
            -29 | 29 => Errno::ESPIPE,
            -30 | 30 => Errno::EROFS,
            -31 | 31 => Errno::EMLINK,
            -32 | 32 => Errno::EPIPE,
            -33 | 33 => Errno::EDOM,
            -34 | 34 => Errno::ERANGE,

            e => Errno::UNK(e),
        }
    }
}
impl From<Errno> for i64 {
    fn from(value: Errno) -> Self {
        value.code()
    }
}

impl<T> From<Errno> for Result<T, Errno> {
    fn from(value: Errno) -> Self {
        Err(value)
    }
}
