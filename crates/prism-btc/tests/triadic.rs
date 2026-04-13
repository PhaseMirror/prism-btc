use prism_btc::TriadicCoords;

#[test]
fn genesis_triadic_coords() {
    // Genesis hash: 000000000019d6689c085ae165831e934ff763ae46a2a6c172b3f1b60a8ce26f
    let hash: [u8; 32] = [
        0x00, 0x00, 0x00, 0x00, 0x00, 0x19, 0xd6, 0x68, 0x9c, 0x08, 0x5a, 0xe1, 0x65, 0x83, 0x1e,
        0x93, 0x4f, 0xf7, 0x63, 0xae, 0x46, 0xa2, 0xa6, 0xc1, 0x72, 0xb3, 0xf1, 0xb6, 0x0a, 0x8c,
        0xe2, 0x6f,
    ];
    let coords = TriadicCoords::from_hash(&hash);

    // First 5 bytes are 0x00 — bits 0..4 of spectrum must be clear
    assert_eq!(
        coords.spectrum & 0x1f,
        0,
        "first 5 bytes should be zero in spectrum"
    );

    // Genesis target requires 3 leading zero bytes — must satisfy
    assert!(coords.satisfies_target(3));

    // Byte 5 is 0x19 (non-zero) — 6 leading zeros must fail
    assert!(!coords.satisfies_target(6));
}
