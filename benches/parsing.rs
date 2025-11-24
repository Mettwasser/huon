use criterion::{Criterion, criterion_group, criterion_main};
use huon::{DecoderOptions, de::from_str, test_list_model::CodeInfo, test_model::Person};
use std::{fs, time::Duration};

fn parsing_benchmark(c: &mut Criterion) {
    let mut group = c.benchmark_group("parsing");
    group.measurement_time(Duration::from_secs(10));

    let input = fs::read_to_string("test.huon").unwrap();
    group.bench_function("test.huon", |b| {
        b.iter(|| {
            let _: Person = from_str(&input, DecoderOptions::default()).unwrap();
        });
    });

    let input = fs::read_to_string("test_list.huon").unwrap();
    group.bench_function("test_list.huon", |b| {
        b.iter(|| {
            let _: CodeInfo = from_str(&input, DecoderOptions::default()).unwrap();
        });
    });

    group.finish();
}

criterion_group!(benches, parsing_benchmark);
criterion_main!(benches);
