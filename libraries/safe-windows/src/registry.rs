// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use crate::error::Error;
use std::ffi::{CStr, CString};
use std::path::Path;
use windows::core::PCSTR;
use windows::Win32::Foundation::ERROR_NO_MORE_ITEMS;
use windows::Win32::System::Registry::{
    RegEnumKeyA, RegOpenKeyA, RegQueryValueExA, HKEY, HKEY_CLASSES_ROOT, HKEY_CURRENT_CONFIG,
    HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, HKEY_PERFORMANCE_DATA, HKEY_USERS,
};

#[derive(Debug, Copy, Clone)]
pub enum SystemHives {
    ClassesRoot,
    CurrentUser,
    LocalMachine,
    CurrentConfig,
    PerformanceData,
    Users,
}
impl SystemHives {
    fn get_hkey(self) -> HKEY {
        match self {
            SystemHives::ClassesRoot => HKEY_CLASSES_ROOT,
            SystemHives::CurrentUser => HKEY_CURRENT_USER,
            SystemHives::LocalMachine => HKEY_LOCAL_MACHINE,
            SystemHives::CurrentConfig => HKEY_CURRENT_CONFIG,
            SystemHives::PerformanceData => HKEY_PERFORMANCE_DATA,
            SystemHives::Users => HKEY_USERS,
        }
    }
    pub fn name(&self) -> &str {
        match self {
            SystemHives::ClassesRoot => "HKCR",
            SystemHives::CurrentUser => "HKCU",
            SystemHives::LocalMachine => "HKLM",
            SystemHives::CurrentConfig => "HKCC",
            SystemHives::PerformanceData => "HKPD",
            SystemHives::Users => "HKU",
        }
    }
}
impl From<SystemHives> for RegKey {
    fn from(value: SystemHives) -> Self {
        RegKey {
            name: value.name().to_string(),
            path: String::new(),
            key: value.get_hkey().into(),
            hkey: Some(value.get_hkey()),
        }
    }
}
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct KEY(isize);
impl From<SystemHives> for KEY {
    fn from(value: SystemHives) -> Self {
        KEY(value.get_hkey().0)
    }
}
impl From<HKEY> for KEY {
    fn from(value: HKEY) -> Self {
        KEY(value.0)
    }
}
impl From<KEY> for HKEY {
    fn from(value: KEY) -> Self {
        HKEY(value.0)
    }
}
#[derive(Debug, Clone)]
pub struct RegKey {
    key: KEY,
    name: String,
    path: String,
    hkey: Option<HKEY>,
}
impl RegKey {
    pub fn try_open_subkey(&mut self, subkey: &str) -> Result<RegKey, Error> {
        let hkey = open_subkey(self.key, subkey)?;
        Ok(RegKey {
            key: hkey.into(),
            hkey: Some(hkey),
            name: subkey.to_string(),
            path: format!("{}/{}", self.path, subkey),
        })
    }

    fn get_hkey(&mut self) -> Result<HKEY, Error> {
        Ok(match self.hkey {
            Some(hkey) => hkey,
            None => {
                let out = open_subkey(self.key, &self.name)?;
                self.hkey = Some(out);
                out
            }
        })
    }

    pub fn try_get_value(&mut self, subkey: &str) -> Result<String, Error> {
        let hkey = self.get_hkey()?;
        let str = CString::new(subkey.as_bytes()).unwrap_or_default();
        let str = PCSTR(str.as_ptr() as *const u8);
        let mut out = [0u8; 1024];
        let mut sz = 1024;
        unsafe {
            let res =
                RegQueryValueExA(hkey, str, None, None, Some(out.as_mut_ptr()), Some(&mut sz));
            if res.is_err() {
                return Error::win32(res);
            }
        }
        Ok(CStr::from_bytes_until_nul(out.as_slice())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string())
    }

    pub fn list_subkeys(&mut self) -> Result<Vec<RegKey>, Error> {
        let hkey = self.get_hkey()?;
        Ok(list_registry_keys_inner(hkey.into(), Some(&self.name)))
    }
}
fn open_subkey(key: KEY, name: &str) -> Result<HKEY, Error> {
    let hkey: HKEY = key.into();
    let str = CString::new(name.as_bytes()).unwrap_or_default();
    let str = PCSTR(str.as_ptr() as *const u8);
    let mut out: HKEY = HKEY::default();
    unsafe {
        let res = RegOpenKeyA(hkey, str, &mut out);
        if res.is_err() {
            return Error::win32(res);
        }
    };
    Ok(out)
}
fn list_registry_keys_inner(key: KEY, parent: Option<&str>) -> Vec<RegKey> {
    let mut out = Vec::new();
    let mut idx = 0;
    loop {
        let mut name = [0u8; 1024];
        let key: HKEY = key.into();
        unsafe {
            let res = RegEnumKeyA(key, idx, Some(&mut name));
            if res == ERROR_NO_MORE_ITEMS || res.is_err() {
                break;
            }
        }
        let name = CStr::from_bytes_until_nul(name.as_slice())
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let path = if let Some(parent) = parent {
            format!("{parent}/{name}")
        } else {
            name.clone()
        };
        out.push(RegKey {
            key: key.into(),
            name,
            path,
            hkey: None,
        });
        idx += 1;
    }
    out
}

pub fn find_key<T: AsRef<Path>>(root: SystemHives, path: T) -> Option<RegKey> {
    let mut current_key: RegKey = root.into();

    for elem in path.as_ref().components() {
        let Some(v) = elem.as_os_str().to_str() else {
            break;
        };
        let Ok(key) = current_key.try_open_subkey(v) else {
            return None;
        };
        current_key = key;
    }

    Some(current_key)
}

#[cfg(test)]
mod test {
    use crate::registry::{find_key, SystemHives};

    #[test]
    pub fn list_keys() {
        let mut key = find_key(SystemHives::CurrentUser, "Software/Valve/Steam").unwrap();
        println!("{:?}", key);
        let val = key.try_get_value("SteamPath").unwrap();
        println!("{:?}", val);
    }
}
