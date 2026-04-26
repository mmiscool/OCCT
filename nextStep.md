# Next Task

Current milestone: `M12. Rust-Owned Planar Face Snapshot Payload` from `portingMilestones.md`.

## Completed Evidence

- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs::single_face_topology_snapshot()` now loads through `load_ported_topology()` and returns `None` when Rust topology cannot be loaded instead of falling back to `context.topology_occt(face_shape)?`.
- `face_topology.rs::single_face_topology_with_route()` now uses the loaded ported edge-shape inventory and no longer enumerates face edges with `context.subshapes_occt(face_shape, ShapeKind::Edge)?`.
- `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs::load_ported_face_snapshot()` now has Rust-owned zero-wire and single-root-wire single-face snapshot reconstruction, covering boundary-free offset-extrusion and supported single-face roots without raw topology rescue.
- `rust/lean_occt/tests/brep_workflows.rs::assert_single_face_ported_area_matches_brep()` requires single-face BRep area to match public ported face area, and `rust/lean_occt/tests/ported_geometry_workflows.rs::ported_face_surface_descriptors_cover_supported_faces()` now asserts that offset-extrusion exercises the Rust zero-wire topology path.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `! rg -n 'None => context\.topology_occt\(face_shape\)\?|subshapes_occt\(face_shape, ShapeKind::Edge\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_topology.rs`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, and `git diff --check`.

## Target

Remove or strictly narrow the direct planar face snapshot payload fallback at `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs::ported_snapshot_plane_payload()`, which still calls `context.face_geometry_occt(face_shape)?` before using Rust-owned ported surface payload extraction.

## Next Bounded Cut

1. Start in `face_snapshot.rs::ported_snapshot_plane_payload()`.
2. Replace the direct `context.face_geometry_occt(face_shape)?` plane discriminator with public/Rust-owned `Context::face_geometry(face_shape)?` plus `PortedSurface::from_context_with_ported_payloads()`.
3. Preserve explicit `Ok(None)` behavior for non-plane, unsupported, or ambiguous face snapshots without letting supported planar snapshot construction silently bypass the M8/M9 Rust face query guard.
4. Strengthen holed planar and simple single-face coverage if the stricter path exposes a gap, then keep a grep guard against `face_geometry_occt(face_shape)` in `face_snapshot.rs`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not reintroduce `None => context.topology_occt(face_shape)?` or `context.subshapes_occt(face_shape, ShapeKind::Edge)?` into `face_topology.rs`.
- Do not reintroduce direct OCCT helper fallbacks into public payload, geometry, sampling, topology, vertex, supported subshape, BRep materialization, or topology-construction wrappers narrowed under M7 through M11.
- Keep the remaining raw wire topology snapshot fallback explicitly limited to duplicate-edge-occurrence wires or occurrence reconstruction misses until the wire inventory can represent repeated edge occurrences.
- Do not reintroduce `context.edge_geometry_occt(edge_shape)?` or `context.edge_endpoints_occt(edge_shape)?` into `edge_topology.rs::root_edge_topology()` or `wire_topology.rs::wire_occurrence()`.
- Do not reintroduce `face_bboxes_occt()`, `OffsetFaceBboxSource::OcctFaceUnion`, `offset_shape_bbox_occt()`, or `SummaryBboxSource::OffsetOcctSubshapeUnion`.
- Do not weaken `unsupported_bbox_summary_fallback_allowed()` or `unsupported_volume_summary_fallback_allowed()`.
- Keep `ModelDocument::edges()`, `ModelDocument::faces()`, `select_edge()`, and `select_face()` centered on `BrepShape`.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `! rg -n 'face_geometry_occt\(face_shape\)' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_snapshot.rs`
- `git diff --check`
