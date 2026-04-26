# Next Task

Current milestone: `M18. Rust-Owned Offset Metadata for Multi-Face Offset Results` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs::ShapeRustMetadata` now has `SingleFaceOffsetResult`, and `Context::make_offset()` records it with `offset_surface_face_metadata_candidate()` for supported single-source offset results without calling `face_offset_payload_occt()` or `face_offset_basis_geometry_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs::load_root_topology_snapshot()` now reattaches that metadata to the generated offset face only when the result has exactly one face and that face is `SurfaceKind::Offset`.
- `rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs::ported_offset_surface_with_geometry()` therefore consumes attached Rust metadata for exercised non-direct extrusion and revolution `make_offset()` faces before the raw offset metadata helpers.
- `rust/lean_occt/tests/ported_geometry_workflows.rs::public_offset_basis_queries_match_occt()` now asserts those non-direct swept offset faces expose Rust-owned offset value, source basis geometry, source-mirroring swept payloads, basis curve spans, descriptor/public query parity, and OCCT oracle parity.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `awk '/pub\(crate\) fn ported_offset_surface_with_geometry/,/let payload = self.face_offset_payload_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'offset_surface_face_metadata'`, `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`, `awk '/fn attach_single_face_offset_metadata/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs | rg -n 'single_face_offset_result_metadata|with_offset_surface_face_metadata|SurfaceKind::Offset'`, and `git diff --check`.

## Target

Remove the same offset metadata helper dependency for exercised multi-face `Context::make_offset()` shell/solid results. Single-face `make_offset()` offsets now carry Rust metadata to the generated face, but offset faces inside shell/solid results still arrive at `ported_offset_surface_with_geometry()` without per-face metadata and can fall through to `face_offset_payload_occt(shape)?` and `face_offset_basis_geometry_occt(shape)?`.

## Next Bounded Cut

1. Build a source-face offset metadata inventory during `Context::make_offset()` for supported multi-face source shapes, starting with the offset-solid/shell fixtures already exercised by `brep_workflows`.
2. Map one generated `SurfaceKind::Offset` face family back to source descriptors using existing topology/geometry data and sample validation, and attach `OffsetSurfaceFaceMetadata` before BRep materialization or public descriptor queries consume those faces.
3. Add regression coverage around the exercised multi-face branch, preferably `ported_brep_uses_rust_owned_volume_for_offset_solids`, that proves public offset payload and basis descriptor construction no longer needs raw metadata helpers for the mapped offset faces.
4. Keep imported, unsupported, or ambiguous multi-face mappings on the explicit raw-helper escape hatch until the mapping can be made deterministic.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_geometry_occt(shape)` into `ported_offset_surface()` or `ported_offset_face_surface_payload()`.
- Do not let metadata-attached offset faces fall through to `face_offset_payload_occt(shape)?` or `face_offset_basis_geometry_occt(shape)?`.
- Keep `SingleFaceOffsetResult` attachment limited to exactly-one-face `SurfaceKind::Offset` results; multi-face attachment needs an explicit validated source/generated mapping.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M16.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`
- `git diff --check`
