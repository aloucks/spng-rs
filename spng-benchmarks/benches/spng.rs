use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn spng_decode(c: &mut Criterion) {
    c.bench_function("spng_decode", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            let d = spng::Decoder::new(spng_benchmarks::TEST_PNG_002);
            let mut reader = d.read_info().unwrap();
            spng_benchmarks::reserve(&mut buf, reader.output_buffer_size());
            let _info = reader.next_frame(&mut buf).unwrap();
            black_box(reader);
        })
    });
}

criterion_group!(benches, spng_decode);
criterion_main!(benches);
