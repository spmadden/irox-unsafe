// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use core::arch::x86_64::{__cpuid, _mm_lfence, _rdtsc};

///
/// Reads the CPU per-core clock counter.  This is monotonic, but likely different per-core.  To get
/// true metrics, lock the thread to a particular core before using this.  This variant uses memory
/// fences around the instruction to prevent re-ordering.
#[inline]
pub fn rdtsc_fenced() -> u64 {
    unsafe {
        _mm_lfence();
        let out = _rdtsc();
        _mm_lfence();
        out
    }
}

///
/// Reads the CPU per-core clock counter.  This is monotonic, but likely different per-core.  To get
/// true metrics, lock the thread to a particular core before using this.  
#[inline]
pub fn rdtsc() -> u64 {
    unsafe { _rdtsc() }
}

#[inline]
pub fn manufacturer_id() -> [u8; 12] {
    let res = unsafe { __cpuid(0) };
    let [a, b, c, d] = res.ebx.to_le_bytes();
    let [e, f, g, h] = res.edx.to_le_bytes();
    let [i, j, k, l] = res.ecx.to_le_bytes();
    [a, b, c, d, e, f, g, h, i, j, k, l]
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct CpuModelInfo {
    pub model: u8,
    pub family: u8,
    pub cpu_type: u8,
    pub ext_model: u8,
    pub ext_family: u8,
}
impl From<u32> for CpuModelInfo {
    fn from(value: u32) -> Self {
        let mut model = (value >> 4) & 0xF;
        let family = (value >> 8) & 0xF;
        let cpu_type = (value >> 12) & 0x3;
        let ext_model = (value >> 16) & 0xF;
        let ext_family = (value >> 20) & 0xFF;
        if family == 6 || family == 15 {
            model += ext_model << 4;
        }
        Self {
            model: model as u8,
            family: family as u8,
            cpu_type: cpu_type as u8,
            ext_model: ext_model as u8,
            ext_family: ext_family as u8,
        }
    }
}
pub fn cpu_model() -> CpuModelInfo {
    let res = unsafe { __cpuid(1) };
    res.eax.into()
}
#[repr(u64)]
#[derive(
    Debug,
    Copy,
    Clone,
    Eq,
    PartialEq,
    Hash,
    irox_enums_derive::EnumName,
    irox_enums_derive::EnumIterItem,
)]
pub enum CpuFeature {
    /// Streaming SIMD Extensions 3
    SSE3 = 1 << 0,
    /// PCLMULDQ Instruction
    PCLMULDQ = 1 << 1,
    /// 64-Bit DS Area
    DTES64 = 1 << 2,
    /// Monitor/MWAIT
    MONITOR = 1 << 3,
    /// CPL Qualified Debug Store
    DSCPL = 1 << 4,
    /// Virtual Machine Extensions
    VMX = 1 << 5,
    /// Safer Mode Extensions
    SMX = 1 << 6,
    /// Enhanced Intel Speedstep
    EIST = 1 << 7,
    /// Thermal Monitor 2
    TM2 = 1 << 8,
    /// Supplemental Streaming SIMD Extension 3
    SSSE3 = 1 << 9,
    /// L1 Context ID
    CNXTID = 1 << 10,
    /// IA32_DEBUG_INTERFACE MSR
    SDBG = 1 << 11,
    /// FMA using YMM
    FMA = 1 << 12,
    /// CMPXCHG16B Available
    CMPXCHG16B = 1 << 13,
    /// xTPR Update Control
    XTPR = 1 << 14,
    /// Perfmon and Debug Capability
    PDCM = 1 << 15,
    /// Process-context Identifiers
    PCID = 1 << 17,
    /// Memory Mapped Device prefetch
    DCA = 1 << 18,
    /// SSE4.1
    SSE4_1 = 1 << 19,
    /// SSE4.2
    SSE4_2 = 1 << 20,
    X2APIC = 1 << 21,
    MOVBE = 1 << 22,
    POPCNT = 1 << 23,
    /// Local one-shot operation using a TSC Deadline value
    TSCDL = 1 << 24,
    /// AES-NI
    AES = 1 << 25,
    /// XSAVE/XRSTOR
    XSAVE = 1 << 26,
    OSXSAVE = 1 << 27,
    AVX = 1 << 28,
    /// 16-bit Floating point conversions
    F16C = 1 << 29,
    /// RDRAND
    RDRAND = 1 << 30,

