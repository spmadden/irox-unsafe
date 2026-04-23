// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors

//!
//! Safe(r) wrappers around the unsafe windows API functions.
//!

#![allow(non_snake_case)]
extern crate core;

pub mod certs;
pub mod error;
pub mod aad;
pub mod types;
pub mod wlan;

irox::tools::cfg_windows! {
    pub mod credentials;
    pub mod fs;
    pub mod net;
    pub mod net_if;
    pub mod priority;
    pub mod registry;
    pub mod smbios;
    pub mod system;
    pub mod term;
    pub mod webauthn;
    pub mod packages;
}
