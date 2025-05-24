// SPDX-License-Identifier: MIT
// Copyright 2025 IROX Contributors
//

#![allow(clippy::integer_division_remainder_used)]

use core::ops::BitXorAssign;
use irox_bits::{MutBits, WriteToLEBits};
use irox_tools::buf::Buffer;
use irox_tools::buf::FixedU8Buf;
use irox_tools::hash::HashDigest;
use std::arch::x86_64::{
    __m128i, _mm_add_epi32, _mm_alignr_epi8, _mm_blend_epi16, _mm_castps_si128, _mm_castsi128_ps,
    _mm_loadu_si128, _mm_set_epi8, _mm_setzero_si128, _mm_shuffle_epi32, _mm_shuffle_epi8,
    _mm_shuffle_ps, _mm_shufflehi_epi16, _mm_slli_epi32, _mm_slli_si128, _mm_srli_epi32,
    _mm_srli_si128, _mm_storeu_si128, _mm_unpackhi_epi32, _mm_unpackhi_epi64, _mm_unpacklo_epi32,
    _mm_unpacklo_epi64, _mm_xor_si128,
};

macro_rules! tof {
    ($reg:expr) => {
        _mm_castsi128_ps($reg)
    };
}
macro_rules! toi {
    ($reg:expr) => {
        _mm_castps_si128($reg)
    };
}
macro_rules! g1 {
    ($a:expr, $b:expr, $c:expr, $d:expr, $x:expr, $r1:literal, $r2:literal) => {
        *$a = _mm_add_epi32(*$b, _mm_add_epi32(*$a, *$x));
        *$d = _mm_xor_si128(*$d, *$a);
        *$d = _mm_roti_epi32!(*$d, $r1);
        *$c = _mm_add_epi32(*$d, *$c);
        *$b = _mm_xor_si128(*$b, *$c);
        *$b = _mm_roti_epi32!(*$b, $r2);
    };
}
macro_rules! diag {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        *$a = _mm_shuffle_epi32(*$a, _mm_shuffle(2, 1, 0, 3));
        *$d = _mm_shuffle_epi32(*$d, _mm_shuffle(1, 0, 3, 2));
        *$c = _mm_shuffle_epi32(*$c, _mm_shuffle(0, 3, 2, 1));
    };
}
macro_rules! undiag {
    ($a:expr, $b:expr, $c:expr, $d:expr) => {
        *$a = _mm_shuffle_epi32(*$a, _mm_shuffle(0, 3, 2, 1));
        *$d = _mm_shuffle_epi32(*$d, _mm_shuffle(1, 0, 3, 2));
        *$c = _mm_shuffle_epi32(*$c, _mm_shuffle(2, 1, 0, 3));
    };
}
macro_rules! round {
    ($a:expr, $b:expr, $c:expr, $d:expr, $buf1:expr, $x:expr, $y:expr, $z:expr,$n:expr) => {{
        *$buf1 = { $x };
        g1!($a, $b, $c, $d, $buf1, 16, 12);
        *$buf1 = { $y };
        g1!($a, $b, $c, $d, $buf1, 8, 7);
        diag!($a, $b, $c, $d);
        *$buf1 = { $z };
        g1!($a, $b, $c, $d, $buf1, 16, 12);
        *$buf1 = { $n };
        g1!($a, $b, $c, $d, $buf1, 8, 7);
        undiag!($a, $b, $c, $d);
    }};
}
#[inline]
pub(crate) const fn _mm_shuffle(a: u8, b: u8, c: u8, d: u8) -> i32 {
    ((a as i32) << 6) | ((b as i32) << 4) | ((c as i32) << 2) | (d as i32)
}
macro_rules! r8 {
    () => {
        _mm_set_epi8(12, 15, 14, 13, 8, 11, 10, 9, 4, 7, 6, 5, 0, 3, 2, 1)
    };
}
macro_rules! r16 {
    () => {
        _mm_set_epi8(13, 12, 15, 14, 9, 8, 11, 10, 5, 4, 7, 6, 1, 0, 3, 2)
    };
}
macro_rules! _mm_roti_epi32 {
    ($r: expr, $c: expr) => {{
        if $c == 8 {
            _mm_shuffle_epi8($r, r8!())
        } else if $c == 16 {
            _mm_shuffle_epi8($r, r16!())
        } else {
            _mm_xor_si128(
                _mm_srli_epi32::<{ $c }>($r),
                _mm_slli_epi32::<{ 32 - $c }>($r),
            )
        }
    }};
}

