//! Benchmarks for QR validation performance

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use image::{DynamicImage, Luma};
use qrai_scanner_core::{decode_only, validate};

/// Create a test QR code with specified content
fn create_qr(content: &str) -> Vec<u8> {
    let code = qrcode::QrCode::new(content.as_bytes()).unwrap();
    let img = code.render::<Luma<u8>>().build();

    let mut buf = Vec::new();
    let dyn_img = DynamicImage::ImageLuma8(img);
    dyn_img
        .write_to(&mut std::io::Cursor::new(&mut buf), image::ImageFormat::Png)
        .unwrap();
    buf
}

/// Benchmark decode_only (fast path)
fn bench_decode_only(c: &mut Criterion) {
    let qr_bytes = create_qr("https://example.com");

    c.bench_function("decode_only", |b| {
        b.iter(|| decode_only(black_box(&qr_bytes)))
    });
}

/// Benchmark full validation (with stress tests)
fn bench_validate(c: &mut Criterion) {
    let qr_bytes = create_qr("https://example.com");

    c.bench_function("validate_full", |b| {
        b.iter(|| validate(black_box(&qr_bytes)))
    });
}

/// Benchmark with different QR sizes
fn bench_qr_sizes(c: &mut Criterion) {
    let mut group = c.benchmark_group("qr_sizes");

    // Different content lengths produce different QR sizes
    let sizes = vec![
        ("small", "Hi"),
        ("medium", "https://example.com/page"),
        ("large", "https://example.com/very/long/url/with/many/segments/that/creates/larger/qr"),
    ];

    for (name, content) in sizes {
        let qr_bytes = create_qr(content);

        group.bench_with_input(
            BenchmarkId::new("decode_only", name),
            &qr_bytes,
            |b, qr| b.iter(|| decode_only(black_box(qr))),
        );

        group.bench_with_input(
            BenchmarkId::new("validate", name),
            &qr_bytes,
            |b, qr| b.iter(|| validate(black_box(qr))),
        );
    }

    group.finish();
}

/// Benchmark image loading overhead
fn bench_image_load(c: &mut Criterion) {
    let qr_bytes = create_qr("https://example.com");

    c.bench_function("image_load_only", |b| {
        b.iter(|| image::load_from_memory(black_box(&qr_bytes)))
    });
}

criterion_group!(
    benches,
    bench_decode_only,
    bench_validate,
    bench_qr_sizes,
    bench_image_load,
);

criterion_main!(benches);
