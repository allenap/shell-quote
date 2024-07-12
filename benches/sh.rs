use criterion::{black_box, criterion_group, criterion_main, Criterion};

use shell_quote::Sh;

fn criterion_benchmark(c: &mut Criterion) {
    let empty_string = "";
    c.bench_function("sh/dash escape empty", |b| {
        b.iter(|| Sh::quote_vec(black_box(empty_string)))
    });

    let alphanumeric_short = "abcdefghijklmnopqrstuvwxyz0123456789";
    c.bench_function("sh/dash escape a-z", |b| {
        b.iter(|| Sh::quote_vec(black_box(alphanumeric_short)))
    });

    let alphanumeric_long = alphanumeric_short.repeat(1000);
    c.bench_function("sh/dash escape a-z long", |b| {
        b.iter(|| Sh::quote_vec(black_box(&alphanumeric_long)))
    });

    let bytes_short = (1..=255u8).map(char::from).collect::<String>();
    c.bench_function("sh/dash escape bytes", |b| {
        b.iter(|| Sh::quote_vec(black_box(&bytes_short)))
    });

    let bytes_long = bytes_short.repeat(1000);
    c.bench_function("sh/dash escape bytes long", |b| {
        b.iter(|| Sh::quote_vec(black_box(&bytes_long)))
    });

    let utf8 = ('\x01'..=char::MAX).collect::<String>();
    c.bench_function("sh/dash escape utf-8", |b| {
        b.iter(|| Sh::quote_vec(black_box(&utf8)))
    });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
