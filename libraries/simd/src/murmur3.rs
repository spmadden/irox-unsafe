use crate::blake2::_mm_shuffle;
use irox_bits::{array_split_8, MutBits};
use irox_tools::buf::{Buffer, FixedU8Buf};
use std::arch::x86_64::{
    __m256i, _mm256_add_epi32, _mm256_cvtsi256_si32, _mm256_loadu_si256, _mm256_mul_epu32,
    _mm256_or_si256, _mm256_set1_epi32, _mm256_set_epi32, _mm256_setzero_si256,
    _mm256_shuffle_epi32, _mm256_slli_si256, _mm256_srli_epi32, _mm256_srli_si256,
    _mm256_unpacklo_epi32, _mm256_xor_si256,
};
use std::ops::BitXorAssign;

const C1: u64 = 0x87c3_7b91_1142_53d5;
const C2: u64 = 0x4cf5_ad43_2745_937f;
const C3: u64 = 0xff51_afd7_ed55_8ccd;
const C4: u64 = 0xc4ce_b9fe_1a85_ec53;
const C5: u32 = 0xCC9E2D51;
const C6: u32 = 0x1B873593;

macro_rules! fmix64 {
    ($k:expr) => {
        $k.bitxor_assign($k >> 33);
        $k = $k.wrapping_mul(C3);
        $k.bitxor_assign($k >> 33);
        $k = $k.wrapping_mul(C4);
        $k.bitxor_assign($k >> 33);
    };
}

macro_rules! round32 {
    ($h:expr, $k:expr) => {
        *$h ^= $k.wrapping_mul(C5).rotate_left(15).wrapping_mul(C6);
        *$h = $h.rotate_left(13);
        *$h = $h.wrapping_mul(5).wrapping_add(0xe6546b64);
    };
}
macro_rules! C5 {
    () => {
        _mm256_set1_epi32(C5)
    };
}
macro_rules! round32_8x_simd {
    ($h:expr, $k:expr) => {
        *$h ^= $k.wrapping_mul(C5).rotate_left(15).wrapping_mul(C6);
        *$h = $h.rotate_left(13);
        *$h = $h.wrapping_mul(5).wrapping_add(0xe6546b64);
    };
}
macro_rules! mul32 {
    ($a:expr,$b:expr) => {
        _mm256_unpacklo_epi32(
            _mm256_shuffle_epi32::<{ _mm_shuffle(0, 0, 2, 0) }>(_mm256_mul_epu32($a, $b)),
            _mm256_shuffle_epi32::<{ _mm_shuffle(0, 0, 2, 0) }>(_mm256_mul_epu32(
                _mm256_srli_si256::<4>($a),
                _mm256_srli_si256::<4>($b),
            )),
        )
    };
}
macro_rules! rotl32 {
    ($a:expr,$b:literal) => {
        _mm256_or_si256(
            _mm256_slli_si256::<$b>($a),
            _mm256_srli_si256::<{ 32 - $b }>($a),
        )
    };
}
pub struct Murmur3_32 {
    h: __m256i,
    buf: FixedU8Buf<32>,
    total_len: u64,
}
impl Murmur3_32 {
    pub fn new() -> Self {
        Self::new_seeded(0)
    }
    pub fn new_seeded(seed: u32) -> Self {
        let seed = unsafe { _mm256_set1_epi32(seed as i32) };
        Self {
            h: seed,
            total_len: 0,
            buf: Default::default(),
        }
    }
    pub fn write(&mut self, mut key: &[u8]) {
        let align = 4 - self.buf.len();
        if align < 4 && align < key.len() {
            let (a, b) = key.split_at(align);
            key = b;
            for val in a {
                let _ = self.buf.write_u8(*val);
                self.total_len += 1;
                if self.buf.is_full() {
                    self.try_chomp();
                }
            }
        }
        let mut chunks = key.chunks_exact(32);
        for c in chunks.by_ref() {
            unsafe {
                self.chomp8(&c);
            }
            self.total_len += 32;
        }
        for b in chunks.remainder() {
            let _ = self.buf.push_back(*b);
            self.total_len += 1;
            self.try_chomp();
        }
    }
    fn try_chomp(&mut self) {
        if !self.buf.is_full() {
            return;
        }
        let k = self.buf.as_buf_default();
        unsafe {
            self.chomp8(&k);
        }
    }
    unsafe fn chomp8(&mut self, buf: &[u8]) {
        if buf.len() < 32 {
            return;
        }
        let e1 = _mm256_set_epi32(
            0xA329EB99u32 as i32,
            0xBE6214AEu32 as i32,
            0x4DC33A4D,
            0x1FCC49A2,
            0xC9031C00u32 as i32,
            0x4093AEE0,
            0xB33F1B01u32 as i32,
            0xB19CE1AAu32 as i32,
        );
        let e2 = _mm256_set_epi32(
            0xC3059BF0u32 as i32,
            0x4A99D300,
            0x55FEC879,
            0x82A03CCEu32 as i32,
            0x91BB3DDDu32 as i32,
            0x1E919393,
            0xA39B1CE1u32 as i32,
            0xDE33BB89u32 as i32,
        );
        let k = _mm256_loadu_si256(buf.as_ptr() as *const _);
        let k = mul32!(k, e1);
        let k = rotl32!(k, 17);
        let k = mul32!(k, e2);
        self.h = _mm256_xor_si256(self.h, k);
        self.h = rotl32!(self.h, 13);
        self.h = mul32!(self.h, _mm256_set1_epi32(5));
        self.h = _mm256_add_epi32(self.h, _mm256_set1_epi32(0x3B11A33C));
    }
    pub fn hash(mut self, key: &[u8]) -> u32 {
        self.write(key);
        self.finish()
    }
    pub fn finish(mut self) -> u32 {
        if !self.buf.is_empty() {
            let b = self.buf.as_buf_default();
            unsafe {
                self.chomp8(&b);
            }
        }
        let mut h = self.h;
        let l = self.total_len;
        unsafe {
            h = _mm256_xor_si256(h, _mm256_set1_epi32(l as i32));
            h = _mm256_xor_si256(h, _mm256_srli_epi32::<16>(h));
            h = mul32!(h, _mm256_set1_epi32(0xC2AD39BBu32 as i32));
            h = _mm256_xor_si256(h, _mm256_srli_epi32::<13>(h));
            h = mul32!(h, _mm256_set1_epi32(0xE037A692u32 as i32));
            h = _mm256_xor_si256(h, _mm256_srli_epi32::<16>(h));
            let mut out = 0u32;
            for _ in 0..8 {
                out ^= _mm256_cvtsi256_si32(h) as u32;
                h = _mm256_srli_si256::<4>(h);
            }
            out
        }
    }
}

