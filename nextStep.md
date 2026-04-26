# Next Task

Current milestone: `M17. Rust-Owned Offset Metadata for Single-Face Offset Results` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs::OffsetSurfaceFaceMetadata` now carries `PortedOffsetBasisSurface`, so direct offset metadata can represent analytic or swept bases.
- `rust/lean_occt/src/lib.rs::offset_surface_face_metadata_candidate()` records signed offset value, source basis `FaceGeometry`, and Rust analytic/swept basis descriptors for direct plane, cylinder, cone, sphere, torus, extrusion, and revolution offset bases without calling `face_offset_payload_occt()` or `face_offset_basis_geometry_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_offset_surface_from_metadata()` now builds direct swept offset descriptors from attached metadata before the raw offset metadata helpers. For swept bases it uses the retained offset value and basis geometry with Rust sample-based swept-basis reconstruction, then validates against OCCT samples as an oracle.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt()` now asserts `extrusion-direct` and `revolution-direct` direct swept offsets expose Rust-owned offset value, source basis geometry, source-mirroring swept payloads and curve spans, descriptor/public query parity, and OCCT oracle parity.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `awk '/pub\(crate\) fn ported_offset_surface_with_geometry/,/let payload = self.face_offset_payload_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'offset_surface_face_metadata'`, `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`, `awk '/fn offset_surface_face_metadata_supports_basis/,/^}/' rust/lean_occt/src/lib.rs | rg -n 'Revolution|Extrusion'`, and `git diff --check`.

## Target

Remove the same offset metadata helper dependency for exercised single-face `Context::make_offset()` results. Direct `make_offset_surface_face()` offsets now carry Rust metadata, but offset faces extracted from `make_offset(&source_face, ...)` still arrive at `ported_offset_surface_with_geometry()` without attached metadata and fall through to `face_offset_payload_occt(shape)?` and `face_offset_basis_geometry_occt(shape)?`.

## Next Bounded Cut

1. Retain single-face offset construction metadata on `Context::make_offset()` when the input source face has a supported analytic, extrusion, or revolution descriptor.
2. Propagate or reattach that metadata when supported offset face subshapes are materialized from the `make_offset()` result.
3. Narrow `ported_offset_surface_with_geometry()` so those exercised `make_offset()` offset faces use attached Rust metadata before the raw offset metadata helpers.
4. Strengthen `public_offset_basis_queries_match_occt` for the non-direct `extrusion` and `revolution` cases that already come from `make_offset()`, while keeping imported, multi-face, or unsupported offsets on the explicit raw-helper escape hatch.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_geometry_occt(shape)` into `ported_offset_surface()` or `ported_offset_face_surface_payload()`.
- Do not let metadata-attached offset faces fall through to `face_offset_payload_occt(shape)?` or `face_offset_basis_geometry_occt(shape)?`.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M16.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`
- `git diff --check`
