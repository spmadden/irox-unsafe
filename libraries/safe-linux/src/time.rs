// SPDX-License-Identifier: MIT
// Copyright 2024 IROX Contributors
//

//!
//! Syscalls from `time.c`

use crate::errno::Errno;
use crate::{syscall_1, syscall_2};
pub use irox_enums::{EnumIterItem, EnumName};

pub const SYSCALL_TIMES: u64 = 100;
pub const SYSCALL_CLOCK_GETTIME: u64 = 228;
pub const SYSCALL_CLOCK_GETRES: u64 = 229;

///
/// Equivalent of POSIX's clock_t type.
pub type ClockT = u64;

///
/// Equivalent of POSIX's clock_t type.
#[derive(Debug, Copy, Clone, Eq, PartialEq, EnumName, EnumIterItem)]
#[repr(u32)]
pub enum ClockType {
    ///
    /// The identifier for the system-wide clock measuring real time.  The "Wall-clock" time usually
    /// set via NTP/PTP/RTC/etc.
    Realtime = 0,

    ///
    /// The identifier for the system-wide monotonic clock, which is defined as a clock measuring
    /// real time, whose value cannot be set via `clock_settime()` and which cannot have negative
    /// clock jumps. This clock is set to some random value and increases at a constant rate in a
    /// positive direction and can roll-over around the maximum value.  Used to measure/instrument
    /// precise durational time.
    Monotonic = 1,

    ///
    /// The CPU-Time clock associated with the process making the call
    ProcessCPUTime = 2,

    /// The CPU-Time clock associated with the thread making the call
    ThreadCPUTime = 3,

    MonotonicRaw = 4,

    ///
    /// The realtime clock, but at a much coarser resolution.  My system has this at 10ms
    RealtimeCoarse = 5,

    ///
    /// The monotonic clock, but at a much coarser resolution.  My system has this at 10ms.
    MonotonicCoarse = 6,
    BootTime = 7,
    RealTimeAlarm = 8,
    BootTimeAlarm = 9,
    TAI = 11,
}

///
/// Kernel's version of `Instant`, referenced to a specific clock.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Timespec {
    /// Seconds value
    pub tv_sec: u64,
    /// Nanoseconds value
    pub tv_nsec: u64,
}

///
/// Associates a returned [`Timespec`] with the [`ClockType`] it was requested with.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockTimespec {
    pub timespec: Timespec,
    pub clock: ClockType,
}

///
/// A Block of times provided by the kernel.
#[repr(C)]
#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct Times {
    /// User time
    pub tms_utime: ClockT,
    /// System time
    pub tms_stime: ClockT,
    /// User time of children
    pub tms_cutime: ClockT,
    /// System time of children
    pub tms_cstime: ClockT,
}

///
/// Calls into the Kernel and requests the current value of the specified clock.
pub fn clock_gettime(clock_type: ClockType) -> Result<ClockTimespec, Errno> {
    let mut ts: Timespec = Default::default();

    let ret = unsafe {
        let ptr = core::ptr::from_mut(&mut ts);
        syscall_2!(SYSCALL_CLOCK_GETTIME, clock_type as u64, ptr)
    };

    if ret != 0 {
        return Err(ret.into());
    }
    Ok(ClockTimespec {
        clock: clock_type,
        timespec: ts,
    })
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ClockResolution {
    pub clock: ClockType,
    pub resolution_ns: u64,
}

///
/// Calls into the Kernel and requests the resolution of the specified clock.
pub fn clock_getres(clock_type: ClockType) -> Result<ClockResolution, Errno> {
    let mut ts: Timespec = Default::default();

    let ret = unsafe {
        let ptr = core::ptr::from_mut(&mut ts);
        syscall_2!(SYSCALL_CLOCK_GETRES, clock_type as u64, ptr)
    };

    if ret != 0 {
        return Err(ret.into());
    }
    Ok(ClockResolution {
        clock: clock_type,
        resolution_ns: ts.tv_nsec,
    })
}

#[derive(Default, Debug, Copy, Clone, Eq, PartialEq)]
pub struct TimesTicks {
    pub times: Times,
    /// Number of clock ticks since some arbitrary point in the past.
    pub ticks: u64,
}

/// Returns the current process times & a 'ticks' value at the time of the call.
pub fn times() -> Result<TimesTicks, Errno> {
    let mut ts: Times = Default::default();

    let ret = unsafe {
        let ptr = core::ptr::from_mut(&mut ts);
        syscall_1!(SYSCALL_TIMES, ptr)
    };

    if ret < 0 {
        return Err(ret.into());
    }
    Ok(TimesTicks {
        times: ts,
        ticks: ret as u64,
    })
}

#[cfg(test)]
mod tests {
    use crate::errno::Errno;
    use crate::time::*;

    #[test]
    pub fn test_clock_gettime() -> Result<(), Errno> {
        let out = clock_gettime(ClockType::Realtime)?;

        println!("{out:#?}");

        for clock in ClockType::iter_items() {
            let out = clock_gettime(clock);
            println!("{clock:?}: {out:#?}");
        }

        Ok(())
    }

    #[test]
    pub fn test_clock_getres() -> Result<(), Errno> {
        let out = clock_getres(ClockType::Realtime)?;

        println!("{out:#?}");

        for clock in ClockType::iter_items() {
            let out = clock_getres(clock);
            println!("{clock:?}: {out:#?}");
        }

        Ok(())
    }

    #[test]
    pub fn test_times() -> Result<(), Errno> {
        let out = times()?;
        println!("{out:#?}");
        Ok(())
    }
}
