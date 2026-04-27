# Next Task

Current milestone: `V1. Authored Analytic Shape Family Ownership Matrix`.

## Strategy Baseline

- Read `RUST_PORT_STRATEGY.md`, `portingMilestones.md`, and this file before editing.
- OCCT should be treated as constructor backend, normalized snapshot producer, and oracle. Automatic OCCT query fallback is debt unless it is explicitly unsupported/imported/raw.
- Do not choose the next task by scanning for the nearest `_occt()` call. Choose a supported authored shape family and move a vertical behavior row toward Rust ownership.
- Do not spend turns translating placeholder `occt_port` package files unless the same turn lands tested behavior in the exercised Rust kernel slice.

## Last Completed Cut

V1 `direct offset` row is complete on 2026-04-27. `ownership_matrix_workflows::direct_offset_authored_family_row_is_rust_owned` promotes the direct-offset row in the authored-family ownership matrix and proves a document-selected box face direct offset through Rust construction metadata, retained offset and basis descriptors, boundary-free normalized topology/BRep snapshots, public offset payload and offset-basis payload queries with OCCT only as an explicit oracle, Rust normalized face sampling parity, ported bbox/area/edge-length summaries, explicit zero volume for the non-solid face result, selectors, document descriptors, and history inspection. The previous `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, and `revolution` rows remain green.

V1 direct offset verification:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows direct_offset_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`

## Active V1 Cut

Start the `generated offset` authored family row. The row should prove the supported generated offset family is Rust-owned across construction metadata, normalized snapshot/BRep data, public queries, summary metrics, selectors, documents, tests, and docs.

Bounded V1 generated offset cut:

1. Promote the `generated offset` row in `ownership_matrix_workflows` from pending to tested behavior, keeping the existing `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, and `direct offset` rows green.
2. Make the generated offset row assert Rust-owned construction metadata for generated offset outputs, offset and offset-basis payload descriptors, normalized BRep/topology snapshots, exact or explicitly Rust-owned summary metrics, public offset payload queries, selectors, and document inspection.
3. Fill any missing metadata/snapshot/BRep/query paths needed to make that row green without automatic OCCT query fallback.
4. Keep explicit OCCT raw APIs only as oracle or unsupported/imported paths.
5. Update `portingMilestones.md` and this file with the next family row after the `generated offset` row is green.

Verification for the V1 generated offset row:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows generated_offset_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_offset_surface_sampling_matches_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows document_runs_analytic_shape_pipeline -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test selector_workflows selectors_choose_expected_faces_and_edges -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
