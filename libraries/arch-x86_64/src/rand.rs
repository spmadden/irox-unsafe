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
/// incremental multiplier for each state
const MULTIPLIER: i64 = 6364136223846793005i64;

/// incremental incrementer for each state
const INCREMENT: i64 = 1442695040888963407i64;
// const MUL_128: u128 = 0x2360ED051FC65DA44385DF649FCCF645u128;
pub const fn _single_round_pcg_xsh_rr(state: u64) -> (u64, u32) {
    let count = (state >> 59) as u32;
    let newstate = state
        .wrapping_mul(MULTIPLIER as u64)
        .wrapping_mul(INCREMENT as u64);
    let state = state | (state >> 18);
    let state = (state >> 27) as u32;
    let out = state.rotate_right(count);
    (newstate, out)
}

///
/// `PCG-XSL-RR-RR`, 128-bit state, 128-bit output.  Fastest PRNG in the west.  Most insecure of them all.
pub struct PcgXslRrRr {
    state: u128,
}
const MULTIPLIER_128: u128 = 0x2360ED051FC65DA44385DF649FCCF645u128;
const INCREMENT_128: u128 = 0x5851F42D4C957F2D14057B7EF767814Fu128;
impl PcgXslRrRr {
    ///
    /// Creates a random seeded with this number.
    pub fn new_seed(seed: u128) -> Self {
        Self {
            state: seed.wrapping_mul(2).wrapping_add(1),
        }
    }
    pub fn next_u128(&mut self) -> u128 {
        let state = self.state;
        self.state = state
            .wrapping_mul(MULTIPLIER_128)
            .wrapping_add(INCREMENT_128);
        let rot1 = (state >> 122) as u32;
        let high = (state >> 64) as u64;
        let newlow = (high ^ state as u64).rotate_right(rot1);
        let newhigh = high.rotate_right((newlow & 0x3F) as u32);
        (newhigh as u128) << 64 | newlow as u128
    }
    pub fn next_u128_asm(&mut self) -> u128 {
        let state = self.state;
        self.state = state
            .wrapping_mul(MULTIPLIER_128)
            .wrapping_add(INCREMENT_128);
        unsafe {
            let [high, low] = std::mem::transmute::<u128, [u64; 2]>(state);
            let rot1: u32 = (high >> 58) as u32;
            let newlow = (high ^ low).rotate_right(rot1);
            let newhigh = high.rotate_right((newlow & 0x3F) as u32);
            std::mem::transmute::<[u64; 2], u128>([newhigh, newlow])
        }
    }
}

#[cfg(test)]
mod test {
    use crate::rand::{rand, PcgXslRrRr};

    #[test]
    pub fn test() {
        let rnd = rand();
        assert_ne!(None, rnd);
    }

    #[test]
    pub fn test2() {
        let mut a = PcgXslRrRr::new_seed(0);
        // a.update_state();
    }
}
