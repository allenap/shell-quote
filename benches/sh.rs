use criterion::{black_box, criterion_group, criterion_main, Criterion};

use shell_quote::Sh;

fn criterion_benchmark(c: &mut Criterion) {
    let empty_string = "";
    c.bench_function("sh escape empty", |b| {
        b.iter(|| Sh::quote_vec(black_box(empty_string)))
    });

    let alphanumeric_short = "abcdefghijklmnopqrstuvwxyz0123456789";
    c.bench_function("sh escape a-z", |b| {
        b.iter(|| Sh::quote_vec(black_box(alphanumeric_short)))
    });

    let alphanumeric_long = alphanumeric_short.repeat(1000);
    c.bench_function("sh escape a-z long", |b| {
        b.iter(|| Sh::quote_vec(black_box(&alphanumeric_long)))
    });

    let complex_short = (1..=255u8).map(char::from).collect::<String>();
    c.bench_function("sh escape complex", |b| {
        b.iter(|| Sh::quote_vec(black_box(&complex_short)))
    });

    let complex_long = complex_short.repeat(1000);
    c.bench_function("sh escape complex long", |b| {
        b.iter(|| Sh::quote_vec(black_box(&complex_long)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
