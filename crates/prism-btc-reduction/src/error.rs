use uor_foundation::enforcement::PipelineFailure;

/// Failure modes from the σ-convergence loop.
#[derive(Debug)]
pub enum ConvergenceFailure {
    /// All 2^32 nonces tried — the nonce fiber over the target shape is exhausted.
    ///
    /// The caller should update the block (change merkle root via coinbase, advance
    /// timestamp, etc.) and retry with a new `MiningRound`.
    FiberExhausted,

    /// The pipeline rejected a candidate that passed the target pre-filter.
    ///
    /// This indicates a discrepancy between the Bitcoin target check and the UOR
    /// shape constraint — the pipeline failure reason is recorded for diagnosis.
    ReductionStall { reason: PipelineFailure },
}

/// Failure modes from certifying an existing wire-format header via `certify_wire_bytes`.
#[derive(Debug)]
pub enum CertifyError {
    /// The byte slice was not exactly 80 bytes.
    InvalidLength { got: usize },

    /// The σ-projection output did not satisfy the UOR pipeline constraints.
    PipelineRejected(PipelineFailure),
}
