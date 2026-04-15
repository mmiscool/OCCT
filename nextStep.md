# Next Task

Carry the topology-owned `Shape` vectors through the Rust topology loaders so internal face and wire enumeration can stop depending on raw `subshapes_occt()`.

## Focus

- Extend the root and face snapshot loading path to preserve face, wire, edge, and vertex `Vec<Shape>` inventories alongside the existing Rust topology indices instead of dropping them immediately after OCCT enumeration.
- Keep the public `subshape()` / `subshapes()` routing Rust-first, but remove the remaining internal raw face/wire traversal in `load_ported_face_snapshot()` and the root topology loaders by consuming those preserved shape vectors directly.
- Preserve explicit `*_occt()` escape hatches for any caller that truly needs raw traversal, and keep the topology builder acyclic while moving those internal inventory lookups onto Rust-owned state.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the change.

## Why This Is Next

The public topology-backed `subshape_count()`, `subshape()`, and `subshapes()` route is now Rust-first for face, wire, edge, and vertex kinds, but the topology construction path still materializes those same inventories through raw `subshapes_occt()` because the loader stages discard the aligned `Shape` vectors they already fetch. The next real traversal boundary is preserving and reusing that loader-owned shape state instead of re-enumerating it later.
