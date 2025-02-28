use crate::error::Error;
use core::ops::Add;
use irox::time::datetime::UTCDateTime;
use irox::time::epoch::WindowsNTTimestamp;
use irox::time::Duration;
use irox::units::units::datasize::{DataSize, DataSizeUnits};
use windows::Win32::Foundation::FILETIME;
use windows::Win32::System::Memory::GetLargePageMinimum;
use windows::Win32::System::ProcessStatus::{GetPerformanceInfo, PERFORMANCE_INFORMATION};
use windows::Win32::System::SystemInformation::{
    GetSystemInfo, GetSystemTime, GetTickCount64, GlobalMemoryStatusEx, MEMORYSTATUSEX, SYSTEM_INFO,
};
use windows::Win32::System::Threading::{GetCurrentProcess, GetProcessTimes};

#[derive(Debug, Default, Copy, Clone)]
pub struct SystemInformation {
    /// Size of a memory page in bytes, usually 4096
    pub page_size: u32,
    /// Mask of active processor cores, 1 bit = 1 core
    pub active_processor_mask: usize,
    /// Total number of processors
    pub number_of_processors: u32,
    pub processor_type: u32,
    pub allocation_granularity: u32,
    pub processor_level: u16,
    pub processor_revision: u16,
    pub processor_architecture: u16,
}
impl From<SYSTEM_INFO> for SystemInformation {
    fn from(info: SYSTEM_INFO) -> Self {
        SystemInformation {
            page_size: info.dwPageSize,
            active_processor_mask: info.dwActiveProcessorMask,
            number_of_processors: info.dwNumberOfProcessors,
            processor_type: info.dwProcessorType,
            allocation_granularity: info.dwAllocationGranularity,
            processor_level: info.wProcessorLevel,
            processor_revision: info.wProcessorRevision,
            processor_architecture: unsafe { info.Anonymous.Anonymous.wProcessorArchitecture.0 },
        }
    }
}

/// Returns some basic system information, page size, number of processors and types
#[must_use]
pub fn get_system_info() -> SystemInformation {
    let mut sys_info = SYSTEM_INFO::default();
    unsafe { GetSystemInfo(&mut sys_info) };
    sys_info.into()
}

/// Returns the local system time in UTC, resolution is milliseconds
#[must_use]
pub fn get_system_time_utc() -> UTCDateTime {
    let sys_time = unsafe { GetSystemTime() };
    UTCDateTime::try_from_values(
        sys_time.wYear as i32,
        sys_time.wMonth as u8,
        sys_time.wDay as u8,
        sys_time.wHour as u8,
        sys_time.wMinute as u8,
        sys_time.wSecond as u8,
    )
    .unwrap_or_default()
    .add(Duration::from_millis(sys_time.wMilliseconds as u64))
}

/// Returns the system uptime, effectively a monotonic clock, resolution is milliseconds
#[must_use]
pub fn get_system_uptime() -> Duration {
    let tick_count = unsafe { GetTickCount64() };
    Duration::from_millis(tick_count)
}

/// Basic system memory info
#[derive(Debug, Default, Copy, Clone)]
pub struct MemoryInfo {
    /// percentage of physical memory used
    pub memory_load_0_100: u8,
    /// Total physical memory
    pub total_physical_memory: DataSize,
    /// Available physical memory
    pub available_physical_memory: DataSize,
    /// Total size of the system page file(s)
    pub total_page_file: DataSize,
    /// Amount of page file available
    pub available_page_file: DataSize,
    /// Total virtual memory capable of being used
    pub total_virtual_memory: DataSize,
    /// Available virtual memory
    pub available_virtual_memory: DataSize,
    /// Available extended virtual memory
    pub available_extended_virtual_memory: DataSize,
}
impl From<MEMORYSTATUSEX> for MemoryInfo {
    fn from(info: MEMORYSTATUSEX) -> Self {
        MemoryInfo {
            memory_load_0_100: info.dwMemoryLoad as u8,
            total_physical_memory: DataSize::new(info.ullTotalPhys as f64, DataSizeUnits::Bytes),
            available_physical_memory: DataSize::new(
                info.ullAvailPhys as f64,
                DataSizeUnits::Bytes,
            ),
            total_page_file: DataSize::new(info.ullTotalPageFile as f64, DataSizeUnits::Bytes),
            available_page_file: DataSize::new(info.ullAvailPageFile as f64, DataSizeUnits::Bytes),
            total_virtual_memory: DataSize::new(info.ullTotalVirtual as f64, DataSizeUnits::Bytes),
            available_virtual_memory: DataSize::new(
                info.ullAvailVirtual as f64,
                DataSizeUnits::Bytes,
            ),
            available_extended_virtual_memory: DataSize::new(
                info.ullAvailExtendedVirtual as f64,
                DataSizeUnits::Bytes,
            ),
        }
    }
}

