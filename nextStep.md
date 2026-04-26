# Next Task

Current milestone: `M20. Rust-Owned Wire Occurrence Topology for Repeated Edge Wires` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs::attach_multi_face_offset_metadata()` now replaces the `match_count == 1` ambiguous multi-source escape hatch with deterministic scoring across validated signed candidates. It reconstructs generated source-basis positions from offset samples and compares them to the retained Rust source-basis descriptor, then attaches the unique best `OffsetSurfaceFaceMetadata`.
- Invalid signed source candidates are treated as non-matches, matching the old rejection behavior, while genuinely no-match or tied-best cases remain explicit fallbacks instead of silently picking arbitrary metadata.
- `rust/lean_occt/src/lib.rs::Shape` now has hidden integration-test probes for Rust offset metadata attachment state and multi-source inventory size, so tests can prove generated handles are metadata-backed before public offset payload queries can use raw helpers.
- `rust/lean_occt/tests/brep_workflows.rs::ported_brep_maps_multi_source_swept_offsets_in_rust` builds the swept `ellipse -> prism -> extrusion_face -> revolved -> make_offset(&revolved)` fixture, verifies the result carries four Rust source metadata candidates, asserts all four generated `SurfaceKind::Offset` root faces have attached Rust metadata, and repeats the metadata assertion through shell-local face handles.
- The same regression checks public offset payloads, basis geometry, swept basis descriptors, and descriptor sampling parity against OCCT oracle samples for the mapped multi-source faces.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml && cargo check --manifest-path rust/lean_occt/Cargo.toml && cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, `! awk '/fn offset_surface_face_metadata_candidate/,/^    }/' rust/lean_occt/src/lib.rs | rg -n 'face_offset_payload_occt|face_offset_basis_geometry_occt'`, `awk '/fn attach_multi_face_offset_metadata/,/^}/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs | rg -n 'offset_metadata_match_score|with_offset_surface_face_metadata|offset_value: -candidate\.offset_value'`, `awk '/pub\(crate\) fn ported_offset_surface_with_geometry/,/let payload = self.face_offset_payload_occt/' rust/lean_occt/src/occt_port/ModelingData/TKG3d/GeomEval/ported_geometry.rs | rg -n 'offset_surface_face_metadata'`, and `git diff --check`.

## Target

Remove or strictly narrow `wire_topology.rs::root_wire_topology_from_snapshot()`, which still calls `context.topology_occt(&prepared_wire_shape.wire_shape)` when Rust occurrence ordering cannot build a wire. The next porting cut should move a repeated-edge or occurrence-order wire family to Rust-owned `RootWireTopology` construction instead of reading a raw OCCT topology snapshot.

## Next Bounded Cut

1. Identify the simplest exercised wire fixture where `root_wire_topology_from_occurrences()` returns `None` and the current code succeeds through `root_wire_topology_from_snapshot()`.
2. Extend the prepared wire occurrence data or matching logic so that fixture can order edge occurrences, orientations, and vertex occurrence order from Rust-owned edge geometry/endpoints/vertex positions.
3. Route the fixture through `root_wire_topology_from_occurrences()` and remove or narrow the snapshot fallback to unsupported/imported/degenerate wires only.
4. Add a focused regression for the formerly snapshot-backed wire and a source guard that prevents reintroducing `context.topology_occt(&prepared_wire_shape.wire_shape)` on the supported path.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M19.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Keep `SingleFaceOffsetResult`, `MultiFaceOffsetResult`, signed offset matching, and deterministic multi-source offset scoring intact.
- Do not let metadata-attached offset faces fall through to `face_offset_payload_occt(shape)?` or `face_offset_basis_geometry_occt(shape)?`.
- Keep `root_wire_topology_from_snapshot()` available only while the next cut proves and narrows the repeated-edge occurrence boundary.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- Add the focused wire-occurrence regression command here once named.
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add an `awk`/`rg` guard proving the supported wire path no longer calls `topology_occt(&prepared_wire_shape.wire_shape)`.
- `git diff --check`
