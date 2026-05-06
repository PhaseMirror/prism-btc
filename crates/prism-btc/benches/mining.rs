use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use prism_btc::{BlockHeader, MiningRound, Target, TriadicCoords};
use prism_btc_types::{Bits, MerkleRoot, Timestamp, Version};

fn genesis_header() -> BlockHeader {
    // Merkle root in Bitcoin internal byte order (reversed from display).
    // Display: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
    // Internal: 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a
    let merkle_bytes: [u8; 32] = [
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f,
        0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e,
        0x5e, 0x4a,
    ];
    BlockHeader {
        version: Version(1),
        prev_hash: [0u8; 32],
        merkle_root: MerkleRoot::from_bytes(merkle_bytes),
        timestamp: Timestamp(1231006505),
        bits: Bits(0x1d00ffff),
    }
}

fn bench_convergence_easy(c: &mut Criterion) {
    // Use a very easy target (0x207fffff) so the bench completes in microseconds.
    // This benchmarks the observable public API — σ-convergence via MiningRound.
    let easy_target = Target::new(0x207fffff);
    let mut g = c.benchmark_group("hot_path");
    g.throughput(Throughput::Elements(1));
    g.bench_function("convergence_easy_target", |b| {
        b.iter(|| {
            MiningRound::new(black_box(genesis_header()), black_box(easy_target))
                .converge()
                .expect("easy target must converge")
        })
    });
    g.finish();
}

fn bench_target_check(c: &mut Criterion) {
    let target = Target::new(Target::GENESIS_NBITS);
    // A hash that does NOT satisfy the target (hot-path reject case)
    let non_satisfying: [u8; 32] = [
        0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
        0x00, 0x00,
    ];
    let mut g = c.benchmark_group("hot_path");
    g.throughput(Throughput::Elements(1));
    g.bench_function("target_check_reject", |b| {
        b.iter(|| target.is_satisfied_by_bytes(black_box(&non_satisfying)))
    });
    g.finish();
}

fn bench_triadic_coords(c: &mut Criterion) {
    // Genesis hash (display/big-endian order)
    let genesis_hash: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e,
        0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c,
        0xe2, 0x6f,
    ];
    let mut g = c.benchmark_group("hot_path");
    g.throughput(Throughput::Elements(1));
    g.bench_function("triadic_coords_from_hash", |b| {
        b.iter(|| TriadicCoords::from_hash(black_box(&genesis_hash)))
    });
    g.finish();
}

criterion_group!(
    benches,
    bench_convergence_easy,
    bench_target_check,
    bench_triadic_coords
);
criterion_main!(benches);
