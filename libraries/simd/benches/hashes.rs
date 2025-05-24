// SPDX-License-Identifier: MIT
// Copyright 2025 IROX Contributors
//

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use irox_tools::buf::ZeroedBuffer;
use irox_tools::hash::murmur3::{Murmur3_128, Murmur3_32};
use irox_tools::hash::{BLAKE2b512, BLAKE2s256, MD5, SHA256, SHA512};
use irox_tools::hex::to_hex_str_upper;
use std::io::Read;

struct Hasher {
    iter: [u8; 4096],
    b256: BLAKE2s256,
    b256simd: irox_simd::blake2::BLAKE2s256,
}
impl Default for Hasher {
    fn default() -> Self {
        Self {
            iter: [0; 4096],
            b256: BLAKE2s256::default(),
            b256simd: irox_simd::blake2::BLAKE2s256::default(),
        }
    }
}
impl Hasher {
    pub fn hash_murmur_3(&mut self) {
        let hash = Murmur3_128::new();
        let _ = hash.hash(&self.iter);
        self.iter[0] += 1;
    }
    pub fn hash_murmur_32(&mut self) {
        let hash = Murmur3_32::new();
        let _ = hash.hash(&self.iter);
        self.iter[0] += 1;
    }
    // pub fn hash_murmur_32_simd(&mut self) {
    //     let hash = irox_simd::murmur3::Murmur3_32::new();
    //     let _ = hash.hash(&self.iter);
    //     self.iter[0] += 1;
    // }
    pub fn hash_sha356(&mut self) {
        let _hash = SHA256::new().hash(&self.iter);
        self.iter[0] += 1;
    }
    pub fn hash_sha512(&mut self) {
        let _hash = SHA512::new().hash(&self.iter);
        self.iter[0] += 1;
    }
    pub fn hash_blake2s(&mut self) {
        self.b256.write(&self.iter);
        self.iter[0] += 1;
    }
    pub fn hash_blake2s_simd(&mut self) {
        self.b256simd.write(&self.iter);
        self.iter[0] += 1;
    }
    pub fn hash_blake2b(&mut self) {
        let _hash = BLAKE2b512::default().hash(&self.iter);
        self.iter[0] += 1;
    }
    pub fn hash_md5(&mut self) {
        let _hash = MD5::default().hash(&self.iter);
        self.iter[0] += 1;
    }
}

pub fn bench_blake2s_file(path: &str) {
    let mut hasher = irox_simd::blake2::BLAKE2s256::default();
    // let mut hasher = BLAKE2s256::default();
    let mut file = std::fs::File::open(path).unwrap();
    let mut buf = <Box<[u8]> as ZeroedBuffer>::new_zeroed(1024 * 1024);
    loop {
        let n = file.read(&mut buf).unwrap();
        if n == 0 {
            break;
        }
        hasher.write(&buf[..n]);
    }
    let res = hasher.finish();
    println!("{}", to_hex_str_upper(&res));
}

pub fn criterion_benchmark(c: &mut Criterion) {
    let mut hasher = Hasher::default();
    let mut grp = c.benchmark_group("sha512");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_sha512", |b| {
        b.iter(|| {
            hasher.hash_sha512();
        })
    });
    grp.finish();
    // std::thread::sleep(Duration::from_secs(20));
    let mut grp = c.benchmark_group("sha256");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_sha256", |b| {
        b.iter(|| {
            hasher.hash_sha356();
        })
    });
    grp.finish();
    // std::thread::sleep(Duration::from_secs(20));
    let mut grp = c.benchmark_group("murmur3_128");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_murmur_3", |b| {
        b.iter(|| {
            hasher.hash_murmur_3();
        })
    });
    grp.finish();
    // std::thread::sleep(Duration::from_secs(20));
    let mut grp = c.benchmark_group("murmur3_32");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_murmur_32", |b| {
        b.iter(|| {
            hasher.hash_murmur_32();
        })
    });
    // grp.bench_function("hash_murmur_32_simd", |b| {
    //     b.iter(|| {
    //         hasher.hash_murmur_32_simd();
    //     })
    // });
    grp.finish();
    // std::thread::sleep(Duration::from_secs(20));
    let mut grp = c.benchmark_group("file_blake2s256");
    grp.throughput(Throughput::Bytes(1073741824));
    grp.bench_function("blake2s256_file", |b| {
        b.iter(|| {
            bench_blake2s_file("C:/proj/BLAKE2/b2sum/test1grand");
        })
    });
    grp.finish();

    let mut grp = c.benchmark_group("blake2s256");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_blake2s256", |b| {
        b.iter(|| {
            hasher.hash_blake2s();
        })
    });
    grp.bench_function("hash_blake2s256_simd", |b| {
        b.iter(|| {
            hasher.hash_blake2s_simd();
        })
    });
    grp.finish();
    // std::thread::sleep(Duration::from_secs(20));
    let mut grp = c.benchmark_group("blake2b512");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_blake2b512", |b| {
        b.iter(|| {
            hasher.hash_blake2b();
        })
    });
    grp.finish();
    let mut grp = c.benchmark_group("md5");
    grp.throughput(Throughput::Bytes(4096));
    grp.bench_function("hash_md5", |b| {
        b.iter(|| {
            hasher.hash_md5();
        })
    });
    grp.finish();
}
pub fn config() -> Criterion {
    Criterion::default()
    // .sample_size(10)
}
criterion_group! {
    name = hashes;
    config = config();
    targets = criterion_benchmark
}
criterion_main!(hashes);
