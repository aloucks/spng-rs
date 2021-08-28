use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn spng_decode(c: &mut Criterion) {
    c.bench_function("spng_decode", |b| {
        let mut buf = Vec::new();
        b.iter(|| {
            let d = spng::Decoder::new(spng_benchmarks::TEST_PNG_002);
            // the png crate changed its api so that that the OutputInfo is now returned
            // from next_frame instead of read_info
            let (_todo, mut reader) = d.read_info().unwrap();
            spng_benchmarks::reserve(&mut buf, reader.output_buffer_size());
            let _todo_info = reader.next_frame(&mut buf).unwrap();
            black_box(reader);
        })
    });
}

criterion_group!(benches, spng_decode);
criterion_main!(benches);
