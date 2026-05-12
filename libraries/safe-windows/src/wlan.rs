// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use crate::error::Error;
use crate::types::SaferToString;
use core::slice;
use std::collections::{HashMap};
use irox::time::datetime::UTCDateTime;
use irox::time::epoch::{FromTimestamp, UnixTimestamp, WindowsNTTimestamp};
use irox::time::Duration;
use std::fmt::{Debug, Display, Formatter};
use std::io::Write;
use std::ops::Deref;
use std::sync::{Arc, Condvar, Mutex, RwLock};
use irox::tools::static_init;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::NetworkManagement::WiFi::{WlanCloseHandle, WlanEnumInterfaces, WlanFreeMemory, WlanGetNetworkBssList, WlanOpenHandle, WlanRegisterNotification, WlanScan, DOT11_BSS_TYPE, DOT11_SSID, L2_NOTIFICATION_DATA, WLAN_BSS_ENTRY, WLAN_BSS_LIST, WLAN_INTERFACE_INFO_LIST, WLAN_NOTIFICATION_SOURCE_ACM, WLAN_NOTIFICATION_SOURCE_ALL, WLAN_NOTIFICATION_SOURCE_NONE, WLAN_RAW_DATA};
use windows_core::{GUID, PCWSTR};

pub struct WlanAPI {
    handle: HANDLE,
}
impl Drop for WlanAPI {
    fn drop(&mut self) {
        unsafe {
            WlanCloseHandle(self.handle, None);
        }
    }
}
impl WlanAPI {
    pub fn open() -> Result<Self, Error> {
        let mut negotiated_version = 0u32;
        let client_version = 2u32;
        let mut handle = HANDLE::default();
        unsafe {
            let res = WlanOpenHandle(client_version, None, &mut negotiated_version, &mut handle);
            if res != 0 {
                return Error::code(res, "Error opening API");
            }
        }
        Ok(Self { handle })
    }

    pub fn get_interfaces(&self) -> Result<Vec<WlanInterface<'_>>, Error> {
        let mut out = Vec::new();
        let mut list = WLAN_INTERFACE_INFO_LIST::default();
        let mut list: *mut WLAN_INTERFACE_INFO_LIST = &mut list;
        unsafe {
            let res = WlanEnumInterfaces(self.handle, None, &mut list);
            if res != 0 {
                return Error::code(res, "Error enumerating interfaces");
            }
            if list.is_null() {
                return Ok(out);
            }
            let li = *list;
            // let index = li.dwIndex;
            // let numitems = li.dwNumberOfItems;

            let ii = li.InterfaceInfo[0];
            let guid = ii.InterfaceGuid;
            let desc = PCWSTR::from_raw(ii.strInterfaceDescription.as_ptr()).to_string_safer();
            out.push(WlanInterface {
                handle: &self.handle,
                guid,
                description: desc,
            });

            WlanFreeMemory(list as *mut std::ffi::c_void);
        }

        Ok(out)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WlanBSS {
    pub ssid: String,
    pub phy_id: u32,
    pub bssid: [u8; 6],
    pub rssi_dbm: i32,
    pub link_quality_0_100: u32,
    pub in_reg_domain: bool,
    pub beacon_period_us: Duration,
    pub bss_uptime: Duration,
    pub host_timestamp: UTCDateTime,
    pub capability_information: u16,
    pub ch_center_frequency: u32,
    pub rate_set: Vec<f32>,
}
impl Display for WlanBSS {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut uptime = String::new();
        self.bss_uptime.write_iso8601_exact_to(&mut uptime)?;

