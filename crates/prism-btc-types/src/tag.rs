//! Zero-sized phantom tag for `Grounded<ConstrainedTypeInput, Tag>`.
//!
//! `GroundedShape` is sealed to `ConstrainedTypeInput`; the phantom `Tag`
//! parameter of `Grounded<T, Tag>` is how prism-btc distinguishes its
//! certificate at the type level from other domains.

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub struct BlockHashTag;
