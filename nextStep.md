# Next Task

Deduplicate the single-face topology materialization in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs`.

## Focus

- Replace the repeated wire and edge construction in `single_face_topology()` and `single_face_topology_public()` with shared helper code.
- Reuse the topology-owned BRep materialization helpers where they fit, while preserving the internal/raw vs public/Rust-first geometry split.
- Keep `brep.rs` as orchestration-only and avoid moving detailed face or edge materialization logic back into the parent module.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the cleanup.

## Why This Is Next

`ported_brep()` is now reduced to top-level orchestration plus summary wiring, and the largest remaining local duplication in this area is inside `face_surface.rs`. The internal and public single-face topology builders both reconstruct the same wire and edge scaffolding, with only the geometry and curve acquisition path differing, so that is the next clean reduction.
