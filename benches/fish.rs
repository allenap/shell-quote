use criterion::{black_box, criterion_group, criterion_main, Criterion};

use shell_quote::Fish;

fn criterion_benchmark(c: &mut Criterion) {
    let empty_string = "";
    c.bench_function("fish escape empty", |b| {
        b.iter(|| Fish::quote(black_box(empty_string)))
    });

    let alphanumeric_short = "abcdefghijklmnopqrstuvwxyz0123456789";
    c.bench_function("fish escape a-z", |b| {
        b.iter(|| Fish::quote(black_box(alphanumeric_short)))
    });

    let alphanumeric_long = alphanumeric_short.repeat(1000);
    c.bench_function("fish escape a-z long", |b| {
        b.iter(|| Fish::quote(black_box(&alphanumeric_long)))
    });

    let complex_short = (1..=255u8).map(char::from).collect::<String>();
    c.bench_function("fish escape complex", |b| {
        b.iter(|| Fish::quote(black_box(&complex_short)))
    });

    let complex_long = complex_short.repeat(1000);
    c.bench_function("fish escape complex long", |b| {
        b.iter(|| Fish::quote(black_box(&complex_long)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
