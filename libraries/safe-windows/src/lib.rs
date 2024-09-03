// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors

//!
//! Safe(r) wrappers around the unsafe windows API functions.
//!

#![allow(non_snake_case)]
extern crate core;

#[cfg(windows)]
pub mod credentials;
pub mod error;
#[cfg(windows)]
pub mod fs;
#[cfg(windows)]
pub mod net;
pub mod net_if;
#[cfg(windows)]
pub mod smbios;
#[cfg(windows)]
pub mod system;
#[cfg(windows)]
pub mod term;
