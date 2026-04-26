# Next Task

Current milestone: `M15. Rust-Owned Offset Surface Metadata Extraction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_offset_surface()` now calls `self.face_geometry(shape)?` before `ported_offset_surface_with_geometry()`, so public offset descriptor and payload entry goes through the M8 public/Rust-owned face geometry gate.
- `rust/lean_occt/src/lib.rs::ported_offset_face_surface_payload()` now also uses `self.face_geometry(shape)?` for its secondary descriptor retry instead of bypassing the public gate with `self.face_geometry_occt(shape)?`.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt()` now checks public, ported, and OCCT face-geometry parity for every exercised offset basis family before public offset payload and basis queries run.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! awk '/pub\(crate\) fn ported_offset_surface/,/^    }/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'face_geometry_occt\(shape\)'`, `! awk '/fn ported_offset_face_surface_payload/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_geometry_occt\(shape\)'`, and `git diff --check`.

## Target

Remove or strictly narrow the direct offset metadata helpers in `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_offset_surface_with_geometry()`, where the exercised public offset path still obtains `OffsetSurfacePayload` and basis `FaceGeometry` with `self.face_offset_payload_occt(shape)?` and `self.face_offset_basis_geometry_occt(shape)?`.

## Next Bounded Cut

1. Start with analytic offset bases covered by `public_offset_basis_queries_match_occt`: plane, cylinder, cone, sphere, and torus.
2. Build a Rust-owned metadata candidate that classifies the analytic basis and recovers the signed offset value from public offset geometry, orientation, and reconstructed basis descriptor validation instead of calling `face_offset_payload_occt()`/`face_offset_basis_geometry_occt()` on that exercised path.
3. Keep swept extrusion/revolution offset metadata as a named follow-up if the analytic family lands cleanly; do not mix the offset sampling OCCT dependency in the same cut unless the same implementation removes it for the exercised analytic path.
4. Strengthen coverage so analytic offset payloads and basis geometry are compared against the existing OCCT oracle helpers while the production descriptor path uses the Rust-owned metadata candidate.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_geometry_occt(shape)` into `ported_offset_surface()` or `ported_offset_face_surface_payload()`.
- Do not reintroduce `edge_sample_occt(shape, 0.0)?.tangent` into `ported_geometry.rs`.
- Do not reintroduce `face_geometry_occt(face_shape)` into `face_snapshot.rs`.
- Do not reintroduce `None => context.topology_occt(face_shape)?` or `context.subshapes_occt(face_shape, ShapeKind::Edge)?` into `face_topology.rs`.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M14.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.
- Do not reintroduce `context.edge_geometry_occt(edge_shape)?` or `context.edge_endpoints_occt(edge_shape)?` into `edge_topology.rs::root_edge_topology()` or `wire_topology.rs::wire_occurrence()`.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
