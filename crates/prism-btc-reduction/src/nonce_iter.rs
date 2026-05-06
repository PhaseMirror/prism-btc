/// Iterates the full u32 nonce space [0, 2^32).
///
/// The nonce is the free dimension of the mining morphism — it is iterated in plain
/// Rust, external to the UOR pipeline. The structural enforcement of `freeRank = 0`
/// is that `Grounded<ConstrainedTypeInput, BlockHashTag>` can only be produced by
/// `uor_foundation::pipeline::run` / `run_const`. User code cannot fabricate a
/// `Grounded<T, Tag>` — the constructor is `pub(crate)` inside `uor-foundation`.
pub(crate) struct NonceIter {
    current: u32,
    exhausted: bool,
}

impl NonceIter {
    pub(crate) fn new() -> Self {
        Self {
            current: 0,
            exhausted: false,
        }
    }

    /// Returns `Some(nonce)` and advances the cursor; returns `None` after `u32::MAX`.
    pub(crate) fn next_nonce(&mut self) -> Option<u32> {
        if self.exhausted {
            return None;
        }
        let n = self.current;
        if n == u32::MAX {
            self.exhausted = true;
        } else {
            self.current = n + 1;
        }
        Some(n)
    }
}

impl Default for NonceIter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonce_iter_zero() {
        let mut iter = NonceIter::new();
        assert_eq!(iter.next_nonce(), Some(0));
        assert_eq!(iter.next_nonce(), Some(1));
        assert_eq!(iter.next_nonce(), Some(2));
    }

    #[test]
    fn nonce_iter_exhaustion() {
        let mut iter = NonceIter::new();
        // advance to u32::MAX - 1
        iter.current = u32::MAX - 1;
        assert_eq!(iter.next_nonce(), Some(u32::MAX - 1));
        assert_eq!(iter.next_nonce(), Some(u32::MAX));
        assert_eq!(iter.next_nonce(), None);
        assert_eq!(iter.next_nonce(), None); // stays exhausted
    }
}
