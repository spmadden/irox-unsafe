// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors

//!
//! Intel x86-64 Architecture-specific bits & bobs
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(target_arch = "x86_64")]
pub mod cpu;
#[cfg(target_arch = "x86_64")]
pub mod rand;
