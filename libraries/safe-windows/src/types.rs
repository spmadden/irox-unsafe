// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use windows::Win32::NetworkManagement::WiFi::DOT11_SSID;
use windows_core::{PCWSTR, PWSTR};

pub trait SaferToString {
    fn to_string_safer(&self) -> String;
}

impl SaferToString for PCWSTR {
    fn to_string_safer(&self) -> String {
        if self.is_null() {
            return String::new();
        }
        unsafe { self.to_string().unwrap_or_default() }
    }
}
impl SaferToString for PWSTR {
    fn to_string_safer(&self) -> String {
        if self.is_null() {
            return String::new();
        }
        unsafe { self.to_string().unwrap_or_default() }
    }
}

impl SaferToString for DOT11_SSID {
    fn to_string_safer(&self) -> String {
        if self.uSSIDLength == 0 {
            return String::new();
        }
        let data = self.ucSSID.get(0..(self.uSSIDLength as usize)).unwrap_or_default();
        let mut out = String::default();
        for d in data {
            if *d == 0 {
                break;
            }

            out.push(*d as char);
        }
        out
    }
}
