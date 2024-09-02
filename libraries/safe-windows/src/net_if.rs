use core::slice;
use std::ffi::c_void;
use windows::Win32::Networking::WinSock::{AF_UNSPEC};
use windows::Win32::NetworkManagement::IpHelper::{FreeMibTable, GetInterfaceActiveTimestampCapabilities, GetIpInterfaceTable, INTERFACE_TIMESTAMP_CAPABILITIES, MIB_IPINTERFACE_ROW, MIB_IPINTERFACE_TABLE};
use windows::Win32::NetworkManagement::Ndis::NET_LUID_LH;
use crate::error::Error;

#[derive(Debug)]
pub struct IfInfo {
    pub family: u16,
    pub index: u32,
    pub max_reassembly_size: u32,
    pub interface_identifier: u64,
    pub metric: u32,
    pub min_router_advertisement_interval: u32,
    pub max_router_advertisement_interval: u32,
    pub advertising_enabled: bool,
    pub forwarding_enabled: bool,
    pub weak_host_send: bool,
    pub weak_host_receive: bool,
    pub use_automatic_metric: bool,
    pub connected: bool,
}
impl From<&MIB_IPINTERFACE_ROW> for IfInfo {
    fn from(value: &MIB_IPINTERFACE_ROW) -> Self {
        IfInfo {
            family: value.Family.0,
            index: value.InterfaceIndex,
            max_reassembly_size: value.MaxReassemblySize,
            interface_identifier: value.InterfaceIdentifier,
            metric: value.Metric,

            min_router_advertisement_interval: value.MinRouterAdvertisementInterval,
            max_router_advertisement_interval: value.MaxRouterAdvertisementInterval,
            advertising_enabled: value.AdvertisingEnabled.as_bool(),
            forwarding_enabled: value.ForwardingEnabled.as_bool(),
            weak_host_send: value.WeakHostSend.as_bool(),
            weak_host_receive: value.WeakHostReceive.as_bool(),
            use_automatic_metric: value.UseAutomaticMetric.as_bool(),
            connected: value.Connected.as_bool(),
        }
    }
}
pub struct Interface {
    pub luid: NET_LUID_LH,
    pub if_info: IfInfo
}

impl Interface {
    pub fn iter() -> Result<Vec<Interface>, Error> {
        let mut tb = MIB_IPINTERFACE_TABLE::default();
        let mut tb = std::ptr::from_mut(&mut tb);
        let mut out = Vec::new();
        unsafe {
            let res = GetIpInterfaceTable(AF_UNSPEC, std::ptr::from_mut(&mut tb));
            if res.is_err() {
                return Err(res.into())
            }
            let entries = slice::from_raw_parts(std::ptr::from_ref(&(*tb).Table), (*tb).NumEntries as usize);
            for entry in entries {
                let luid = entry[0].InterfaceLuid;
                let entry: IfInfo = (&entry[0]).into();
                println!("{entry:#?}");
                out.push(Interface {
                    if_info: entry,
                    luid
                })
            }
            println!("Num Entries: {}", entries.len());
        }
        unsafe {
            FreeMibTable(tb as *const c_void);
        }
        Ok(out)
    }
    
    pub fn get_interface_timestamp_capabilities(&self) {
        let mut caps = INTERFACE_TIMESTAMP_CAPABILITIES::default();
        unsafe {
            let iface = std::ptr::from_ref(&self.luid);
            GetInterfaceActiveTimestampCapabilities(
                iface, std::ptr::from_mut(&mut caps)
            );
        }
        println!("{caps:#?}");
    }
}

#[cfg(test)]
mod test {
    use crate::error::Error;
    use crate::net_if::Interface;

    #[test]
    pub fn test() -> Result<(), Error> {
        let out = Interface::iter()?;
        for iface in &out {
            iface.get_interface_timestamp_capabilities();
        }
        Ok(())
    }
}