        f.debug_struct("WlanBSS")
            .field("ssid", &self.ssid)
            .field("phy_id", &self.phy_id)
            .field("bssid", &irox::tools::hex::to_hex_str_upper(&self.bssid))
            .field("rssi_dbm", &self.rssi_dbm)
            .field("link_quality_0_100", &self.link_quality_0_100)
            .field("in_reg_domain", &self.in_reg_domain)
            .field("beacon_period_us", &self.beacon_period_us.to_string())
            .field("bss_uptime", &uptime)
            .field(
                "host_timestamp",
                &self.host_timestamp.format_iso8601_extended(),
            )
            .field("capability_information", &self.capability_information)
            .field("ch_center_frequency", &self.ch_center_frequency)
            .field("rate_set", &self.rate_set)
            .finish()
    }
}
pub struct WlanInterface<'a> {
    handle: &'a HANDLE,
    pub guid: GUID,
    pub description: String,
}
static_init!(scans_in_progress, RwLock<HashMap<GUID, Arc<(Mutex<bool>, Condvar)>>>, {
    RwLock::new(HashMap::new())
});
unsafe extern "system" fn callback(
    param0: *mut L2_NOTIFICATION_DATA,
    _param1: *mut std::ffi::c_void,
) {
    if !param0.is_null() {
        let p0 = *param0;
        if p0.NotificationSource != WLAN_NOTIFICATION_SOURCE_ACM {
            return;
        }
        if p0.NotificationCode != 0x07 {
            return;
        }
        if let Ok(mut lock) = scans_in_progress().write() {
            if let Some(var) = lock.remove(&p0.InterfaceGuid) {
                let dr = var.deref();
                if let Ok(mut lock) = dr.0.lock() {
                    *lock = true;
                }
                dr.1.notify_all();
            }
        }
    }
}
impl<'a> Drop for WlanInterface<'a> {
    fn drop(&mut self) {
        unsafe {
            WlanRegisterNotification(
                self.handle.clone(),
                WLAN_NOTIFICATION_SOURCE_NONE,
                false,
                None,
                None,
                None,
                None,
            );
        }
    }
}
impl<'a> WlanInterface<'a> {
    pub fn reg_callback(&'a self) {
        unsafe {
            let _res = WlanRegisterNotification(
                self.handle.clone(),
                WLAN_NOTIFICATION_SOURCE_ALL,
                false,
                Some(callback),
                None,
                None,
                None,
            );
        }
    }
    pub fn request_scan(&'a self, ssid_in: Option<&str>) -> Result<(), Error> {
        let pair = Arc::new((Mutex::new(false), Condvar::new()));
        unsafe {
            let handle = self.handle.clone();
            let raw = WLAN_RAW_DATA::default();
            self.reg_callback();
            if let Ok(mut lock) = scans_in_progress().write() {
                lock.entry(self.guid).or_insert_with(|| pair.clone());
            }
            let res = {
                if let Some(ssid_in) = ssid_in {
                    let mut ssid = DOT11_SSID::default();
                    write!(ssid.ucSSID.as_mut_slice(), "{ssid_in}")?;
                    WlanScan(handle, &self.guid, Some(&ssid), Some(&raw), None)
                } else {
                    WlanScan(handle, &self.guid, None, Some(&raw), None)
                }
            };
            if res != 0 {
                return Error::code(res, "Error requesting scan");
            }
        }
        let (lock, cvar) = &*pair;
        let mut started = lock.lock().unwrap();
        while !*started {
            started = cvar.wait(started).unwrap();
        }
        // std::thread::sleep(std::time::Duration::from_secs(10));
        Ok(())
    }
    pub fn list_bss(&'a self) -> Result<Vec<WlanBSS>, Error> {
        let mut out = Vec::new();
        let mut list = WLAN_BSS_LIST::default();
        let mut list: *mut WLAN_BSS_LIST = &mut list;
        let handle = self.handle.clone();
        let bss_any = DOT11_BSS_TYPE(3);
        unsafe {
            let res =
                WlanGetNetworkBssList(handle, &self.guid, None, bss_any, false, None, &mut list);
            if res != 0 {
                return Error::code(res, "Error enumerating BSS");
            }

            if list.is_null() {
                return Ok(out);
            }
            let li = *list;
            let numitems = li.dwNumberOfItems as usize;
            let totalsize = li.dwTotalSize as usize;
            let itemsize = std::mem::size_of::<WLAN_BSS_ENTRY>();
            let calcitems = totalsize / itemsize;
            let itemcount = calcitems.min(numitems);

            if itemcount < 1 {
                return Ok(out);
            }
            println!("{numitems} items, total size: {totalsize}, calc_size: {calcitems}, itemsize: {itemsize}");
            std::io::stdout().flush()?;
            // return Ok(out);

            let p = &raw const (*list).wlanBssEntries[0];

            let items = slice::from_raw_parts(p, itemcount);
            for (_idx, bss) in items.iter().enumerate() {
                let host_timestamp =
                    WindowsNTTimestamp::from_seconds_f64(bss.ullHostTimestamp as f64 / 1e7);
                let host_timestamp: UnixTimestamp = UnixTimestamp::from_timestamp(&host_timestamp);
                let _iesize = bss.ulIeSize;
                let _ieoffset = bss.ulIeOffset;

                let val = WlanBSS {
                    ssid: bss.dot11Ssid.to_string_safer(),
                    phy_id: bss.uPhyId,
                    bssid: bss.dot11Bssid,
                    rssi_dbm: bss.lRssi,
                    link_quality_0_100: bss.uLinkQuality,
                    in_reg_domain: bss.bInRegDomain,
                    beacon_period_us: irox::time::Duration::from_micros(bss.usBeaconPeriod as u64),
                    bss_uptime: irox::time::Duration::from_micros(bss.ullTimestamp),
                    host_timestamp: host_timestamp.into(),
                    capability_information: bss.usCapabilityInformation,
                    ch_center_frequency: bss.ulChCenterFrequency,
                    rate_set: Vec::from(
                        bss.wlanRateSet
                            .usRateSet
                            .get(0..(bss.wlanRateSet.uRateSetLength as usize))
                            .unwrap_or_default()
                            .iter()
                            .map(|v| ((*v) & 0x7FFF) as f32 / 0.5)
                            .collect::<Vec<_>>(),
                    ),
                };
                // println!("{idx} :: {val:#?}");
                // std::thread::sleep(Duration::from_secs(1));
                out.push(val);
            }

            WlanFreeMemory(list as *mut std::ffi::c_void);
        }
        Ok(out)
    }
}
impl Debug for WlanInterface<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WlanInterface")
            .field("guid", &self.guid)
            .field("description", &self.description)
            .finish()
    }
}
#[cfg(test)]
mod tests {
    use irox::time::datetime::UTCDateTime;
    use crate::error::Error;
    use crate::wlan::WlanAPI;

    #[test]
    pub fn test() -> Result<(), Error> {
        let api = WlanAPI::open()?;
        let interfaces = api.get_interfaces()?;
        for i in interfaces {
            i.request_scan(None)?;
            let bss_list = i.list_bss()?;
            let now = UTCDateTime::now();
            for bss in bss_list {
                let delta = (now - bss.host_timestamp).as_seconds_f32();
                println!("BSS: {} // {delta}", bss.ssid);
            }
        }
        Ok(())
    }
}
