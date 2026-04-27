# Next Task

Current milestone: `V1. Authored Analytic Shape Family Ownership Matrix`.

## Strategy Baseline

- Read `RUST_PORT_STRATEGY.md`, `portingMilestones.md`, and this file before editing.
- OCCT should be treated as constructor backend, normalized snapshot producer, and oracle. Automatic OCCT query fallback is debt unless it is explicitly unsupported/imported/raw.
- Do not choose the next task by scanning for the nearest `_occt()` call. Choose a supported authored shape family and move a vertical behavior row toward Rust ownership.
- Do not spend turns translating placeholder `occt_port` package files unless the same turn lands tested behavior in the exercised Rust kernel slice.

## Last Completed Cut

V1 `simple face/wire assembly` row is complete on 2026-04-27. `ownership_matrix_workflows::simple_face_wire_assembly_authored_family_row_is_rust_owned` promotes the row in the authored-family ownership matrix and proves retained Rust assembly/source metadata for compound construction from supported faces, wires, edges, and vertices, normalized topology/BRep snapshots, public plane/line/vertex queries, Rust-owned summary bbox/area/edge-length/zero-volume source behavior, selectors, descriptors, named document subshape extraction, document compound construction, reports where mesh-backed reports are supported, and history inspection. The cut also fixed Rust-owned boundary bbox summaries so isolated vertex children are unioned with analytic edge bounds in mixed assemblies. The previous `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, `direct offset`, `generated offset`, and `simple shell/solid assembly` rows remain green.

V1 simple face/wire assembly verification:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows simple_face_wire_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows simple_shell_solid_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_root_compound_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_nested_root_compound_face_free_wires -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_root_compound_edges_and_vertices -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `git diff --check`

## Active V1 Cut

Start a `mixed analytic solid assembly` authored family row. The row should prove compounds and compsolids assembled from multiple already-supported analytic solid families are Rust-owned across construction/source metadata, normalized snapshot/BRep data, public analytic queries, summary metrics, selectors, documents, tests, and docs.

Bounded V1 mixed analytic solid assembly cut:

1. Add a mixed analytic solid assembly row to `ownership_matrix_workflows`, keeping the existing `box/planar`, `cylinder`, `cone`, `sphere`, `torus`, `prism/extrusion`, `revolution`, `direct offset`, `generated offset`, `simple shell/solid assembly`, and `simple face/wire assembly` rows green.
2. Make the assembly row assert Rust-owned construction/source metadata for compounds/compsolids assembled from several supported analytic solid families, normalized BRep/topology snapshots for curved child faces, public analytic face/edge query behavior, Rust-owned summary bbox/area/edge-length/child-solid volume metrics, selectors, and document inspection.
3. Fill any missing metadata/snapshot/BRep/query paths needed to make that row green without automatic OCCT query fallback.
4. Keep explicit OCCT raw APIs only as oracle or unsupported/imported paths.
5. Update `portingMilestones.md` and this file with the next family row after the mixed analytic solid assembly row is green.

Verification for the V1 mixed analytic solid assembly row:

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows mixed_analytic_solid_assembly_authored_family_row_is_rust_owned -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ownership_matrix_workflows -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_root_compound_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_nested_root_compound_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test document_workflows -- --nocapture`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
