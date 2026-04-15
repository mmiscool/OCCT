# Next Task

Move the successful `ported_brep()` summary face path off the all-raw descriptor route in a face-kind-scoped way.

## Focus

- Keep the new public/Rust-first `brep.faces` materialization path in [`face_surface.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs), but start shrinking the temporary raw-summary split introduced in [`brep.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs).
- Thread a summary-specific face route or mixed face inventory so successful Rust-topology `ported_brep()` can reuse the Rust-first/public face preparation for the safe face families first:
  analytic faces and the offset faces that already pass the public parity checks.
- Keep swept extrusion/revolution summary inputs explicitly raw-stable until the swept summary formulas and descriptor selection are verified against the existing OCCT-parity tests.
- Preserve the current guardrails added in [`ported_geometry.rs`](rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs) and [`face_snapshot.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs): mismatched “plane” classification should decline back to fallback behavior, not hard-error.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the change.

## Why This Is Next

The successful Rust-topology BRep path now materializes `brep.faces` through the public/Rust-first face preparation route, and the new parity coverage in `brep_workflows.rs` locks that in. The remaining conservative boundary is summary derivation: `ported_shape_summary()` still consumes a raw-stable face inventory for all faces. The next aggressive but bounded step is to retire that all-raw summary split incrementally, starting with the face kinds that already have stable Rust-first descriptors and areas.
