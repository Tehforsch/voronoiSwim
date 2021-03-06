use std::path::PathBuf;

use criterion::black_box;
use criterion::criterion_group;
use criterion::criterion_main;
use criterion::Criterion;
use voronoi_swim::command_line_args::CommandLineArgs;
use voronoi_swim::run::run;

pub fn bench_16(c: &mut Criterion) {
    c.bench_function("ics_16_1", |b| b.iter(|| run_ics_16(black_box(1))));
    c.bench_function("ics_16_1024", |b| b.iter(|| run_ics_16(black_box(1024))));
}

fn run_ics_16(num_cores: usize) {
    let args = CommandLineArgs {
        grid_files: vec![PathBuf::from("testFiles/ics_16.dat")],
        domain_decomposition: Some(num_cores),
        quiet: true,
    };
    run(&args).unwrap();
}

criterion_group! {
    name = benches;
    // This can be any expression that returns a `Criterion` object.
    config = Criterion::default().significance_level(0.1).sample_size(10);
    targets = bench_16
}

criterion_main!(benches);
