// SPDX-License-Identifier: MIT
// Copyright 2024 IROX Contributors
//

//!
//! System Calls from `sys.c`

use crate::errno::Errno;
use crate::syscall_1;

pub const SYSCALL_SYSINFO: u64 = 99;

/// System-info shuttle structure.
#[repr(C)]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct SysInfo {
    /// Seconds since boot.
    pub uptime: i64,
    /// 1m, 5m, and 15m load averages
    pub loads: [u64; 3],
    /// Total usable main memory size in 'mem_unit's
    pub total_ram: u64,
    /// Available memory size in 'mem_unit's
    pub free_ram: u64,
    /// Amount of shared memory in 'mem_unit's
    pub shared_ram: u64,
    /// Memory used by buffers in 'mem_unit's
    pub buffer_ram: u64,
    /// Total swap space size
    pub total_swap: u64,
    /// Swap space still available
    pub free_swap: u64,
    /// Number of current processes
    pub num_procs: u16,
    pub _pad: u16,
    /// Total high memory size
    pub total_high_mem: u64,
    /// Available high memory size
    pub free_high_mem: u64,
    /// Memory unit size in bytes
    pub mem_unit: u32,
}

///
/// Linux `sysinfo` Syscall, returns most of the values in the 'top' command.
pub fn sysinfo() -> Result<SysInfo, Errno> {
    let mut sysinfo = SysInfo::default();
    let ret = unsafe {
        let ptr = core::ptr::from_mut(&mut sysinfo);
        syscall_1!(SYSCALL_SYSINFO, ptr)
    };

    if ret < 0 {
        return Err(ret.into());
    }

    Ok(sysinfo)
}

#[cfg(test)]
mod tests {
    use crate::errno::Errno;
    use crate::sys::sysinfo;

    #[test]
    pub fn test_sysinfo() -> Result<(), Errno> {
        let res = sysinfo()?;
        println!("{res:#?}");
        Ok(())
    }
}
