# Next Task

Carry the loader-owned edge and face `Shape` inventories through `ported_brep()` so the higher-level BRep materializers stop depending on raw `subshapes_occt()`.

## Focus

- Preload the root edge and face `Vec<Shape>` inventories once at the `ported_brep()` entry boundary instead of letting `ported_brep_edges()` and `ported_brep_faces()` enumerate them again internally.
- Thread those preserved shape vectors through [`brep_materialize.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/brep_materialize.rs) and [`face_surface.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs) so the materialization stages consume loader-owned state the same way the topology loaders now do.
- Keep the explicit `*_occt()` escape hatches intact for actual raw geometry queries, but move the edge/face traversal boundary itself up to the `ported_brep()` loader stage.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the change.

## Why This Is Next

The topology loader path now preserves per-root-wire edge shapes and per-face wire/edge shapes, so `root_wire_topology()` and `load_ported_face_snapshot()` no longer call raw `subshapes_occt()` internally. The next remaining internal traversal reload is higher up: `ported_brep_edges()` still reloads edge shapes and `ported_brep_faces()` still reloads face shapes inside the BRep materialization path, even though those inventories can be loaded once and carried through `ported_brep()` instead.
