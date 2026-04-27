# Next Task

Current milestone: `V1. Authored Analytic Shape Family Ownership Matrix`.

## Strategy Baseline

- Read `RUST_PORT_STRATEGY.md`, `portingMilestones.md`, and this file before editing.
- OCCT should be treated as constructor backend, normalized snapshot producer, and oracle. Automatic OCCT query fallback is debt unless it is explicitly unsupported/imported/raw.
- Do not choose the next task by scanning for the nearest `_occt()` call. Choose a supported authored shape family and move a vertical behavior row toward Rust ownership.
- Do not spend turns translating placeholder `occt_port` package files unless the same turn lands tested behavior in the exercised Rust kernel slice.

## Last Completed Cut

M50 is complete on 2026-04-27. The supported single-face surface bbox path now resolves reconstructed and degenerate cap contributions from loaded `BrepFace` geometry plus sampled `PortedFaceSurface` descriptors. The direct OCCT surface, pcurve, and edge-curve bbox helpers remain only behind explicitly named unsupported/raw helpers, supported descriptor rows fail closed instead of entering raw fallback, and multi-face offset shell bbox resolution can use retained offset metadata when root faces do not carry offset descriptors.

M50 verification:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows single_face_surface_bbox_keeps_supported_faces_on_rust_descriptors -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_bounding_boxes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`

## Active V1 Cut

Start the `box/planar` authored family row. The row should prove the supported box/planar family is Rust-owned across construction metadata, normalized snapshot/BRep data, public queries, summary metrics, selectors, documents, tests, and docs.

Bounded V1 cut:

1. Add a compact ownership matrix in tests or a small crate-internal support module for Rust-authored families: box/plane, cylinder, cone, sphere, torus, prism/extrusion, revolution, direct offset, and generated offset.
2. Make the `box/planar` row assert Rust-owned construction metadata, planar face payload descriptors, normalized BRep/topology snapshots, summary bbox/area/edge-length metrics, public plane payload queries, selectors, and document inspection.
3. Fill any missing metadata/snapshot/BRep/query paths needed to make that row green without automatic OCCT query fallback.
4. Keep explicit OCCT raw APIs only as oracle or unsupported/imported paths.
5. Update `portingMilestones.md` and this file with the next family row after the `box/planar` row is green.

Verification for the V1 box/planar row:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_box_plane_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
