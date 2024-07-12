use criterion::{black_box, criterion_group, criterion_main, Criterion};

use shell_quote::Fish;

fn criterion_benchmark(c: &mut Criterion) {
    let empty_string = "";
    c.bench_function("fish escape empty", |b| {
        b.iter(|| Fish::quote_vec(black_box(empty_string)))
    });

    let alphanumeric_short = "abcdefghijklmnopqrstuvwxyz0123456789";
    c.bench_function("fish escape a-z", |b| {
        b.iter(|| Fish::quote_vec(black_box(alphanumeric_short)))
    });

    let alphanumeric_long = alphanumeric_short.repeat(1000);
    c.bench_function("fish escape a-z long", |b| {
        b.iter(|| Fish::quote_vec(black_box(&alphanumeric_long)))
    });

    let bytes_short = (1..=255u8).map(char::from).collect::<String>();
    c.bench_function("fish escape bytes", |b| {
        b.iter(|| Fish::quote_vec(black_box(&bytes_short)))
    });

    let bytes_long = bytes_short.repeat(1000);
    c.bench_function("fish escape bytes long", |b| {
        b.iter(|| Fish::quote_vec(black_box(&bytes_long)))
    });

    let utf8 = ('\x01'..=char::MAX).collect::<String>();
    c.bench_function("fish escape utf-8", |b| {
        b.iter(|| Fish::quote_vec(black_box(&utf8)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
