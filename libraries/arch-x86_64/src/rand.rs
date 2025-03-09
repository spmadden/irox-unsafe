// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use core::arch::x86_64::{_rdrand32_step, _rdrand64_step};
use std::arch::x86_64::{_rdseed32_step, _rdseed64_step};

///
/// Wrapper around the `rdrand64` instruction
pub fn rdrand64() -> Option<u64> {
    let mut out: u64 = 0;
    (unsafe { _rdrand64_step(&mut out) } == 1).then_some(out)
}

///
/// Wrapper around the `rdrand32` instruction
pub fn rdrand32() -> Option<u32> {
    let mut out = 0u32;
    (unsafe { _rdrand32_step(&mut out) } == 1).then_some(out)
}
///
/// Wrapper around the `rdseed` instruction
pub fn rdseed32() -> Option<u32> {
    let mut out: u32 = 0;
    (unsafe { _rdseed32_step(&mut out) } == 1).then_some(out)
}
///
/// Wrapper around the `rdseed64` instruction
pub fn rdseed64() -> Option<u64> {
    let mut out: u64 = 0;
    (unsafe { _rdseed64_step(&mut out) } == 1).then_some(out)
}
///
/// Wrapper around the `rdseed64` instruction
pub fn seed() -> Option<u64> {
    rdseed64()
}
///
/// Wrapper around the `rdrand64` instruction
pub fn rand() -> Option<u64> {
    rdrand64()
}

///
/// Generates two 64's and concats them into a u128.
pub fn rand128() -> Option<u128> {
    let a = rdrand64()?;
    let b = rdrand64()?;
    Some(((a as u128) << 64) | b as u128)
}

#[cfg(test)]
mod test {
    use crate::rand::rand;

    #[test]
    pub fn test() {
        let rnd = rand();
        assert_ne!(None, rnd);
    }
}
