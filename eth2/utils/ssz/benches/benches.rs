#[macro_use]
extern crate criterion;

use criterion::black_box;
use criterion::{Benchmark, Criterion};
use ssz::{Decodable, Encodable};

fn criterion_benchmark(c: &mut Criterion) {
    let n = 8196;

    let vec: Vec<u64> = vec![4242; 8196];
    c.bench(
        &format!("vec_of_{}_u64", n),
        Benchmark::new("as_ssz_bytes", move |b| {
            b.iter_with_setup(|| vec.clone(), |vec| black_box(vec.as_ssz_bytes()))
        })
        .sample_size(100),
    );

    let vec: Vec<u64> = vec![4242; 8196];
    let bytes = vec.as_ssz_bytes();
    c.bench(
        &format!("vec_of_{}_u64", n),
        Benchmark::new("from_ssz_bytes", move |b| {
            b.iter_with_setup(
                || bytes.clone(),
                |bytes| {
                    let vec: Vec<u64> = Vec::from_ssz_bytes(&bytes).unwrap();
                    black_box(vec)
                },
            )
        })
        .sample_size(100),
    );
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
