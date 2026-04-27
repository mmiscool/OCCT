# Next Task

Current milestone: `V1. Authored Analytic Shape Family Ownership Matrix`.

## Strategy Baseline

- Read `RUST_PORT_STRATEGY.md`, `portingMilestones.md`, and this file before editing.
- OCCT should be treated as constructor backend, normalized snapshot producer, and oracle. Automatic OCCT query fallback is debt unless it is explicitly unsupported/imported/raw.
- Do not choose the next task by scanning for the nearest `_occt()` call. Choose a supported authored shape family and move a vertical behavior row toward Rust ownership.
- Do not spend turns translating placeholder `occt_port` package files unless the same turn lands tested behavior in the exercised Rust kernel slice.

## Last Completed Cut

V1 `box/planar` row is complete on 2026-04-27. `ownership_matrix_workflows::box_planar_authored_family_row_is_rust_owned` adds the compact authored-family ownership matrix and proves the `box/planar` row through Rust construction metadata, normalized topology/BRep snapshots, public plane payload queries with OCCT only as an explicit oracle, exact bbox/surface-area/volume/edge-length summaries, selectors, document descriptors, reports, and history inspection.

V1 box/planar verification:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows box_planar_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_box_plane_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`

## Active V1 Cut

Start the `cylinder` authored family row. The row should prove the supported cylinder side/cap family is Rust-owned across construction metadata, normalized snapshot/BRep data, public queries, summary metrics, selectors, documents, tests, and docs.

Bounded V1 cylinder cut:

1. Promote the `cylinder` row in `ownership_matrix_workflows` from pending to tested behavior, keeping the existing `box/planar` row green.
2. Make the cylinder row assert Rust-owned construction metadata for the side face and planar caps, cylinder/plane payload descriptors, normalized BRep/topology snapshots, summary bbox/area/edge-length metrics, public cylinder and cap-plane payload queries, selectors, and document inspection.
3. Fill any missing metadata/snapshot/BRep/query paths needed to make that row green without automatic OCCT query fallback.
4. Keep explicit OCCT raw APIs only as oracle or unsupported/imported paths.
5. Update `portingMilestones.md` and this file with the next family row after the `cylinder` row is green.

Verification for the V1 cylinder row:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows cylinder_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_cylinder_faces_use_rust_analytic_seed_metadata -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_surface_and_volume_formulas -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