static BLAKE2S_IV: &[u32; 8] = &[
    0x6A09E667, 0xBB67AE85, 0x3C6EF372, 0xA54FF53A, 0x510E527F, 0x9B05688C, 0x1F83D9AB, 0x5BE0CD19,
];
#[repr(C)]
pub struct BLAKE2s<const NN: usize> {
    h: [u32; 8],
    written: u64,
    flags: [u32; 2],
    buf: FixedU8Buf<64>,
}
impl<const NN: usize> Default for BLAKE2s<NN> {
    fn default() -> Self {
        Self::new(&[])
    }
}
impl<const NN: usize> BLAKE2s<NN> {
    pub fn new(key: &[u8]) -> Self {
        let mut out = Self {
            h: *BLAKE2S_IV,
            written: 0,
            flags: [0, 0],
            buf: Default::default(),
        };
        let nn = (NN as u32) & 0xFF;
        let kk = ((key.len() as u32) & 0xFF) << 8;
        let p = (0x01010000u32) | nn | kk;
        out.h[0].bitxor_assign(p);
        if kk > 0 {
            let _ = out.buf.write_all_bytes(key);
            let _ = out.buf.update_length(64);
            out.written += 64;
        }

        out
    }

    unsafe fn chomp(&mut self, last: bool) {
        let m = self.buf.as_buf_default();
        let mp: *const u32 = core::mem::transmute(m.as_ptr());
        self.chomp_exact(mp, last);
    }
    unsafe fn chomp_exact(&mut self, mp: *const u32, last: bool) {
        // if self.buf.is_empty() && self.written > 0 && !last {
        //     return;
        // }
        let m0 = _mm_loadu_si128(mp as *const _);
        let m1 = _mm_loadu_si128(mp.offset(4) as *const _);
        let m2 = _mm_loadu_si128(mp.offset(8) as *const _);
        let m3 = _mm_loadu_si128(mp.offset(12) as *const _);

        let hp = self.h.as_ptr();
        let mut row1 = _mm_loadu_si128(hp as *const _);
        let mut row2 = _mm_loadu_si128(hp.offset(4) as *const _);
        let ff0 = row1;
        let ff1 = row2;

        let ivp = BLAKE2S_IV.as_ptr();
        let mut row3 = _mm_loadu_si128(ivp as *const _);
        if last {
            self.flags[0] = u32::MAX;
        }
        let a = _mm_loadu_si128(ivp.offset(4) as *const _);
        let b = _mm_loadu_si128(&raw const self.written as *const _);
        let mut row4 = _mm_xor_si128(a, b);
        let mut buf1: __m128i = _mm_setzero_si128();
        let mut t0: __m128i;
        let mut t1: __m128i;
        let mut t2: __m128i;

        // round 0
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            toi!(_mm_shuffle_ps::<{ _mm_shuffle(2, 0, 2, 0) }>(
                tof!(m0),
                tof!(m1)
            )),
            toi!(_mm_shuffle_ps::<{ _mm_shuffle(3, 1, 3, 1) }>(
                tof!(m0),
                tof!(m1)
            )),
            {
                t0 = _mm_shuffle_epi32::<{ _mm_shuffle(3, 2, 0, 1) }>(m2);
                t1 = _mm_shuffle_epi32::<{ _mm_shuffle(0, 1, 3, 2) }>(m3);
                _mm_blend_epi16::<0xC3>(t0, t1)
            },
            {
                t0 = _mm_blend_epi16::<0x3C>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 3, 0, 1) }>(t0)
            }
        );
        // round 1
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_blend_epi16::<0x0C>(m1, m2);
                t1 = _mm_slli_si128::<4>(m3);
                t2 = _mm_blend_epi16::<0xF0>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 1, 0, 3) }>(t2)
            },
            {
                t0 = _mm_shuffle_epi32::<{ _mm_shuffle(0, 0, 2, 0) }>(m2);
                t1 = _mm_blend_epi16::<0xC0>(m1, m3);
                t2 = _mm_blend_epi16::<0xF0>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 3, 0, 1) }>(t2)
            },
            {
                t0 = _mm_slli_si128::<4>(m1);
                t1 = _mm_blend_epi16::<0x30>(m2, t0);
                t2 = _mm_blend_epi16::<0xF0>(m0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(3, 0, 1, 2) }>(t2)
            },
            {
                t0 = _mm_unpackhi_epi32(m0, m1);
                t1 = _mm_slli_si128::<4>(m3);
                t2 = _mm_blend_epi16::<0x0C>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(3, 0, 1, 2) }>(t2)
            }
        );
        // round 2
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_unpackhi_epi32(m2, m3);
                t1 = _mm_blend_epi16::<0x0C>(m3, m1);
                t2 = _mm_blend_epi16::<0x0F>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(3, 1, 0, 2) }>(t2)
            },
            {
                t0 = _mm_unpacklo_epi32(m2, m0);
                t1 = _mm_blend_epi16::<0xF0>(t0, m0);
                t2 = _mm_slli_si128::<8>(m3);
                _mm_blend_epi16::<0xC0>(t1, t2)
            },
            {
                t0 = _mm_blend_epi16::<0x3C>(m0, m2);
                t1 = _mm_srli_si128::<12>(m1);
                t2 = _mm_blend_epi16::<0x03>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(0, 3, 2, 1) }>(t2)
            },
            {
                t0 = _mm_slli_si128::<4>(m3);
                t1 = _mm_blend_epi16::<0x33>(m0, m1);
                t2 = _mm_blend_epi16::<0xC0>(t1, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(1, 2, 3, 0) }>(t2)
            }
        );
        // round 3
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_unpackhi_epi32(m0, m1);
                t1 = _mm_unpackhi_epi32(t0, m2);
                t2 = _mm_blend_epi16::<0x0C>(t1, m3);
                _mm_shuffle_epi32::<{ _mm_shuffle(3, 1, 0, 2) }>(t2)
            },
            {
                t0 = _mm_slli_si128::<8>(m2);
                t1 = _mm_blend_epi16::<0x0C>(m3, m0);
                t2 = _mm_blend_epi16::<0xC0>(t1, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 0, 1, 3) }>(t2)
            },
            {
                t0 = _mm_blend_epi16::<0x0F>(m0, m1);
                t1 = _mm_blend_epi16::<0xC0>(t0, m3);
                _mm_shuffle_epi32::<{ _mm_shuffle(0, 1, 2, 3) }>(t1)
            },
            {
                t0 = _mm_alignr_epi8::<4>(m0, m1);
                _mm_blend_epi16::<0x33>(t0, m2)
            }
        );
        // round 4
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_unpacklo_epi64(m1, m2);
                t1 = _mm_unpackhi_epi64(m0, m2);
                t2 = _mm_blend_epi16::<0x33>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 0, 1, 3) }>(t2)
            },
            {
                t0 = _mm_unpackhi_epi64(m1, m3);
                t1 = _mm_unpacklo_epi64(m0, m1);
                _mm_blend_epi16::<0x33>(t0, t1)
            },
            {
                t0 = _mm_unpackhi_epi64(m3, m1);
                t1 = _mm_unpackhi_epi64(m2, m0);
                t2 = _mm_blend_epi16::<0x33>(t1, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 1, 0, 3) }>(t2)
            },
            {
                t0 = _mm_blend_epi16::<0x03>(m0, m2);
                t1 = _mm_slli_si128::<8>(t0);
                t2 = _mm_blend_epi16::<0x0F>(t1, m3);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 0, 3, 1) }>(t2)
            }
        );
        // round 5
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_unpackhi_epi32(m0, m1);
                t1 = _mm_unpacklo_epi32(m0, m2);
                _mm_unpacklo_epi64(t0, t1)
            },
            {
                t0 = _mm_srli_si128::<4>(m2);
                t1 = _mm_blend_epi16::<0x03>(m0, m3);
                _mm_blend_epi16::<0x3C>(t1, t0)
            },
            {
                t0 = _mm_blend_epi16::<0x0C>(m1, m0);
                t1 = _mm_srli_si128::<4>(m3);
                t2 = _mm_blend_epi16::<0x30>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 3, 0, 1) }>(t2)
            },
            {
                t0 = _mm_unpacklo_epi64(m2, m1);
                t1 = _mm_shuffle_epi32::<{ _mm_shuffle(2, 0, 1, 0) }>(m3);
                t2 = _mm_srli_si128::<4>(t0);
                _mm_blend_epi16::<0x33>(t1, t2)
            }
        );
        // round 6
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_slli_si128::<12>(m1);
                t1 = _mm_blend_epi16::<0x33>(m0, m3);
                _mm_blend_epi16::<0xC0>(t1, t0)
            },
            {
                t0 = _mm_blend_epi16::<0x30>(m3, m2);
                t1 = _mm_srli_si128::<4>(m1);
                t2 = _mm_blend_epi16::<0x03>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 1, 3, 0) }>(t2)
            },
            {
                t0 = _mm_unpacklo_epi64(m0, m2);
                t1 = _mm_srli_si128::<4>(m1);
                t2 = _mm_blend_epi16::<0x0C>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(3, 1, 0, 2) }>(t2)
            },
            {
                t0 = _mm_unpackhi_epi32(m1, m2);
                t1 = _mm_unpackhi_epi64(m0, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(0, 1, 2, 3) }>(t1)
            }
        );
        // round 7
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_unpackhi_epi32(m0, m1);
                t1 = _mm_blend_epi16::<0x0F>(t0, m3);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 0, 3, 1) }>(t1)
            },
            {
                t0 = _mm_blend_epi16::<0x30>(m2, m3);
                t1 = _mm_srli_si128::<4>(m0);
                t2 = _mm_blend_epi16::<0x03>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(1, 0, 2, 3) }>(t2)
            },
            {
                t0 = _mm_unpackhi_epi64(m0, m3);
                t1 = _mm_unpacklo_epi64(m1, m2);
                t2 = _mm_blend_epi16::<0x3C>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 3, 1, 0) }>(t2)
            },
            {
                t0 = _mm_unpacklo_epi32(m0, m1);
                t1 = _mm_unpackhi_epi32(m1, m2);
                t2 = _mm_unpacklo_epi64(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(2, 1, 0, 3) }>(t2)
            }
        );
        // round 8
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_unpackhi_epi32(m1, m3);
                t1 = _mm_unpacklo_epi64(t0, m0);
                t2 = _mm_blend_epi16::<0xC0>(t1, m2);
                _mm_shufflehi_epi16::<{ _mm_shuffle(1, 0, 3, 2) }>(t2)
            },
            {
                t0 = _mm_unpackhi_epi32(m0, m3);
                t1 = _mm_blend_epi16::<0xF0>(m2, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(0, 2, 1, 3) }>(t1)
            },
            {
                t0 = _mm_unpacklo_epi64(m0, m3);
                t1 = _mm_srli_si128::<8>(m2);
                t2 = _mm_blend_epi16::<0x03>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(1, 3, 2, 0) }>(t2)
            },
            {
                t0 = _mm_blend_epi16::<0x30>(m1, m0);
                _mm_shuffle_epi32::<{ _mm_shuffle(0, 3, 2, 1) }>(t0)
            }
        );
        // round 9
        round!(
            &mut row1,
            &mut row2,
            &mut row3,
            &mut row4,
            &mut buf1,
            {
                t0 = _mm_blend_epi16::<0x03>(m0, m2);
                t1 = _mm_blend_epi16::<0x30>(m1, m2);
                t2 = _mm_blend_epi16::<0x0F>(t1, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(1, 3, 0, 2) }>(t2)
            },
            {
                t0 = _mm_slli_si128::<4>(m0);
                t1 = _mm_blend_epi16::<0xC0>(m1, t0);
                _mm_shuffle_epi32::<{ _mm_shuffle(1, 2, 0, 3) }>(t1)
            },
            {
                t0 = _mm_unpackhi_epi32(m0, m3);
                t1 = _mm_unpacklo_epi32(m2, m3);
                t2 = _mm_unpackhi_epi64(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(0, 2, 1, 3) }>(t2)
            },
            {
                t0 = _mm_blend_epi16::<0xC0>(m3, m2);
                t1 = _mm_unpacklo_epi32(m0, m3);
                t2 = _mm_blend_epi16::<0x0F>(t0, t1);
                _mm_shuffle_epi32::<{ _mm_shuffle(1, 2, 3, 0) }>(t2)
            }
        );

        let mp = self.h.as_mut_ptr();
        _mm_storeu_si128(mp as *mut _, _mm_xor_si128(ff0, _mm_xor_si128(row1, row3)));
        _mm_storeu_si128(
            mp.offset(4) as *mut _,
            _mm_xor_si128(ff1, _mm_xor_si128(row2, row4)),
        );
    }
    pub fn write(&mut self, mut v: &[u8]) {
        if v.is_empty() {
            return;
        }
        if self.buf.is_full() {
            unsafe {
                self.chomp(false);
            }
        }
        let rem = 64 - self.buf.len();
        if !self.buf.is_empty() && v.len() > rem {
            let (a, b) = v.split_at(rem);
            v = b;
            let _ = self.buf.write_all_bytes(a);
            if self.buf.is_full() {
                unsafe {
                    self.chomp(false);
                }
            }
            self.written += a.len() as u64;
        }

        let mut chunks = v.chunks_exact(64);
        for c in chunks.by_ref() {
            if self.buf.is_full() {
                unsafe {
                    self.chomp(false);
                }
            } else {
                unsafe {
                    let mp: *const u32 = core::mem::transmute(c.as_ptr());
                    self.chomp_exact(mp, false);
                }
            }
            self.written += 64;
        }

        for val in chunks.remainder() {
            if self.buf.is_full() {
                unsafe {
                    self.chomp(false);
                }
            }
            let _ = self.buf.write_u8(*val);
            self.written += 1;
        }
    }
    pub fn hash(mut self, v: &[u8]) -> [u8; NN] {
        self.write(v);
        self.finish()
    }
    pub fn finish(mut self) -> [u8; NN] {
        unsafe {
            self.chomp(true);
        }

        // let out: [u8; NN] = unsafe { self.h.align_to::<u8>().1 }.copy_subset();
        let mut out: FixedU8Buf<NN> = FixedU8Buf::default();
        for v in self.h {
            let _ = v.write_le_to(&mut out);
        }
        // out
        out.take()
    }
}
impl<const NN: usize> HashDigest<32, NN> for BLAKE2s<NN> {
    fn finish(self) -> [u8; NN] {
        todo!()
    }
    fn hash(self, _: &[u8]) -> [u8; NN] {
        todo!()
    }
    fn write(&mut self, _: &[u8]) {
        todo!()
    }
}

