# Next Task

Current milestone: `M14. Rust-Owned Offset Surface Descriptor Entry` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_edge_geometry()` no longer calls `self.edge_sample_occt(shape, 0.0)?.tangent` for circle or ellipse geometry.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry/payloads.rs::ported_periodic_curve_geometry()` now chooses signed periodic spans from Rust-owned payload endpoint parameters and Rust curve length evaluation, using the existing edge descriptor parameter span only when endpoint/length candidates are symmetric.
- Negative closed-period spans now survive canonicalization before endpoint snapping, preserving reversed full-circle geometry used by BRep wire area and volume paths.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::ported_curve_sampling_matches_occt()` now checks Rust start/end parameter samples against explicit OCCT oracle samples for position and tangent parity.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_areas_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'edge_sample_occt\(shape, 0\.0\)\?\.tangent' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs`, and `git diff --check`.

## Target

Remove the exercised raw face-geometry entry in `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_offset_surface()`, where public offset descriptor/payload paths still begin with `self.face_geometry_occt(shape)?` instead of the public/Rust-owned `self.face_geometry(shape)?` gate.

## Next Bounded Cut

1. Start in `ported_geometry.rs::ported_offset_surface()` and the public offset descriptor/payload tests in `ported_geometry_workflows.rs`.
2. Change `ported_offset_surface()` to call `self.face_geometry(shape)?` and then `ported_offset_surface_with_geometry(shape, geometry)`.
3. Verify that public face geometry validation does not recurse through `ported_offset_surface()` for supported offset faces; keep any unsupported-kind behavior explicit.
4. Strengthen offset descriptor, offset payload, or offset sampling assertions if the stricter entry path exposes missing coverage, then add a grep guard against `let geometry = self.face_geometry_occt(shape)?` in `ported_offset_surface()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `edge_sample_occt(shape, 0.0)?.tangent` into `ported_geometry.rs`.
- Do not reintroduce `face_geometry_occt(face_shape)` into `face_snapshot.rs`.
- Do not reintroduce `None => context.topology_occt(face_shape)?` or `context.subshapes_occt(face_shape, ShapeKind::Edge)?` into `face_topology.rs`.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M12.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.
- Do not reintroduce `context.edge_geometry_occt(edge_shape)?` or `context.edge_endpoints_occt(edge_shape)?` into `edge_topology.rs::root_edge_topology()` or `wire_topology.rs::wire_occurrence()`.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'let geometry = self\.face_geometry_occt\(shape\)\?' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs`
- `git diff --check`
