// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use std::arch::asm;
use std::arch::x86_64::_rdrand64_step;

pub fn rdrand64() -> u64 {
    let mut num: u64;
    let mut _carry: u8 = 0;
    unsafe {
        asm!(
        "rdrand {num}",
        "setc {carry}",
        num = out(reg) num,
        carry = out(reg_byte) _carry,
        );
    }
    assert_ne!(0, _carry);
    num
}

pub fn rdrand32() -> u32 {
    let mut num: u32;
    let mut _carry: u8 = 0;
    unsafe {
        asm!(
        "rdrand {num:e}",
        "setc {carry}",
        num = out(reg) num,
        carry = out(reg_byte) _carry,
        );
    }
    assert_ne!(0, _carry);
    num
}

pub fn rand() -> Option<u64> {
    let mut out: u64 = 0;
    (unsafe { _rdrand64_step(&mut out) } == 1).then_some(out)
}

pub fn rand128() -> u128 {
    let a = rdrand64();
    let b = rdrand64();
    ((a as u128) << 64) | b as u128
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