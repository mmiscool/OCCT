# Next Task

Current milestone: `M16. Rust-Owned Swept Offset Surface Metadata Extraction` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs::Shape` now carries Rust-owned offset construction metadata for direct analytic offset faces created by `Context::make_offset_surface_face()`.
- `rust/lean_occt/src/lib.rs::offset_surface_face_metadata_candidate()` records the signed offset value, basis `FaceGeometry`, and Rust `PortedSurface` descriptor for direct plane, cylinder, cone, sphere, and torus offset bases without calling `face_offset_payload_occt()` or `face_offset_basis_geometry_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_offset_surface_with_geometry()` now checks attached shape metadata before the OCCT offset metadata helpers and builds `OffsetSurfacePayload` plus `PortedOffsetBasisSurface::Analytic` from Rust-owned data. When metadata is attached but invalid, it returns `None` instead of falling through to the raw metadata helpers.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt()` now asserts direct analytic offset payloads, basis geometry, and basis payloads mirror their source Rust basis face before the broader OCCT parity checks run.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml`.

## Target

Remove the same direct offset metadata helper dependency for direct swept offset faces. `make_offset_surface_face()` still attaches metadata only for analytic bases, so direct extrusion and revolution offset faces still fall through to `face_offset_payload_occt(shape)?` and `face_offset_basis_geometry_occt(shape)?` inside `ported_offset_surface_with_geometry()`.

## Next Bounded Cut

1. Broaden `OffsetSurfaceFaceMetadata` so it can carry an analytic or swept `PortedOffsetBasisSurface`, not only a `PortedSurface`.
2. In `Context::make_offset_surface_face()`, collect swept extrusion/revolution basis descriptors through the existing Rust swept face descriptor path and retain the source swept basis geometry.
3. Teach `ported_offset_surface_with_geometry()` to build direct swept offset descriptors from attached metadata before the raw metadata helpers.
4. Strengthen `public_offset_basis_queries_match_occt` so `extrusion-direct` and `revolution-direct` prove direct swept offset payloads, basis geometry, swept payloads, and basis curves mirror the source Rust swept basis while still matching OCCT oracle helpers.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_geometry_occt(shape)` into `ported_offset_surface()` or `ported_offset_face_surface_payload()`.
- Do not let metadata-attached direct offset faces fall through to `face_offset_payload_occt(shape)?` or `face_offset_basis_geometry_occt(shape)?`.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M15.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`
