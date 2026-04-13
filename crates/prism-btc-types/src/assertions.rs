/// Compile-time ring identity assertions.
///
/// These `uor!` assertions guard against ring arithmetic regression at compile time.
/// Any change to the UOR ring operations that breaks the core identity
/// `neg(bnot(x)) = succ(x)` will cause a `compile_error!` here — failing the build
/// before any runtime code executes.
///
/// Two Witt levels are covered:
/// - W32 (Z/(2^32)Z): the nonce ring, spot checks at x=0, x=42
/// - W8  (Z/(2^8)Z, `@Q7` suffix): the per-byte hash ring, spot checks at x=0, x=255
use uor_foundation::uor;

// W32 identity checks (nonce ring). Each uor! assertion call is an expression that
// evaluates to an Assertion struct. Wrapping in `const _:` ensures the embedded
// `const _: () = assert!(...)` inside the block fires at compile time.
#[allow(dead_code)]
const _RING_ASSERT_NEG_BNOT_42: uor_foundation::enforcement::Assertion =
    uor! { assert neg(bnot(42)) = 43; };

#[allow(dead_code)]
const _RING_ASSERT_NEG_BNOT_0: uor_foundation::enforcement::Assertion =
    uor! { assert neg(bnot(0)) = 1; };

// W8 identity checks (per-byte hash ring, @Q7 denotes Witt level 7 = 2^8)
#[allow(dead_code)]
const _RING_ASSERT_W8_NEG_BNOT_0: uor_foundation::enforcement::Assertion =
    uor! { assert neg(bnot(0@Q7)) = 1@Q7; };

#[allow(dead_code)]
const _RING_ASSERT_W8_NEG_BNOT_255: uor_foundation::enforcement::Assertion =
    uor! { assert neg(bnot(255@Q7)) = 0@Q7; };
