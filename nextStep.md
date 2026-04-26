# Next Task

Current milestone: `M7. Public Payload Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `M6. BRep Surface Payload Fallback Cleanup` is complete for the exercised kernel slice.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs` now rebuilds planar multi-wire root-face snapshot plane payloads through `PortedSurface::from_context_with_ported_payloads()` via `ported_snapshot_plane_payload()`. The module no longer calls `face_plane_payload()` or `face_plane_payload_occt()`, so supported holed planar faces do not accept a direct OCCT plane-payload rescue after Rust extraction fails.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_face_geometry()` no longer retries direct analytic `face_plane_payload_occt()`, `face_cylinder_payload_occt()`, `face_cone_payload_occt()`, `face_sphere_payload_occt()`, or `face_torus_payload_occt()` helpers when Rust sample-derived payload extraction returns `None`.
- `rust/lean_occt/tests/brep_workflows.rs::ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes` now asserts the holed planar face raw-geometry route yields a Rust plane payload, the BRep face retained that Rust analytic surface route, and the reconstructed loops still expose outer/inner roles and expected area.
- `rust/lean_occt/src/lib.rs` public analytic face payload wrappers (`face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()`) now use `ported_analytic_face_surface_payload()`: they accept an existing `ported_face_surface()` descriptor, otherwise use raw OCCT geometry only to identify supported surface kind/bounds and then require `PortedSurface::from_context_with_geometry()` to produce the Rust payload. They no longer silently call direct `face_*_payload_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_analytic_curve_and_surface_payload_queries_match_occt` now requires matching Rust Plane, Cylinder, Cone, Sphere, and Torus descriptors before each public analytic face payload query, compares the public payload to that descriptor, and then compares against the explicit OCCT oracle.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.face_(plane|cylinder|cone|sphere|torus)_payload_occt|face_(plane|cylinder|cone|sphere|torus)_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the next public payload fallback family: `rust/lean_occt/src/lib.rs` swept face payload methods still call direct `face_revolution_payload_occt()` and `face_extrusion_payload_occt()` helpers when `ported_face_surface_descriptor()` returns `None`.

## Next Bounded Cut

1. Start with `Context::face_revolution_payload()` and `face_extrusion_payload()` in `rust/lean_occt/src/lib.rs`.
2. Split supported swept-kind failures from unsupported descriptor absence: when raw/ported face geometry identifies Revolution or Extrusion, require the Rust `PortedFaceSurface::Swept` descriptor path to produce the matching payload and return an explicit Rust error if it cannot.
3. Preserve the explicit `face_revolution_payload_occt()` and `face_extrusion_payload_occt()` methods as opt-in parity/oracle APIs, and keep test oracle usage explicit.
4. Strengthen `ported_geometry_workflows::public_swept_and_offset_payload_queries_match_occt` so exercised swept public payloads are proven to come from Rust descriptors before comparing to OCCT.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Public raw `*_payload_occt()` methods may remain available, but supported public payload wrappers should not silently use them after a Rust descriptor failure.
- Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not replace a Rust curve extraction failure with `PortedCurve::from_context_with_geometry()` in public-route BRep code; make the unsupported/error distinction explicit.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
