# Next Task

Move the remaining helper-only structs out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` and into the sibling modules that actually own them.

## Focus

- Move `CurveDifferential` and `OffsetCurveDifferential` into `brep/face_metrics.rs`.
- Move `MeshFaceProperties` into `brep/summary.rs`.
- Move `SingleFaceTopology` alongside `single_face_topology()` and `single_face_topology_public()` instead of leaving the type in `brep.rs`.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the move.

## Why This Is Next

`brep.rs` is now down to the public BRep types, the `Context` entry points, and a small set of helper-only structs that are only consumed by sibling modules. Moving those last private data carriers to their owning modules finishes the cleanup path that the recent `topology`, `swept_face`, `mesh`, and `math` extractions started.
