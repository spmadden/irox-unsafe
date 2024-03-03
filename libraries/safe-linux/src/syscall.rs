// SPDX-License-Identifier: MIT
// Copyright 2024 IROX Contributors

//!
//! Core syscall structures & macros
//!
//!  *
//!  * Registers on entry:
//!  * rax  system call number
//!  * rcx  return address
//!  * r11  saved rflags (note: r11 is callee-clobbered register in C ABI)
//!  * rdi  arg0
//!  * rsi  arg1
//!  * rdx  arg2
//!  * r10  arg3 (needs to be moved to rcx to conform to C ABI)
//!  * r8   arg4
//!  * r9   arg5
//!  * (note: r12-r15, rbp, rbx are callee-preserved in C ABI)
//!

use core::arch::asm;
pub unsafe fn syscall_x64_6(
    num: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
    arg5: u64,
) -> u64 {
    let mut ret: u64;
    asm!(
    "syscall",
    in("rax") num,
    in("rdi") arg0,
    in("rsi") arg1,
    in("rdx") arg2,
    in("r10") arg3, // this is usually rcx
    in("r8") arg4,
    in("r9") arg5,
    out("rcx") _, // clobber return address in rcx
    out("r11") _, // clobber saved rflags in r11
    lateout("rax") ret, // return value
    );
    ret
}
pub unsafe fn syscall_x64_5(
    num: u64,
    arg0: u64,
    arg1: u64,
    arg2: u64,
    arg3: u64,
    arg4: u64,
) -> u64 {
    let mut ret: u64;
    asm!(
    "syscall",
    in("rax") num,
    in("rdi") arg0,
    in("rsi") arg1,
    in("rdx") arg2,
    in("r10") arg3, // this is usually rcx
    in("r8") arg4,
    out("rcx") _, // clobber return address in rcx
    out("r11") _, // clobber saved rflags in r11
    lateout("rax") ret, // return value
    );
    ret
}
pub unsafe fn syscall_x64_4(num: u64, arg0: u64, arg1: u64, arg2: u64, arg3: u64) -> u64 {
    let mut ret: u64;
    asm!(
    "syscall",
    in("rax") num,
    in("rdi") arg0,
    in("rsi") arg1,
    in("rdx") arg2,
    in("r10") arg3, // this is usually rcx
    out("rcx") _, // clobber return address in rcx
    out("r11") _, // clobber saved rflags in r11
    lateout("rax") ret, // return value
    );
    ret
}
pub unsafe fn syscall_x64_3(num: u64, arg0: u64, arg1: u64, arg2: u64) -> u64 {
    let mut ret: u64;
    asm!(
    "syscall",
    in("rax") num,
    in("rdi") arg0,
    in("rsi") arg1,
    in("rdx") arg2,
    out("rcx") _, // clobber return address in rcx
    out("r11") _, // clobber saved rflags in r11
    lateout("rax") ret, // return value
    );
    ret
}
pub unsafe fn syscall_x64_2(num: u64, arg0: u64, arg1: u64) -> u64 {
    let mut ret: u64;
    asm!(
    "syscall",
    in("rax") num,
    in("rdi") arg0,
    in("rsi") arg1,
    out("rcx") _, // clobber return address in rcx
    out("r11") _, // clobber saved rflags in r11
    lateout("rax") ret, // return value
    );
    ret
}

pub unsafe fn syscall_x64_1(num: u64, arg1: u64) -> u64 {
    let mut ret: u64;
    asm!(
    "syscall",
    in("rax") num,
    in("rdi") arg1,
    out("rcx") _, // clobber return address in rcx
    out("r11") _, // clobber saved rflags in r11
    lateout("rax") ret, // return value
    );
    ret
}

pub unsafe fn syscall_x64_0(num: u64) -> u64 {
    let mut ret: u64;
    asm!(
        "syscall",
        in("rax") num,
        out("rcx") _, // clobber return address in rcx
        out("r11") _, // clobber saved rflags in r11
        lateout("rax") ret, // return value
    );
    ret
}

#[macro_export]
macro_rules! syscall_1 {
    ($num:ident, $arg0:expr) => {
        {
            let mut ret: i64;
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $arg0,
                out("rcx") _, // clobber return address in rcx
                out("r11") _, // clobber saved rflags in r11
                lateout("rax") ret, // return value
                );
            ret
        }
    };
}
#[macro_export]
macro_rules! syscall_2 {
    ($num:ident, $arg0:expr, $arg1:expr) => {
        {
            let mut ret: i64;
            core::arch::asm!(
                "syscall",
                in("rax") $num,
                in("rdi") $arg0,
                in("rsi") $arg1,
                out("rcx") _, // clobber return address in rcx
                out("r11") _, // clobber saved rflags in r11
                lateout("rax") ret, // return value
                );
            ret
        }
    };
}
