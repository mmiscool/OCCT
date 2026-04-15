# Next Task

Tighten the root snapshot module boundary now that the face snapshot side is narrowed, leaving `load_root_topology_snapshot()` as the sole exported root snapshot entry.

## Focus

- Make the intermediate root snapshot helpers and carrier types private where possible, or otherwise collapse them so `topology.rs` depends only on `load_root_topology_snapshot()`.
- Keep the current root loading, wire/edge matching, and ordering behavior unchanged.
- Preserve `ported_topology_snapshot()` as a thin coordinator over the root snapshot helper, the face snapshot helper, and the final snapshot constructor.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the face snapshot side narrowed to a single exported entry, the next symmetric cleanup is to collapse the remaining root-stage carrier types and helper exports so `topology.rs` no longer needs anything from `root_topology.rs` beyond the top-level loader.
