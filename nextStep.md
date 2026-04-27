# Next Task

Current milestone: `V1. Authored Analytic Shape Family Ownership Matrix`.

## Strategy Baseline

- Read `RUST_PORT_STRATEGY.md`, `portingMilestones.md`, and this file before editing.
- OCCT should be treated as constructor backend, normalized snapshot producer, and oracle. Automatic OCCT query fallback is debt unless it is explicitly unsupported/imported/raw.
- Do not choose the next task by scanning for the nearest `_occt()` call. Choose a supported authored shape family and move a vertical behavior row toward Rust ownership.
- Do not spend turns translating placeholder `occt_port` package files unless the same turn lands tested behavior in the exercised Rust kernel slice.

## Last Completed Cut

V1 `mixed analytic solid assembly` row is complete on 2026-04-27. `ownership_matrix_workflows::mixed_analytic_solid_assembly_authored_family_row_is_rust_owned` promotes the row in the authored-family ownership matrix and proves retained Rust compound/compsolid metadata for assemblies made from supported box, cylinder, cone, sphere, and torus solids; child analytic source inventories; normalized topology/BRep snapshots with curved child faces; public analytic face payload queries; Rust-owned line/circle edge queries with torus seam `Other` curves kept explicitly unsupported; PortedBrep bbox/area/edge-length/child-volume summary metrics; selectors; document reports; and history inspection. The cut also fixed multi-face ported summary bboxes so mixed curved analytic assemblies union topological bounds with Rust ported face-surface bounds instead of falling back to mesh summaries, while keeping unsupported face-free edge roots such as helix wires from masquerading as Rust-owned endpoint-only PortedBrep bboxes. The previous `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, `direct offset`, `generated offset`, `simple shell/solid assembly`, and `simple face/wire assembly` rows remain green.

V1 mixed analytic solid assembly verification:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows mixed_analytic_solid_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows simple_shell_solid_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows simple_face_wire_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_root_compound_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_nested_root_compound_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_bounding_boxes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`

## Active V1 Cut

Start a `mixed swept/offset assembly` authored family row. The row should prove compounds assembled from multiple already-supported swept and offset families are Rust-owned across construction/source metadata, normalized snapshot/BRep data, public swept/offset queries, summary metrics, selectors, documents, tests, and docs.

Bounded V1 mixed swept/offset assembly cut:

1. Add a mixed swept/offset assembly row to `ownership_matrix_workflows`, keeping the existing `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, `direct offset`, `generated offset`, `simple shell/solid assembly`, `simple face/wire assembly`, and `mixed analytic solid assembly` rows green.
2. Make the assembly row assert Rust-owned construction/source metadata for compounds assembled from supported prism/extrusion, revolution, direct offset, and generated offset families, normalized BRep/topology snapshots for swept and offset faces, public swept/offset face and basis query behavior, Rust-owned summary bbox/area/edge-length/zero-or-child-volume metrics, selectors, and document inspection.
3. Fill any missing metadata/snapshot/BRep/query paths needed to make that row green without automatic OCCT query fallback.
4. Keep explicit OCCT raw APIs only as oracle or unsupported/imported paths.
5. Update `portingMilestones.md` and this file with the next family row after the mixed swept/offset assembly row is green.

Verification for the V1 mixed swept/offset assembly row:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows mixed_swept_offset_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows public_offset_basis_queries_match_occt -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