#[derive(Default)]
pub struct Murmur3_128 {
    h1: u64,
    h2: u64,
    buf: FixedU8Buf<16>,
    total_len: u64,
}
impl Murmur3_128 {
    pub fn new() -> Self {
        Default::default()
    }
    pub fn new_seeded(seed: u64) -> Self {
        Self {
            h1: seed,
            h2: seed,
            total_len: 0,
            buf: Default::default(),
        }
    }
    pub fn hash(mut self, key: &[u8]) -> u128 {
        self.write(key);
        self.finish()
    }
    pub fn write(&mut self, mut key: &[u8]) {
        let align = 16 - self.buf.len();
        if align < 16 && align < key.len() {
            let (a, b) = key.split_at(align);
            key = b;
            for val in a {
                let _ = self.buf.write_u8(*val);
                self.total_len += 1;
                if self.buf.is_full() {
                    self.try_chomp();
                }
            }
        }
        let mut chunks = key.chunks_exact(16);
        for c in chunks.by_ref() {
            let (k1, k2) = c.split_at(8);
            let k1 = u64::from_le_bytes(k1.try_into().unwrap_or_default());
            let k2 = u64::from_le_bytes(k2.try_into().unwrap_or_default());
            self.h1
                .bitxor_assign(k1.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2));
            self.h1 = self
                .h1
                .rotate_left(27)
                .wrapping_add(self.h2)
                .wrapping_mul(5)
                .wrapping_add(0x52dce729);
            self.h2
                .bitxor_assign(k2.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1));
            self.h2 = self
                .h2
                .rotate_left(31)
                .wrapping_add(self.h1)
                .wrapping_mul(5)
                .wrapping_add(0x38495ab5);
            self.total_len += 16;
        }
        for b in chunks.remainder() {
            let _ = self.buf.push_back(*b);
            self.total_len += 1;
            self.try_chomp();
        }
    }
    fn try_chomp(&mut self) {
        if !self.buf.is_full() {
            return;
        }
        let chunk_16 = self.buf.as_buf_default();
        let (k1, k2) = array_split_8(chunk_16);
        let k1 = u64::from_le_bytes(k1);
        let k2 = u64::from_le_bytes(k2);
        self.h1
            .bitxor_assign(k1.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2));
        self.h1 = self
            .h1
            .rotate_left(27)
            .wrapping_add(self.h2)
            .wrapping_mul(5)
            .wrapping_add(0x52dce729);
        self.h2
            .bitxor_assign(k2.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1));
        self.h2 = self
            .h2
            .rotate_left(31)
            .wrapping_add(self.h1)
            .wrapping_mul(5)
            .wrapping_add(0x38495ab5);
    }
    pub fn finish(mut self) -> u128 {
        if !self.buf.is_empty() {
            let mut k1: u64 = 0;
            let mut k2: u64 = 0;
            let len = self.buf.len();
            let mut iter = self.buf.iter();

            let mut shift: u32 = 0;
            for i in 0..len {
                let Some(val) = iter.next() else {
                    break;
                };
                let val: u64 = *val as u64;
                if i == 8 {
                    shift = 0;
                }
                if i >= 8 {
                    k2.bitxor_assign(val.wrapping_shl(shift));
                } else {
                    k1.bitxor_assign(val.wrapping_shl(shift));
                }
                shift += 8;
            }
            if len > 8 {
                self.h2
                    .bitxor_assign(k2.wrapping_mul(C2).rotate_left(33).wrapping_mul(C1));
            }
            self.h1
                .bitxor_assign(k1.wrapping_mul(C1).rotate_left(31).wrapping_mul(C2));
        }
        self.h1.bitxor_assign(self.total_len);
        self.h2.bitxor_assign(self.total_len);
        self.h1 = self.h1.wrapping_add(self.h2);
        self.h2 = self.h2.wrapping_add(self.h1);
        fmix64!(self.h1);
        fmix64!(self.h2);
        self.h1 = self.h1.wrapping_add(self.h2);
        self.h2 = self.h2.wrapping_add(self.h1);

        ((self.h1 as u128) << 64) | self.h2 as u128
    }
}

