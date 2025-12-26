use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

static INPUT_TEST: [i16; 64] = [
    -370, -33, 18, -10, 0, 0, 0, 0, 220, -72, -48, -12, -24, 0, 0, 0, 80, 184, -20, -28, 0, 0, 0,
    0, -76, 5, 21, 0, 15, 0, 0, 0, 15, 7, 0, -17, 0, 0, 0, 0, -7, 11, 34, 0, 0, 0, 0, 0, 0, -19, 0,
    0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
];

fn idct_naive(c: &mut Criterion) {
    let mut output = [0; 64];
    c.bench_function("idct naive", |b| {
        b.iter(|| {
            mediary_jpeg::dct::inverse::idct_naive(&INPUT_TEST, &mut output, 8, 0, 0);
            black_box(());
        })
    });
}

fn idct_precomputed(c: &mut Criterion) {
    let mut output = [0; 64];
    c.bench_function("idct precomputed", |b| {
        b.iter(|| {
            mediary_jpeg::dct::inverse::idct_precomputed(&INPUT_TEST, &mut output, 8, 0, 0);
            black_box(());
        })
    });
}

fn idct_two_pass(c: &mut Criterion) {
    let mut output = [0; 64];
    c.bench_function("idct two_pass", |b| {
        b.iter(|| {
            mediary_jpeg::dct::inverse::idct_two_pass(&INPUT_TEST, &mut output, 8, 0, 0);
            black_box(());
        })
    });
}

criterion_group!(benches, idct_naive, idct_precomputed, idct_two_pass);
criterion_main!(benches);
