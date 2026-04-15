# Next Task

Move the remaining face-area dispatch helper out of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs` and into `brep/face_metrics.rs`.

## Focus

- Extract `ported_face_area_from_surface()` from `face_surface.rs` into `face_metrics.rs`, alongside `analytic_face_area()`, `analytic_ported_swept_face_area()`, and `analytic_offset_face_area()`.
- Keep `face_surface.rs` focused on face preparation, mesh fallback, and BRep face assembly after the move.
- Preserve the current `Option<f64>` behavior and the existing analytic/offset/swept dispatch order; this is a code-ownership cleanup, not a behavior change.
- Keep the call sites in `ported_brep_face()` and `ported_face_area()` thin by routing them through the shared face-metrics helper.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the extraction.

## Why This Is Next

With the swept-face reconstruction block now living in `brep/swept_face.rs`, the remaining `ported_face_area_from_surface()` helper is the next self-contained block whose ownership already matches the `brep/face_metrics.rs` module that implements the concrete area calculations it dispatches to.
