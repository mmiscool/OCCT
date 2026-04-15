# Next Task

Move the per-face BRep materialization block out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` and into `brep/face_surface.rs`.

## Focus

- Extract the `face_shapes` iteration in `Context::ported_brep()` into face-owned helper code in `brep/face_surface.rs`.
- Keep `brep.rs` responsible for top-level orchestration and final summary wiring, but not for the detailed face sample/area/materialization logic.
- Preserve the current Rust-first surface descriptor path, mesh fallback behavior, and adjacency wiring while moving that code behind the face helper boundary.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the move.

## Why This Is Next

The topology-driven prefix of `ported_brep()` now lives in `brep/topology.rs`, so the largest remaining implementation block in the parent module is the face-materialization loop. That code already depends on face-surface descriptors, area evaluation, and mesh fallback handling that conceptually belongs with `brep/face_surface.rs`, making it the next clean extraction on the way to an orchestration-only `brep.rs`.
