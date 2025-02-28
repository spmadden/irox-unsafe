use crate::error::Error;
use windows::Win32::System::Threading::{
    GetCurrentProcess, GetPriorityClass, SetPriorityClass, PROCESS_CREATION_FLAGS,
};

const REALTIME: u32 = 0x0100;
const HIGH_PRIORITY: u32 = 0x0080;
const ABOVE_NORMAL: u32 = 0x8000;
const NORMAL: u32 = 0x0020;
const BELOW_NORMAL: u32 = 0x4000;
const IDLE_PRIORITY: u32 = 0x0040;

///
/// Standard windows priority classes.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum PriorityClass {
    Realtime,
    High,
    AboveNormal,
    Normal,
    BelowNormal,
    Idle,
    Other(u32),
}

impl From<u32> for PriorityClass {
    fn from(value: u32) -> Self {
        match value {
            REALTIME => PriorityClass::Realtime,
            HIGH_PRIORITY => PriorityClass::High,
            ABOVE_NORMAL => PriorityClass::AboveNormal,
            NORMAL => PriorityClass::Normal,
            BELOW_NORMAL => PriorityClass::BelowNormal,
            IDLE_PRIORITY => PriorityClass::Idle,
            other => PriorityClass::Other(other),
        }
    }
}
impl From<PriorityClass> for u32 {
    fn from(value: PriorityClass) -> Self {
        match value {
            PriorityClass::Realtime => REALTIME,
            PriorityClass::High => HIGH_PRIORITY,
            PriorityClass::AboveNormal => ABOVE_NORMAL,
            PriorityClass::Normal => NORMAL,
            PriorityClass::BelowNormal => BELOW_NORMAL,
            PriorityClass::Idle => IDLE_PRIORITY,
            PriorityClass::Other(other) => other,
        }
    }
}

///
/// Returns the priority class as returned by [GetPriorityClass](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-getpriorityclass)
pub fn get_current_process_priority() -> PriorityClass {
    unsafe {
        let proc = GetCurrentProcess();
        let clz = GetPriorityClass(proc);
        if clz == 0 {
            return PriorityClass::Other(0);
        }
        clz
    }
    .into()
}

///
/// Sets the priority of the current process using [SetPriorityClass](https://learn.microsoft.com/en-us/windows/win32/api/processthreadsapi/nf-processthreadsapi-setpriorityclass?redirectedfrom=MSDN)
pub fn set_current_process_priority(priority: PriorityClass) -> Result<(), Error> {
    let flags = PROCESS_CREATION_FLAGS(priority.into());
    unsafe {
        let proc = GetCurrentProcess();
        SetPriorityClass(proc, flags)?;
    }
    Ok(())
}
