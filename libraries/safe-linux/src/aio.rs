// SPDX-License-Identifier: MIT
// Copyright 2024 IROX Contributors
//

use alloc::boxed::Box;
use core::pin::Pin;
use crate::errno::Errno;
use crate::{syscall_1, syscall_2};

const SYSCALL_IO_SETUP: u32 = 209u32;
const SYSCALL_IO_DESTROY: u32 = 207u32;

const IOCB_CMD_PREAD: u16 = 0u16;
const IOCB_CMD_PWRITE: u16 = 1u16;
const IOCB_CMD_FSYNC: u16 = 2u16;
const IOCB_CMD_FDSYNC: u16 = 3u16;
const IOCB_CMD_POLL: u16 = 5u16;
const IOCB_CMD_NOOP: u16 = 6u16;
const IOCB_CMD_PREADV: u16 = 7u16;
const IOCB_CMD_PWRITEV: u16 = 8u16;

pub struct AioContext(u32);
impl Drop for AioContext {
    fn drop(&mut self) {
        unsafe {
            io_destroy(self.0).unwrap()
        }
    }
}
impl AioContext {
    pub fn new(max_num_events: u32) -> Result<Self, Errno> {
        unsafe {
            io_setup(max_num_events)
        }
    }

    // pub fn write(&self, )
}

pub struct AioData {
    data: Pin<Box<[u8]>>,

}

pub unsafe fn io_setup(max_num_events: u32) -> Result<AioContext, Errno> {
    let mut out = 0u32;
    let ptr = core::ptr::from_mut(&mut out);
    let res = syscall_2!(SYSCALL_IO_SETUP, max_num_events, ptr);

    if res < 0 {
        return Err(res.into())
    }
    Ok(AioContext(out))
}

pub unsafe fn io_destroy(context: u32) -> Result<(), Errno> {
    let res = syscall_1!(SYSCALL_IO_DESTROY, context);
    if res < 0 {
        return Err(res.into())
    }
    Ok(())
}

