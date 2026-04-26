# Next Task

Current milestone: `M19. Rust-Owned Offset Metadata for Ambiguous Multi-Source Offset Results` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/lib.rs::ShapeRustMetadata` now has `MultiFaceOffsetResult(Vec<OffsetSurfaceFaceMetadata>)`, and `Context::make_offset()` builds that inventory for multi-face sources by walking source faces through `offset_surface_face_metadata_candidate()` without calling `face_offset_payload_occt()` or `face_offset_basis_geometry_occt()`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs::attach_offset_result_face_metadata()` now handles both single-face and multi-face offset results. Multi-face generated offset faces are validated against the retained source inventory through `Context::ported_offset_surface_from_metadata()` before `with_offset_surface_face_metadata(...)` is applied.
- Multi-face attachment tries both signed offset variants for each source candidate, which covers the exercised offset-solid case where generated offset faces may report `+2.5` or `-2.5` while sharing the same supported swept source family.
- `load_root_topology_snapshot()` now propagates `MultiFaceOffsetResult` metadata onto public shell subshapes and attaches validated metadata to shell face inventories, so `subshapes(&offset, ShapeKind::Shell)` followed by shell-local face descriptor queries stays on the Rust metadata path for the exercised solid.
- `ported_brep_uses_rust_owned_volume_for_offset_solids()` now asserts public generated offset face payloads, swept basis descriptors, ellipse basis curves, descriptor sample parity against OCCT, and shell-subshape descriptor preservation.
- Verification passed: `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo fmt --manifest-path rust/lean_occt/Cargo.toml && cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo fmt --manifest-path rust/lean_occt/Cargo.toml && cargo check --manifest-path rust/lean_occt/Cargo.toml && cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`, `awk '/fn attach_multi_face_offset_metadata/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs | rg -n 'ported_offset_surface_from_metadata|with_offset_surface_face_metadata|offset_value: -candidate\.offset_value'`, `awk '/pub\(crate\) fn ported_offset_surface_with_geometry/,/let payload = self.face_offset_payload_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'offset_surface_face_metadata'`, and `git diff --check`.

## Target

Remove the remaining raw offset metadata helper dependency for generated offset faces whose multi-face source inventory has multiple supported candidates. M18 deliberately leaves ambiguous or unmatched generated faces without attached metadata, so those faces can still reach `face_offset_payload_occt(shape)?` and `face_offset_basis_geometry_occt(shape)?` in `ported_offset_surface_with_geometry()`.

## Next Bounded Cut

1. Add or identify a multi-source offset fixture where `Context::make_offset()` records more than one supported source-face metadata candidate and at least one generated `SurfaceKind::Offset` face remains ambiguous under the current sample-validation-only mapper.
2. Extend the Rust metadata inventory with a deterministic source signature, such as source face kind, parameter bounds, swept basis curve kind/span, orientation, or sampled basis points, sufficient to choose one source/generated assignment without querying raw offset metadata.
3. Attach the selected metadata to root and shell generated offset face handles and keep ambiguous cases explicit only when the deterministic signature genuinely cannot decide.
4. Strengthen regression coverage so public root and shell offset face payload, basis geometry, swept basis curve, and descriptor sampling stay green without relying on the raw offset metadata helpers for the newly covered ambiguous family.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `face_geometry_occt(shape)` into `ported_offset_surface()` or `ported_offset_face_surface_payload()`.
- Do not let metadata-attached offset faces fall through to `face_offset_payload_occt(shape)?` or `face_offset_basis_geometry_occt(shape)?`.
- Keep `SingleFaceOffsetResult` attachment limited to exactly-one-face `SurfaceKind::Offset` results.
- Keep `MultiFaceOffsetResult` attachment validated; do not blindly attach every inventory entry to every generated offset face.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M18.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`
- Add the new ambiguous multi-source offset regression command here once named.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`
- `awk '/fn attach_multi_face_offset_metadata/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs | rg -n 'ported_offset_surface_from_metadata|with_offset_surface_face_metadata|offset_value: -candidate\.offset_value'`
- `awk '/pub\(crate\) fn ported_offset_surface_with_geometry/,/let payload = self.face_offset_payload_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'offset_surface_face_metadata'`
- `git diff --check`