#[cfg(test)]
mod test {
    extern crate alloc;
    use crate::murmur3::{Murmur3_128, Murmur3_32};
    use alloc::vec::Vec;
    use irox_tools::assert_eq_hex;

    #[test]
    pub fn tests() {
        let tests: Vec<(&'static str, u128)> = vec![
            ("", 0x0000000000000000_0000000000000000_u128),
            ("1", 0x71FBBBFE8A7B7C71_942AEB9BF9F0F637_u128),
            ("12", 0x4A533C6209E3FD95_88C72C695E0B311D_u128),
            ("123", 0x985B2D1B0D667F6A_427EA1E3CE0ECF69_u128),
            ("1234", 0x0897364D218FE7B4_341E8BD92437FDA5_u128),
            ("12345", 0x20F83A176B21DFCB_F13C5C41325CA9F4_u128),
            ("123456", 0xE417CF050BBBD0D6_51A48091002531FE_u128),
            ("1234567", 0x2CDAC5F7F2C623A2_37DC518BCAE1D955_u128),
            ("12345678", 0x3B4A640638B1419C_913B0E676BD42557_u128),
            ("123456789", 0x3C84645EDB66CCA4_99f8FAC73A1EA105_u128),
            ("1234567890", 0xECFA4AE68079870A_C1D017C820EBD22B_u128),
            ("12345678901", 0x2A84FB1385B327D3_DAEB95857DE0DFC1_u128),
            ("123456789012", 0xDDA6E38B7C022914_75A23983FD719D1E_u128),
            ("1234567890123", 0xE3DDF2853772DF49_1BC521F05EEF2497_u128),
            ("12345678901234", 0x7D51E170E83CCC91_C63D6CBEFAF85AD0_u128),
            ("123456789012345", 0x887001AEA2AFCFD6_1EC326364F0801B3_u128),
            ("1234567890123456", 0x4FBE5DC5C0E32CF8_C0C8E96B60C322C1_u128),
            (
                "12345678901234567",
                0x748617968026B77E_291E6386473F7103_u128,
            ),
            (
                "123456789012345678",
                0xEAEAE51CCFA961AF_754C657D52CC0469_u128,
            ),
            (
                "1234567890123456789",
                0x0C722FBA0A479959_4EBBCD6912218A2A_u128,
            ),
            (
                "12345678901234567890",
                0xB11CD81925DC8C3A_719F603CE8F1367D_u128,
            ),
            (
                "123456789012345678901",
                0xA2D7F23C16EE6855_FEE63702A5F53DD3_u128,
            ),
            (
                "1234567890123456789010",
                0x37208BC7AE7E7076_EFA979587AABB8AF_u128,
            ),
            ("Hello, world!", 0xF1512DD1D2D665DF_2C326650A8F3C564_u128),
        ];
        for (data, exp) in tests {
            let hash = Murmur3_128::new().hash(data.as_bytes());
            assert_eq!(exp, hash);
            let mut hash = Murmur3_128::new();
            hash.write(data.as_bytes());
            let hash = hash.finish();
            assert_eq!(exp, hash);
        }
    }

    #[test]
    pub fn test_32() {
        let tests = [
            ("", 0x00000000u32, 0x00000000u32),
            ("", 0x00000001, 0x514E28B7),
            ("", 0xFFFFFFFF, 0x81F16F39),
            ("test", 0x00000000, 0xBA6BD213),
            ("test", 0x9747B28C, 0x704B81DC),
            ("Hello, world!", 0x00000000, 0xC0363E43),
            ("Hello, world!", 0x9747B28C, 0x24884CBA),
            (
                "The quick brown fox jumps over the lazy dog",
                0x00000000,
                0x2E4FF723,
            ),
            (
                "The quick brown fox jumps over the lazy dog",
                0x9747B28C,
                0x2FA826CD,
            ),
        ];
        for (inp, seed, exp) in tests {
            let hash = Murmur3_32::new_seeded(seed).hash(inp.as_bytes());
            assert_eq_hex!(exp, hash);
        }
    }
}
