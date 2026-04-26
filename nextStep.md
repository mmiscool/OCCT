# Next Task

Current milestone: `M7. Public Payload Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` public analytic face payload wrappers (`face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()`) use `ported_analytic_face_surface_payload()` and no longer silently call direct analytic `face_*_payload_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public swept face payload wrappers (`face_revolution_payload()` and `face_extrusion_payload()`) now use `ported_swept_face_surface_payload()`: they accept an existing `ported_face_surface_descriptor()` result, otherwise use raw OCCT face geometry only to confirm the requested swept kind and then require `brep::ported_face_surface_descriptor()` to produce the Rust-owned `PortedFaceSurface::Swept` payload.
- The public swept wrappers no longer call direct `face_revolution_payload_occt()` or `face_extrusion_payload_occt()` helpers after Rust descriptor extraction returns `None`; the explicit swept `*_payload_occt()` APIs remain available as opt-in parity/oracle APIs.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_swept_and_offset_payload_queries_match_occt` now requires matching Rust Extrusion and Revolution descriptors before public swept payload queries, compares those public payloads to the Rust descriptors, and only then compares against explicit OCCT oracles.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.face_(revolution|extrusion)_payload_occt|face_(revolution|extrusion)_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_analytic_curve_and_surface_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the next public payload fallback family: `rust/lean_occt/src/lib.rs::face_offset_payload()` still calls direct `face_offset_payload_occt()` when `ported_offset_surface()` returns `None`.

## Next Bounded Cut

1. Start with `Context::face_offset_payload()` in `rust/lean_occt/src/lib.rs`.
2. Split supported offset failures from unsupported descriptor absence: when raw/ported face geometry identifies an Offset face, require the Rust `PortedOffsetSurface` descriptor path to produce the payload and return an explicit Rust error if it cannot.
3. Preserve `face_offset_payload_occt()` as an opt-in parity/oracle API, and keep test oracle usage explicit.
4. Strengthen `ported_geometry_workflows::public_swept_and_offset_payload_queries_match_occt` so exercised offset public payloads are proven to come from Rust offset descriptors before comparing to OCCT.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Public raw `*_payload_occt()` methods may remain available, but supported public payload wrappers should not silently use them after a Rust descriptor failure.
- Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs` or the public swept payload wrappers.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'None => self.face_offset_payload_occt|face_offset_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
