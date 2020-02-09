use std::time::Duration;

use criterion::{
    criterion_group, criterion_main, Criterion, Throughput,
};

use hilbert::HilbertPrecompute;
use hilbert_reference::xy2d;

#[inline(always)]
fn bench_distance<D>(distance: D, n: u32) -> u64 where D: Fn(u32, u32) -> u64 {
    let mut count = 0;
    let n = n as u32;
    for x in 0 .. n {
        for y in 0 .. n {
            count += distance(x, y);
        }
    }
    count
}

#[inline(always)]
fn bench_compare_inner<T, I>(outer: &T, inner: &I, n: u32) -> u64 where I: Fn(&T, u32, u32) -> bool {
    let mut count = 0;
    for x1 in 0 .. n {
        for y1 in 0 .. n {
            if inner(outer, x1, y1) {
                count += 1;
            }
        }
    }
    count
}

#[inline(always)]
fn bench_compare<O, T, I>(outer: O, inner: I, n: u32) -> u64 where O: Fn(u32, u32) -> T, I: Fn(&T, u32, u32) -> bool {
    let mut count = 0;

    for x0 in 0 .. n {
        for y0 in 0 .. n {
            let intermediate = outer(x0, y0);
            count += bench_compare_inner(&intermediate, &inner, n);
        }
    }

    count
}

#[allow(dead_code)]
fn distance(c: &mut Criterion) {
    let n: u32 = 25;
    let mut distance = c.benchmark_group("distance");
    distance.throughput(Throughput::Elements((n * n) as u64));
    distance.warm_up_time(Duration::from_micros(100_000));

    distance.sample_size(150);
    distance.bench_function("reference", move |b| {
        b.iter(|| bench_distance(xy2d, n))
    });

    distance.sample_size(150);
    distance.bench_function("optimized", move |b| {
        b.iter(|| bench_distance(|x,y| HilbertPrecompute::new(x, y).distance(), n))
    });

    distance.finish();
}

#[allow(dead_code)]
fn compare(c: &mut Criterion) {
    let n: u32 = 15;
    let mut compare = c.benchmark_group("compare");
    compare.throughput(Throughput::Elements((n * n * n * n) as u64));
    compare.warm_up_time(Duration::from_micros(100_000));

    compare.sample_size(50);
    compare.bench_function("reference", |b| {
        b.iter(|| bench_compare(|x0, y0| xy2d(x0, y0), |d, x1, y1| d < &xy2d(x1, y1), n))
    });

    compare.sample_size(100);
    compare.bench_function("optimized", |b| {
        b.iter(|| bench_compare(|x0, y0| HilbertPrecompute::new(x0, y0), |d, x1, y1| d < &(x1, y1), n))
    });

    compare.finish();
}

criterion_group!(benches, distance, compare);
criterion_main!(benches);
