# Next Task

Current milestone: `M13. Rust-Owned Periodic Edge Geometry Direction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs::ported_snapshot_plane_payload()` now calls public `Context::face_geometry(face_shape)?` instead of direct `context.face_geometry_occt(face_shape)?`.
- The plane snapshot path still uses `PortedSurface::from_context_with_ported_payloads()` after the public geometry gate, preserving explicit `Ok(None)` behavior for non-plane faces while forcing supported planar snapshot construction through the Rust-owned face query contract.
- `rust/lean_occt/tests/brep_workflows.rs::ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes()` now validates the holed planar snapshot plane payload through public `face_geometry()` rather than a raw geometry route.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `! rg -n 'face_geometry_occt\(face_shape\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs`, `git diff --check`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## Target

Remove the exercised raw tangent sample dependency in `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_edge_geometry()`, where the circle and ellipse branches still call `self.edge_sample_occt(shape, 0.0)?.tangent` after Rust payload extraction.

## Next Bounded Cut

1. Start in `ported_geometry.rs::ported_edge_geometry()` and `ported_geometry/payloads.rs::ported_periodic_curve_geometry()`.
2. Replace the direct `edge_sample_occt(shape, 0.0)?.tangent` calls for circle and ellipse geometry with Rust-owned periodic direction inference from the ported payload, endpoint parameters, and edge length.
3. Preserve explicit unsupported behavior for non-ported curve kinds without reintroducing direct OCCT sampling into supported line/circle/ellipse public geometry.
4. Strengthen public curve sampling or BRep exact-curve coverage if the stricter direction logic exposes an uncovered arc case, then keep a grep guard against `edge_sample_occt(shape, 0.0)?.tangent`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
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
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_curve_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_curve_bounding_boxes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'edge_sample_occt\(shape, 0\.0\)\?\.tangent' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs`
- `git diff --check`
