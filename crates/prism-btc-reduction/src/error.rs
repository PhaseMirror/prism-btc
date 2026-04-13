use uor_foundation::enforcement::PipelineFailure;

/// Errors that can occur during the nonce-iteration mining loop.
#[derive(Debug)]
pub enum MineError {
    /// The pipeline rejected a candidate that passed the target pre-filter.
    ///
    /// This indicates a discrepancy between the Bitcoin target check and the UOR
    /// shape constraint — the nonce and failure reason are recorded for diagnosis.
    PipelineFailed { nonce: u32, reason: PipelineFailure },

    /// All 2^32 nonces were exhausted without finding a valid block hash.
    ///
    /// The caller should update the block (change merkle root via coinbase, advance
    /// timestamp, etc.) and retry.
    NonceSpaceExhausted,
}
