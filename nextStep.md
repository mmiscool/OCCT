# Next Task

Current milestone: `M8. Public Geometry and Sampling Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` public analytic, swept, offset, offset-basis, and edge payload wrappers no longer silently call direct `*_payload_occt()` helpers after Rust descriptor extraction returns `None`; explicit raw payload APIs remain opt-in parity/oracle APIs.
- `rust/lean_occt/src/lib.rs` public edge geometry/sampling wrappers (`edge_geometry()`, `edge_endpoints()`, `edge_sample()`, and `edge_sample_at_parameter()`) now require Rust `ported_edge_*` extraction for supported `Line`, `Circle`, and `Ellipse` edges. If Rust extraction returns `None` for those supported curve kinds, the wrapper returns a Rust-owned unsupported error instead of silently calling direct `edge_*_occt()` helpers.
- `rust/lean_occt/src/lib.rs` public face geometry/sampling wrappers (`face_geometry()`, `face_sample()`, and `face_sample_normalized()`) now require Rust `PortedFaceSurface` extraction for supported analytic (`Plane`, `Cylinder`, `Cone`, `Sphere`, `Torus`), swept (`Revolution`, `Extrusion`), and `Offset` faces. If Rust extraction returns `None` for those supported surface kinds, the wrapper returns a Rust-owned unsupported error instead of silently calling direct `face_*_occt()` helpers.
- The face geometry/sampling OCCT escape hatch is now limited to unsupported raw surface kinds, while explicit `face_geometry_occt()`, `face_sample_occt()`, and `face_sample_normalized_occt()` remain available as opt-in parity/oracle APIs.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_face_geometry()` validates swept and offset descriptors before reporting public geometry as ported, and it keeps Rust analytic recovery for raw BSpline/Bezier/unknown labels that sample as supported analytic surfaces.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs` now calls `ported_offset_surface_with_geometry()` so offset descriptor extraction uses the already-known face geometry instead of re-entering public `face_geometry()`.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::ported_surface_sampling_matches_occt`, `ported_swept_surface_sampling_matches_occt`, and `ported_offset_surface_sampling_matches_occt` now require ported face geometry/descriptors and compare public face geometry/samples against Rust results before explicit OCCT oracle checks.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.face_(sample|sample_normalized|geometry)_occt\\(' rust/lean_occt/src/lib.rs`, `! rg -n 'ported_offset_surface\\(face_shape\\)|eprintln!|validated shell mesh|offset shell bbox|shape summary bbox failed|offset shell face bbox' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_prepare.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the remaining public vertex/topology/subshape fallback family in `rust/lean_occt/src/lib.rs`: `vertex_point()`, `topology()`, `subshape_count()`, `subshape()`, and `subshapes()` still have direct OCCT helper escape paths around Rust topology extraction.

## Next Bounded Cut

1. Start with the public topology/subshape family in `rust/lean_occt/src/lib.rs`: `topology()`, `subshape_count()`, `subshape()`, and `subshapes()`, then include `vertex_point()` if the same topology inventory gives a bounded Rust-owned replacement.
2. Require Rust `ported_topology()`/`ported_subshapes()` behavior before public supported topology queries can succeed; do not silently rescue through direct OCCT helpers after ported topology extraction can identify the shape.
3. Preserve explicit raw OCCT APIs as opt-in parity/oracle APIs or strictly unsupported-topology escape hatches.
4. Strengthen `brep_workflows`, `document_workflows`, and `selector_workflows` so public traversal/count/materialization is compared against Rust topology before explicit OCCT oracle comparison.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Do not reintroduce direct `*_payload_occt()` fallbacks into public payload wrappers; explicit raw payload APIs may remain available only as opt-in parity/oracle APIs.
- Do not reopen supported line/circle/ellipse public edge geometry or sampling fallbacks; direct `edge_*_occt()` helpers may only remain public oracles or unsupported curve-kind escape hatches.
- Do not reopen supported analytic/swept/offset public face geometry or sampling fallbacks; direct `face_*_occt()` helpers may only remain public oracles or unsupported surface-kind escape hatches.
- Once a Rust descriptor returns `Some(...)`, mismatched payload, geometry, sampling, or topology requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()`, `ported_edge_geometry()`, or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs` or the public swept payload wrappers.
- Do not reintroduce direct `face_offset_payload_occt()`, `face_offset_basis_*_occt()`, or `face_offset_basis_curve_*_occt()` fallbacks inside public wrappers narrowed under M7.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `rg -n 'pub fn (vertex_point|topology|subshape_count|subshape|subshapes)' rust/lean_occt/src/lib.rs`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
