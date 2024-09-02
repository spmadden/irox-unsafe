// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

#![allow(non_camel_case_types)]

use crate::error::Error;
use std::ffi::{c_void, CString};
use std::future::Future;
use std::num::NonZeroU8;
use std::os::windows::raw::HANDLE;
use std::path::Path;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use windows::Win32::Foundation::{
    ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, GENERIC_READ, GENERIC_WRITE,
};
use windows::Win32::Storage::FileSystem::{
    FILE_ATTRIBUTE_NORMAL, FILE_FLAG_NO_BUFFERING, FILE_FLAG_OVERLAPPED, FILE_FLAG_WRITE_THROUGH,
    FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_ALWAYS,
};
use windows::Win32::System::Threading::CreateEventA;
use windows::Win32::System::IO::{OVERLAPPED, OVERLAPPED_0, OVERLAPPED_0_0};

pub type DWORD = u32;
pub type LPSTR = *mut i8;
pub type LPCSTR = *const i8;
pub type LPSECURITY_ATTRIBUTES = *mut c_void;
pub type BOOL = i32;
pub const TRUE: BOOL = 1;
pub const FALSE: BOOL = 0;

#[link(name = "kernel32")]
extern "system" {
    pub fn CloseHandle(h: HANDLE) -> BOOL;
    pub fn CreateFileA(
        lpFileName: LPCSTR,
        dwDesiredAccess: DWORD,
        dwShareMode: DWORD,
        lpSecurityAttributes: LPSECURITY_ATTRIBUTES,
        dwCreationDisposition: DWORD,
        dwFlagsAndAttributes: DWORD,
        hTemplateFile: HANDLE,
    ) -> HANDLE;
    fn ReadFile(
        hfile: HANDLE,
        lpbuffer: *mut u8,
        nnumberofbytestoread: u32,
        lpnumberofbytesread: *mut u32,
        lpoverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    fn WriteFile(
        hfile: HANDLE,
        lpbuffer: *const u8,
        number_to_write: u32,
        num_written: *mut u32,
        lpoverlapped: *mut OVERLAPPED,
    ) -> BOOL;
    fn GetOverlappedResult(
        hfile: HANDLE,
        lpoverlapped: *mut OVERLAPPED,
        lpnumberofbytestransferred: *mut u32,
        bwait: BOOL,
    ) -> BOOL;
    fn GetLastError() -> u32;
}

struct AsyncFileInner {
    handle: HANDLE,
}
impl Drop for AsyncFileInner {
    fn drop(&mut self) {
        let ret = unsafe { CloseHandle(self.handle) };
        if ret == FALSE {
            // check?
        }
    }
}
unsafe impl Send for AsyncFileInner {}
unsafe impl Sync for AsyncFileInner {}

#[derive(Clone)]
pub struct AsyncFile {
    inner: Arc<AsyncFileInner>,
}

impl AsyncFile {
    pub fn open<T: AsRef<Path>>(path: T) -> Result<AsyncFile, Error> {
        let path = path.as_ref().to_string_lossy().bytes().filter_map(NonZeroU8::new).collect::<Vec<_>>();
        let path = CString::from(path);
        let security = std::ptr::null_mut();
        let creation_disposition = OPEN_ALWAYS.0;
        let share_mode = FILE_SHARE_READ.0 | FILE_SHARE_WRITE.0;
        let flags_and_attrs = FILE_ATTRIBUTE_NORMAL.0
                | FILE_FLAG_OVERLAPPED.0
                // | FILE_FLAG_RANDOM_ACCESS.0
                | FILE_FLAG_WRITE_THROUGH.0
                | FILE_FLAG_NO_BUFFERING.0;
        let handle_template = std::ptr::null_mut();
        let desired_access = GENERIC_READ.0 | GENERIC_WRITE.0;
        let handle = unsafe {
            CreateFileA(
                path.as_ptr(),
                desired_access,
                share_mode,
                security,
                creation_disposition,
                flags_and_attrs,
                handle_template,
            )
        };
        let haddr = handle.cast::<()>() as usize;
        if haddr == 0 || haddr == usize::MAX {
            let err = unsafe { GetLastError() };
            return Error::code(err, "Error opening file");
        }

        Ok(AsyncFile {
            inner: Arc::new(AsyncFileInner { handle }),
        })
    }
}

pub struct FileIOFuture {
    _buffer: Pin<Box<[u8]>>,
    _offset: u64,
    handle: HANDLE,
    overlapped: Pin<Box<OVERLAPPED>>,
    done: Option<u32>,
}
unsafe impl Send for FileIOFuture {}
impl Future for FileIOFuture {
    type Output = Result<u32, Error>;

    fn poll(mut self: Pin<&mut Self>, ctx: &mut Context<'_>) -> Poll<Self::Output> {
        if let Some(done) = &self.done {
            return Poll::Ready(Ok(*done));
        }
        let mut transferred: u32 = 0;
        let ret = unsafe {
            GetOverlappedResult(
                self.handle,
                self.overlapped.as_mut().get_mut(),
                &mut transferred,
                FALSE,
            )
        };
        if ret == 0 {
            let error = unsafe { GetLastError() };
            if error == ERROR_IO_INCOMPLETE.0 {
                ctx.waker().wake_by_ref();
                return Poll::Pending;
            }
            self.done = Some(0);
            return Poll::Ready(Error::code(error, "Error reading file"));
        }
        self.done = Some(transferred);
        Poll::Ready(Ok(transferred))
    }
}
impl Drop for FileIOFuture {
    fn drop(&mut self) {
        let ret = unsafe { CloseHandle(self.overlapped.hEvent.0 as HANDLE) };
        if ret == FALSE {
            // check?
        }
    }
}

pub trait AsyncFileExt {
    fn seek_read<const N: usize>(
        &self,
        buf: &mut [u8; N],
        offset: u64,
    ) -> Result<FileIOFuture, Error>;

    fn seek_write(&self, buf: Box<[u8]>, offset: u64) -> Result<FileIOFuture, Error>;
}

impl AsyncFileExt for AsyncFile {
    fn seek_read<const N: usize>(
        &self,
        buf: &mut [u8; N],
        offset: u64,
    ) -> Result<FileIOFuture, Error> {
        unsafe {
            let hEvent = CreateEventA(
                None,  //lpEventAttributes,
                true,  //bManualReset,
                false, //bInitialState,
                None,  //lpName
            )?;
            if hEvent.is_invalid() {
                return Error::notfound();
            }
            let mut overlapped = OVERLAPPED {
                Internal: 0,
                InternalHigh: 0,
                Anonymous: OVERLAPPED_0 {
                    Anonymous: OVERLAPPED_0_0 {
                        Offset: offset as u32,
                        OffsetHigh: (offset >> 32) as u32,
                    },
                },
                hEvent,
            };
            let handle = self.inner.handle;
            let res = ReadFile(
                handle,
                buf.as_mut_ptr(),
                buf.len() as u32,
                ::core::ptr::null_mut::<u32>(),
                &mut overlapped,
            );
            if res == FALSE {
                let err = GetLastError();
                return Error::code(err, "Error writing file");
            }
            todo!()
            // Ok(FileIOFuture {
            //     overlapped,
            //     _offset: offset,
            //     handle,
            //     _buffer: buf,
            // })
        }
    }

    fn seek_write(&self, buf: Box<[u8]>, offset: u64) -> Result<FileIOFuture, Error> {
        unsafe {
            let boxed = Pin::from(buf);
            let hEvent = CreateEventA(
                None,  //lpEventAttributes,
                true,  //bManualReset,
                false, //bInitialState,
                None,  //lpName
            )?;
            let overlapped = OVERLAPPED {
                Internal: 0,
                InternalHigh: 0,
                Anonymous: OVERLAPPED_0 {
                    Anonymous: OVERLAPPED_0_0 {
                        Offset: offset as u32,
                        OffsetHigh: (offset >> 32) as u32,
                    },
                },
                hEvent,
            };
            let mut overlapped = Box::pin(overlapped);
            let handle = self.inner.handle;
            let number_to_write = boxed.len() as u32;
            let num_written = core::ptr::null_mut();
            let res = WriteFile(
                handle,
                boxed.as_ptr(),
                number_to_write,
                num_written,
                overlapped.as_mut().get_mut(),
            );
            if res == FALSE {
                let err = GetLastError();
                if err != ERROR_IO_PENDING.0 && err != ERROR_IO_INCOMPLETE.0 {
                    return Error::code(err, "Error writing file");
                }
            }
            Ok(FileIOFuture {
                overlapped,
                _offset: offset,
                handle,
                _buffer: boxed,
                done: None,
            })
        }
    }
}
