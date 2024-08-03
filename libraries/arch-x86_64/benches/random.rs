// SPDX-License-Identifier: MIT
// Copyright 2023 IROX Contributors
//

use criterion::{criterion_group, criterion_main, Criterion, Throughput};
use irox_arch_x86_64::rand::*;

pub fn criterion_benchmark(c: &mut Criterion) {

    let mut grp = c.benchmark_group("Intel Intrinsics Rand");
    grp.throughput(Throughput::Bytes(8));
    grp.bench_function("rdrand64", |b| b.iter(|| {
        rdrand64()
    }));

    grp.finish();
    let mut grp = c.benchmark_group("Intel Intrinsics Rand");
    grp.throughput(Throughput::Bytes(4));
    grp.bench_function("rdrand32", |b| b.iter(|| {
        rdrand32()
    }));

    grp.finish();

    let mut grp = c.benchmark_group("Intel Intrinsics Rand");
    grp.throughput(Throughput::Bytes(16));
    grp.bench_function("rand128", |b| b.iter(|| {
        rand128()
    }));

    grp.finish();
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
