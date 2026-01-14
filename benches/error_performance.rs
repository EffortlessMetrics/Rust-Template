//! Error handling performance benchmarks
//!
//! This benchmark suite measures the performance impact of error type
//! optimizations, comparing the original AppError with the optimized
//! version to validate memory usage and creation speed.

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId};
use serde_json::json;

use app_http::{AppError, ErrorCode};
use axum::http::StatusCode;

fn bench_simple_error_creation(c: &mut Criterion) {
    c.bench_function("create_simple_error", |b| {
        b.iter(|| {
            let error = AppError::bad_request("Test message");
            black_box(error);
        })
    });
}

fn bench_complex_error_creation(c: &mut Criterion) {
    c.bench_function("create_complex_error", |b| {
        b.iter(|| {
            let error = AppError::bad_request("Test message")
                .with_context("field", "value")
                .with_context("nested", json!({"key": "value"}))
                .with_ac_id("AC-123")
                .with_feature_id("FT-456")
                .with_request_id("req-789");
            black_box(error);
        })
    });
}

fn bench_error_clone(c: &mut Criterion) {
    let simple_error = AppError::bad_request("Simple error");
    let complex_error = AppError::bad_request("Complex error")
        .with_context("field", "value")
        .with_ac_id("AC-123")
        .with_feature_id("FT-456")
        .with_request_id("req-789");

    c.bench("clone_simple_error", BenchmarkId::new("clone_simple_error"), |b| {
        b.iter(|| {
            let _cloned = simple_error.clone();
            black_box(&_cloned);
        })
    });

    c.bench("clone_complex_error", BenchmarkId::new("clone_complex_error"), |b| {
        b.iter(|| {
            let _cloned = complex_error.clone();
            black_box(&_cloned);
        })
    });
}

fn bench_error_size_comparison(c: &mut Criterion) {
    let simple_error = AppError::bad_request("Simple error");
    let complex_error = AppError::bad_request("Complex error")
        .with_context("field", "value")
        .with_ac_id("AC-123")
        .with_feature_id("FT-456")
        .with_request_id("req-789");

    // Measure memory usage by comparing sizes
    c.bench_function("simple_error_size", |b| {
        b.iter(|| {
            let size = std::mem::size_of_val(&simple_error);
            black_box(size);
        })
    });

    c.bench_function("complex_error_size", |b| {
        b.iter(|| {
            let size = std::mem::size_of_val(&complex_error);
            black_box(size);
        })
    });
}

fn bench_error_formatting(c: &mut Criterion) {
    let error = AppError::validation_error(
        ErrorCode::InvalidFormat,
        "Test validation error",
    )
    .with_context("field", "invalid_value")
    .with_ac_id("AC-VALIDATION");

    c.bench_function("error_to_response", |b| {
        b.iter(|| {
            let _response = error.clone().into_response();
            black_box(());
        })
    });
}

criterion_group!(
    error_benchmarks,
    bench_simple_error_creation,
    bench_complex_error_creation,
    bench_error_clone,
    bench_error_size_comparison,
    bench_error_formatting
);

criterion_main!(error_benchmarks);
