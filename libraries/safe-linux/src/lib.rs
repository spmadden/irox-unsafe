// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors

//!
//! Linux x86_64 System Calls (SYSCALLs)
//!

#![cfg_attr(not(test), no_std)]

//#[cfg(any(not(target_os = "linux"), not(target_arch = "x86_64")))]
//compile_error!("Only linux x86_64 currently supported.  Submit a PR!");

pub mod errno;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub mod sys;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub mod syscall;
#[cfg(all(target_os = "linux", target_arch = "x86_64"))]
pub mod time;
