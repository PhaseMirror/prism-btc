use prism_btc_wasm::{mine_block, JsBlockHeader};
use wasm_bindgen_test::*;

wasm_bindgen_test_configure!(run_in_browser);

#[wasm_bindgen_test]
fn test_mine_block_genesis() {
    // Merkle root in Bitcoin internal byte order (reversed from display).
    // Display: 4a5e1e4baab89f3a32518a88c31bc87f618f76673e2cc77ab2127b7afdeda33b
    // Internal: 3ba3edfd7a7b12b27ac72c3e67768f617fc81bc3888a51323a9fb8aa4b1e5e4a
    let merkle_root: Vec<u8> = vec![
        0x3b, 0xa3, 0xed, 0xfd, 0x7a, 0x7b, 0x12, 0xb2, 0x7a, 0xc7, 0x2c, 0x3e, 0x67, 0x76, 0x8f,
        0x61, 0x7f, 0xc8, 0x1b, 0xc3, 0x88, 0x8a, 0x51, 0x32, 0x3a, 0x9f, 0xb8, 0xaa, 0x4b, 0x1e,
        0x5e, 0x4a,
    ];

    let header = JsBlockHeader::new(1, vec![0u8; 32], merkle_root, 1231006505, 0x1d00ffff)
        .expect("valid genesis header");

    let result = mine_block(&header, 0x1d00ffff).expect("genesis must succeed");
    assert_eq!(result.nonce, 2083236893);
}
