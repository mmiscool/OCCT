# Next Task

Current milestone: `M7. Public Payload Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs` public analytic face payload wrappers (`face_plane_payload()`, `face_cylinder_payload()`, `face_cone_payload()`, `face_sphere_payload()`, and `face_torus_payload()`) use `ported_analytic_face_surface_payload()` and no longer silently call direct analytic `face_*_payload_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public swept face payload wrappers (`face_revolution_payload()` and `face_extrusion_payload()`) use `ported_swept_face_surface_payload()` and no longer call direct swept `face_*_payload_occt()` helpers after Rust descriptor extraction returns `None`.
- `rust/lean_occt/src/lib.rs` public top-level offset payload wrapper (`face_offset_payload()`) now uses `ported_offset_face_surface_payload()`: it accepts `ported_offset_surface()` when available, otherwise uses raw OCCT face geometry only to confirm the face is `SurfaceKind::Offset` and then requires `brep::ported_face_surface_descriptor()` to produce a Rust `PortedFaceSurface::Offset` payload.
- The public offset wrapper no longer calls direct `face_offset_payload_occt()` after Rust descriptor extraction returns `None`; the explicit `face_offset_payload_occt()` API remains available as an opt-in parity/oracle API.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_swept_and_offset_payload_queries_match_occt` now requires a matching Rust Offset descriptor before the public offset payload query, compares that public payload to the descriptor, and only then compares against the explicit OCCT oracle.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt` now requires matching Rust Offset descriptors before every public offset payload query across analytic, swept, direct-face, and offset-shape basis cases.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => self.face_offset_payload_occt|face_offset_payload_occt\\(shape\\)' rust/lean_occt/src/lib.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the next public payload fallback family: `rust/lean_occt/src/lib.rs::face_offset_basis_geometry()` and the public offset-basis surface payload wrappers still call direct `face_offset_basis_*_occt()` helpers when `ported_offset_surface()` returns `None`.

## Next Bounded Cut

1. Start with the offset-basis face-surface family in `rust/lean_occt/src/lib.rs`: `face_offset_basis_geometry()`, `face_offset_basis_plane_payload()`, `face_offset_basis_cylinder_payload()`, `face_offset_basis_cone_payload()`, `face_offset_basis_sphere_payload()`, `face_offset_basis_torus_payload()`, `face_offset_basis_revolution_payload()`, and `face_offset_basis_extrusion_payload()`.
2. Reuse or generalize `ported_offset_face_surface_payload()` so supported Offset faces require a Rust `PortedOffsetSurface` descriptor before public offset-basis geometry or surface payload APIs can succeed.
3. Preserve the explicit `face_offset_basis_*_occt()` APIs as opt-in parity/oracle APIs, and keep test oracle usage explicit.
4. Keep mismatched basis requests as explicit Rust errors, and keep `public_offset_basis_queries_match_occt` proving descriptor routing before OCCT oracle comparison.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Public raw `*_payload_occt()` methods may remain available, but supported public payload wrappers should not silently use them after a Rust descriptor failure.
- Once a Rust descriptor returns `Some(...)`, mismatched payload requests should fail explicitly in Rust instead of trying another OCCT helper.
- Do not reintroduce OCCT line, circle, or ellipse payload helper rescues into `PortedCurve::from_context_with_geometry()` or BRep edge materialization.
- Do not reintroduce OCCT plane, cylinder, cone, sphere, or torus payload helper rescues into `PortedSurface::from_context_with_geometry()`, `ported_face_geometry()`, BRep face materialization, or planar face snapshot reconstruction.
- Do not reintroduce `face_extrusion_payload_occt()` or `face_revolution_payload_occt()` inside `brep/swept_face.rs` or the public swept payload wrappers.
- Do not reintroduce the direct `face_offset_payload_occt()` fallback inside `face_offset_payload()`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'None => self.face_offset_basis_(geometry|plane_payload|cylinder_payload|cone_payload|sphere_payload|torus_payload|revolution_payload|extrusion_payload)_occt\\(shape\\)|face_offset_basis_(geometry|plane_payload|cylinder_payload|cone_payload|sphere_payload|torus_payload|revolution_payload|extrusion_payload)_occt\\(shape\\)' rust/lean_occt/src/lib.rs`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
