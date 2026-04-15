# Next Task

Tighten the face snapshot module boundary now that `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` is coordinator-only, leaving `load_ported_face_snapshot()` as the sole exported face snapshot entry.

## Focus

- Make the intermediate face snapshot helpers and carrier types private where possible, or otherwise collapse them so `topology.rs` depends only on `load_ported_face_snapshot()`.
- Keep the current face packing behavior, ordering, and failure handling unchanged.
- Preserve `ported_topology_snapshot()` as a thin coordinator over the root snapshot helper, the face snapshot helper, and the final snapshot constructor.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With `topology.rs` now reduced to orchestration plus a tiny constructor, the next smallest cleanup is to narrow the face snapshot surface so the coordinator no longer needs any intermediate face-stage types or helpers beyond the single top-level entry point.
