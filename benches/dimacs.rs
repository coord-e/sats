use std::fs::File;
use std::io::{Seek, SeekFrom};

use satat::dimacs;

use criterion::{criterion_group, criterion_main, Criterion};

fn bench_dimacs_parse(c: &mut Criterion) {
    let mut group = c.benchmark_group("dimacs-parser");
    group.sample_size(20);

    group.bench_function("complete-300", |b| {
        let mut file = File::open("./benches/data/complete-300-0.1-18-98765432130018.cnf").unwrap();
        b.iter(|| {
            dimacs::parse(&file).unwrap();
            file.seek(SeekFrom::Start(0)).unwrap();
        })
    });
    group.finish();
}

criterion_group!(benches, bench_dimacs_parse);
criterion_main!(benches);
