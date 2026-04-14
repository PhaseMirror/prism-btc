extern crate alloc;
use alloc::format;

use crate::types::{JsBlockHeader, JsMiningResult};
use prism_btc::{BlockHeader, MerkleRoot, MiningRound, Target};
use prism_btc_primitives::{Bits, Timestamp, Version};
use wasm_bindgen::prelude::*;

/// Mine a block header from JavaScript.
///
/// Returns a `JsMiningResult` on success, or throws a JS error string on failure.
///
/// # Arguments
/// * `js_header` — block header fields (version, prev_hash, merkle_root, timestamp, bits)
/// * `nbits`     — compact target encoding (e.g. `0x1d00ffff` for genesis)
#[wasm_bindgen]
pub fn mine_block(js_header: &JsBlockHeader, nbits: u32) -> Result<JsMiningResult, JsValue> {
    let header = BlockHeader {
        version: Version(js_header.version),
        prev_hash: *js_header.prev_hash_bytes(),
        merkle_root: MerkleRoot::from_bytes(*js_header.merkle_root_bytes()),
        timestamp: Timestamp(js_header.timestamp),
        bits: Bits(js_header.bits),
    };

    MiningRound::new(header, Target::new(nbits))
        .converge()
        .map(|cert| {
            JsMiningResult::new(
                cert.coords().datum,
                cert.coords().stratum,
                cert.coords().spectrum,
            )
        })
        .map_err(|e| JsValue::from_str(&format!("{:?}", e)))
}