pub type BLAKE2s128 = BLAKE2s<16>;
pub type BLAKE2s160 = BLAKE2s<20>;
pub type BLAKE2s224 = BLAKE2s<28>;
pub type BLAKE2s256 = BLAKE2s<32>;

#[cfg(test)]
mod tests {
    use crate::blake2::{BLAKE2s224, BLAKE2s256};
    use irox_tools::{assert_eq_hex_slice, hex};

    #[test]
    pub fn test0() {
        let h = BLAKE2s224::default().hash(b"");
        let exp = hex!("1fa1291e65248b37b3433475b2a0dd63d54a11ecc4e3e034e7bc1ef4");
        assert_eq_hex_slice!(exp, h);
        let h = BLAKE2s256::default().hash(b"");
        let exp = hex!("69217a3079908094e11121d042354a7c1f55b6482ca1a51e1b250dfd1ed0eef9");
        assert_eq_hex_slice!(exp, h);
    }

    #[test]
    pub fn test_abc() {
        let h = BLAKE2s256::default().hash(b"abc");
        let exp = hex!("508C5E8C327C14E2E1A72BA34EEB452F37458B209ED63A294D999B4C86675982");
        assert_eq_hex_slice!(exp, h);
    }

    #[test]
    pub fn test_long() {
        let _exp = hex!("508C5E8C327C14E2E1A72BA34EEB452F37458B209ED63A294D999B4C86675982");
        let mut inp = [0u8; 128];
        let mut v = BLAKE2s256::default();
        for i in 0..100000 {
            v.write(&inp);
            inp[0] = inp[0].wrapping_add(i as u8);
        }
    }
}
