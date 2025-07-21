#![allow(unused)]

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
use index_permute::*;

struct BigData {
    data: [u8; 1000],
}

struct MediumData {
    data: [u8; 100],
}

impl BigData {
    fn new() -> Self {
        BigData { data: [0; 1000] }
    }
}

impl MediumData {
    fn new() -> Self {
        MediumData { data: [0; 100] }
    }
}

const TEST_SIZE: usize = 10000000;
pub fn criterion_benchmark(c: &mut Criterion) {
    let index = (0..TEST_SIZE).rev().collect::<Vec<_>>();
    let index = PermuteIndex::try_new(&index).unwrap();
    let mut data = (0..TEST_SIZE).collect::<Vec<_>>();
    let mut medium_data = (0..TEST_SIZE)
        .map(|_| MediumData::new())
        .collect::<Vec<_>>();
    let mut big_data = (0..TEST_SIZE).map(|_| BigData::new()).collect::<Vec<_>>();
    c.bench_function("permute_seq_small", |b| {
        b.iter(|| {
            index_permute::try_order_by_index_inplace(
                black_box(&mut data),
                black_box(index.clone()),
            )
            .unwrap();
            black_box(&mut data);
        });
    });

    c.bench_function("permute_seq_medium", |b| {
        b.iter(|| {
            index_permute::try_order_by_index_inplace(
                black_box(&mut medium_data),
                black_box(index.clone()),
            )
            .unwrap();
            black_box(&mut medium_data);
        });
    });

    c.bench_function("permute_seq_big", |b| {
        b.iter(|| {
            index_permute::try_order_by_index_inplace(
                black_box(&mut big_data),
                black_box(index.clone()),
            )
            .unwrap();
            black_box(&mut big_data);
        });
    });

    c.bench_function("permute_parallel_small", |b| {
        b.iter(|| {
            index_permute::try_order_by_index_parallel_inplace(
                black_box(&mut data),
                black_box(index.clone()),
            )
            .unwrap();
            black_box(&mut data);
        });
    });

    c.bench_function("permute_parallel_medium", |b| {
        b.iter(|| {
            index_permute::try_order_by_index_parallel_inplace(
                black_box(&mut medium_data),
                black_box(index.clone()),
            )
            .unwrap();
            black_box(&mut medium_data);
        });
    });

    c.bench_function("permute_parallel_big", |b| {
        b.iter(|| {
            index_permute::try_order_by_index_parallel_inplace(
                black_box(&mut big_data),
                black_box(index.clone()),
            )
            .unwrap();
            black_box(&mut big_data);
        });
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
