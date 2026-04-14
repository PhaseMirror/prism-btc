import Lake
open Lake DSL

package PrismBtcLean where

-- Depend on UOR Foundation's Lean library (mirrors the Rust crate we depend on).
-- Zero external dependencies beyond this — no mathlib.
require uor from git
  "https://github.com/UOR-Foundation/UOR-Framework"
  @ "main"

lean_lib «PrismBtc» where
  roots := #[`PrismBtc]
