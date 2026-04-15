# Next Task

Split the root BRep materialization helpers out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs` into a dedicated sibling module.

## Focus

- Move `ported_brep_vertices()`, `ported_brep_wires()`, and `ported_brep_edges()` out of `brep/topology.rs` into a focused helper module such as `brep/brep_materialize.rs`.
- Keep those helpers reusing the existing topology accessor functions from `topology.rs`; this is an ownership cleanup, not a behavior change.
- Leave `topology.rs` focused on topology snapshot construction, wire/edge matching, and generic topology accessors only.
- Preserve the current behavior for `Context::ported_brep()` and the internal face assembly path.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the face-only helpers and root-shape query wrappers now split out, the biggest remaining top-level ownership chunk in `topology.rs` is the root BRep materialization trio. Pulling those into their own sibling module is the next bounded step toward making `topology.rs` purely about topology construction and traversal.
