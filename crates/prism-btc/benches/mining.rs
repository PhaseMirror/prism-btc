use criterion::{black_box, criterion_group, criterion_main, Criterion, Throughput};
use prism_btc::{serialize_header, sha256d, BlockHeader, Target, TriadicCoords};
use prism_btc_primitives::{Bits, Timestamp, Version};
use prism_btc_types::MerkleRoot;

fn genesis_header() -> BlockHeader {
    // Merkle root in Bitcoin internal byte order.
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

fn bench_serialize_header(c: &mut Criterion) {
    let header = genesis_header();
    let mut g = c.benchmark_group("hot_path");
    g.throughput(Throughput::Elements(1));
    g.bench_function("serialize_header", |b| {
        b.iter(|| serialize_header(black_box(&header), black_box(0u32)))
    });
    g.finish();
}

fn bench_sha256d(c: &mut Criterion) {
    let header = genesis_header();
    let buf = serialize_header(&header, 2083236893);
    let mut g = c.benchmark_group("hot_path");
    g.throughput(Throughput::Bytes(80));
    g.bench_function("sha256d_80_bytes", |b| b.iter(|| sha256d(black_box(&buf))));
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
    bench_serialize_header,
    bench_sha256d,
    bench_target_check,
    bench_triadic_coords
);
criterion_main!(benches);
