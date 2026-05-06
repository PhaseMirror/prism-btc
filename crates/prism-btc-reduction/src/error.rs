/// Failure modes from the σ-convergence loop.
///
/// `block_hash_shape_certificate` is infallible (const-validated), so the only
/// way `run_convergence` can fail is by exhausting the 2^32 nonce fiber.
#[derive(Debug)]
pub enum ConvergenceFailure {
    /// All 2^32 nonces tried — the nonce fiber over the target shape is exhausted.
    ///
    /// The caller should update the block (change merkle root via coinbase, advance
    /// timestamp, etc.) and retry with a new `MiningRound`.
    FiberExhausted,
}

/// Failure mode from certifying an existing wire-format header.
///
/// The shape pipeline is infallible; the only thing that can go wrong with
/// `certify_wire_bytes` is the byte-slice length.
#[derive(Debug)]
pub struct InvalidLength {
    pub got: usize,
}
