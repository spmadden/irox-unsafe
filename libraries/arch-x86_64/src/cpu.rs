// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use std::arch::x86_64::{_mm_lfence, _rdtsc};

///
/// Reads the CPU per-core clock counter.  This is monotonic, but likely different per-core.  To get
/// true metrics, lock the thread to a particular core before using this.
pub fn rdtsc() -> u64 {
    unsafe {
        _mm_lfence();
        let out = _rdtsc();
        _mm_lfence();
        out
    }
}