/// Returns some basic system memory info, total and available physical and virtual memory
pub fn get_system_memory_info() -> Result<MemoryInfo, Error> {
    let mut mem = MEMORYSTATUSEX::default();
    mem.dwLength = std::mem::size_of::<MEMORYSTATUSEX>() as u32;
    unsafe {
        GlobalMemoryStatusEx(&mut mem)?;
    }
    Ok(mem.into())
}
#[derive(Debug, Default, Copy, Clone)]
pub struct PerformanceInfo {
    pub commit_total: DataSize,
    pub commit_limit: DataSize,
    pub commit_peak: DataSize,
    pub physical_total: DataSize,
    pub physical_available: DataSize,
    pub system_cache: DataSize,
    pub kernel_total: DataSize,
    pub kernel_paged: DataSize,
    pub kernel_nonpaged: DataSize,
    pub page_size: DataSize,
    pub handle_count: u32,
    pub process_count: u32,
    pub thread_count: u32,
}
impl From<PERFORMANCE_INFORMATION> for PerformanceInfo {
    fn from(info: PERFORMANCE_INFORMATION) -> Self {
        let page_size = info.PageSize as f64;
        PerformanceInfo {
            commit_total: DataSize::new(info.CommitTotal as f64 * page_size, DataSizeUnits::Bytes),
            commit_limit: DataSize::new(info.CommitLimit as f64 * page_size, DataSizeUnits::Bytes),
            commit_peak: DataSize::new(info.CommitPeak as f64 * page_size, DataSizeUnits::Bytes),
            physical_total: DataSize::new(
                info.PhysicalTotal as f64 * page_size,
                DataSizeUnits::Bytes,
            ),
            physical_available: DataSize::new(
                info.PhysicalAvailable as f64 * page_size,
                DataSizeUnits::Bytes,
            ),
            system_cache: DataSize::new(info.SystemCache as f64 * page_size, DataSizeUnits::Bytes),
            kernel_total: DataSize::new(info.KernelTotal as f64 * page_size, DataSizeUnits::Bytes),
            kernel_paged: DataSize::new(info.KernelPaged as f64 * page_size, DataSizeUnits::Bytes),
            kernel_nonpaged: DataSize::new(
                info.KernelNonpaged as f64 * page_size,
                DataSizeUnits::Bytes,
            ),
            page_size: DataSize::new(info.PageSize as f64, DataSizeUnits::Bytes),
            handle_count: info.HandleCount,
            process_count: info.ProcessCount,
            thread_count: info.ThreadCount,
        }
    }
}
pub fn get_performance_info() -> Result<PerformanceInfo, Error> {
    let mut perf_info = PERFORMANCE_INFORMATION::default();
    let size = std::mem::size_of::<PERFORMANCE_INFORMATION>() as u32;
    perf_info.cb = size;
    unsafe {
        GetPerformanceInfo(&mut perf_info, size)?;
    }
    Ok(perf_info.into())
}

pub fn get_large_page_minimum_bytes() -> Result<usize, Error> {
    let size = unsafe { GetLargePageMinimum() };
    if size == 0 {
        return Error::notfound();
    }
    Ok(size)
}

#[derive(Debug, Copy, Clone)]
pub struct ProcessTimes {
    pub creation_time: WindowsNTTimestamp,
    pub exit_time: WindowsNTTimestamp,
    pub kernel_time: Duration,
    pub user_time: Duration,
}

pub fn get_process_times() -> Result<ProcessTimes, Error> {
    let mut creation_time = FILETIME::default();
    let mut exit_time = FILETIME::default();
    let mut kernel_time = FILETIME::default();
    let mut user_time = FILETIME::default();
    unsafe {
        let hnd = GetCurrentProcess();
        GetProcessTimes(
            hnd,
            &mut creation_time,
            &mut exit_time,
            &mut kernel_time,
            &mut user_time,
        )?;
    }

    Ok(ProcessTimes {
        creation_time: creation_time.to_nt_timestamp(),
        exit_time: exit_time.to_nt_timestamp(),
        kernel_time: kernel_time.to_duration(),
        user_time: user_time.to_duration(),
    })
}

pub trait FTimeConversions {
    fn to_nt_timestamp(&self) -> WindowsNTTimestamp;
    fn to_duration(&self) -> Duration;
}
impl FTimeConversions for FILETIME {
    fn to_nt_timestamp(&self) -> WindowsNTTimestamp {
        let hns: u64 = (self.dwHighDateTime as u64) << 32 | self.dwLowDateTime as u64;
        let sec: f64 = hns as f64 / 1e7;
        WindowsNTTimestamp::from_seconds_f64(sec)
    }

    fn to_duration(&self) -> Duration {
        self.to_nt_timestamp().get_offset()
    }
}

#[cfg(test)]
mod test {
    use crate::system::{
        get_large_page_minimum_bytes, get_performance_info, get_process_times, get_system_info,
        get_system_memory_info, get_system_time_utc, get_system_uptime,
    };
    use irox::time::format::iso8601::ISO8601_DATE_TIME;

    #[test]
    pub fn test_get_system_info() {
        println!("{:#?}", get_system_info());
    }

    #[test]
    pub fn test_get_system_time() {
        println!("{:#?}", get_system_time_utc());
        println!("{}", get_system_time_utc().format(&ISO8601_DATE_TIME));
    }

    #[test]
    pub fn test_get_system_uptime() {
        let (d, h, m, s) = get_system_uptime().as_dhms();
        println!("{} days {} hours {} minutes {} seconds", d, h, m, s);
    }

    #[test]
    pub fn test_get_system_memory_info() {
        let info = get_system_memory_info().unwrap();
        println!("{:#?}", info);
        println!("{}", info.total_physical_memory);
        println!(
            "{:2.3} GiB",
            info.total_physical_memory.value() / 1024. / 1024. / 1024.
        );
    }

    #[test]
    pub fn test_get_performance_info() {
        let info = get_performance_info().unwrap();
        println!("{:#?}", info);
    }

    #[test]
    pub fn test_get_large_page_minimum() {
        let lpm = get_large_page_minimum_bytes().unwrap();
        println!("{}", lpm);
        println!("{} k", lpm as f64 / 1024.);
    }

    #[test]
    pub fn test_get_process_times() {
        println!("{:#?}", get_process_times().unwrap());
    }
}