    FPUX87 = 1 << 32,
    VME = 1 << 33,
    DE = 1 << 34,
    PSE = 1 << 35,
    TSC = 1 << 36,
    MSR = 1 << 37,
    PAE = 1 << 38,
    MCE = 1 << 39,
    CX8 = 1 << 40,
    APIC = 1 << 41,
    SEP = 1 << 43,
    MTRR = 1 << 44,
    PGE = 1 << 45,
    MCA = 1 << 46,
    CMOV = 1 << 47,
    PAT = 1 << 48,
    PSE36 = 1 << 49,
    PSM = 1 << 50,
    CLFSH = 1 << 51,
    DS = 1 << 53,
    ACPI = 1 << 54,
    MMX = 1 << 55,
    FXSR = 1 << 56,
    SSE = 1 << 57,
    SSE2 = 1 << 58,
    SS = 1 << 59,
    HTT = 1 << 60,
    TM = 1 << 61,
    PBE = 1 << 63,
}
pub struct CpuFeatures {
    inner: u64,
}
impl CpuFeatures {
    pub fn has_feature(&self, f: CpuFeature) -> bool {
        let v = f as u64;
        self.inner & v == v
    }
    #[cfg(feature = "std")]
    pub fn print_features(&self) {
        let mut idx = 0;
        for f in <CpuFeature as irox_enums::EnumIterItem>::iter_items() {
            if self.has_feature(f) {
                idx += 1;
                idx += f.name().len();
                if idx >= 60 {
                    idx -= 60;
                    println!();
                }
                print!("{} ", f.name());
            }
        }
        println!()
    }
}
pub fn cpu_features() -> CpuFeatures {
    let res = unsafe { __cpuid(1) };
    let _brandidx = res.ebx & 0xFF;
    let fig37tbl310 = res.ecx;
    let fig38tbl311 = res.edx;
    CpuFeatures {
        inner: fig37tbl310 as u64 | ((fig38tbl311 as u64) << 32),
    }
}
#[derive(Debug)]
pub struct TimeStampCounter {
    pub core_crystal_freq_hz: u32,
    pub tsc_ratio_upper: u32,
    pub tsc_ratio_lower: u32,
}
impl TimeStampCounter {
    pub fn get() -> Self {
        let res = unsafe { __cpuid(1) };
        TimeStampCounter {
            core_crystal_freq_hz: res.ecx,
            tsc_ratio_lower: res.eax,
            tsc_ratio_upper: res.ebx,
        }
    }
    pub fn get_tsc_frequency(&self) -> u64 {
        0
    }
}
#[cfg(all(test, feature = "std"))]
mod test {
    use crate::cpu::{cpu_features, cpu_model, manufacturer_id, TimeStampCounter};
    use std::arch::x86_64::__cpuid;

    #[test]
    pub fn get_manufacturer_id() {
        let id = manufacturer_id();
        let v = core::str::from_utf8(&id).unwrap_or_default();
        println!("manufacturer id: {v}");
    }
    #[test]
    pub fn get_cpu_model() {
        let m = cpu_model();
        println!("{m:#?}");
        let res = unsafe { __cpuid(1) };
        println!("{:08X}", res.eax);
    }

    #[test]
    pub fn features() {
        cpu_features().print_features();
    }

    #[test]
    pub fn tsc() {
        let tsc = TimeStampCounter::get();
        println!("{tsc:?}");
    }
}
