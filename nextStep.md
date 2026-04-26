# Next Task

Current milestone: `M27. Rust-Owned Root Shell Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M26. Rust-Owned Root Face Topology Inventory` is complete.
- `load_root_topology_snapshot()` now calls `load_root_face_topology_snapshot()` for supported single root faces after the completed root edge, root vertex, and face-free root-wire branches and before the generic raw root-inventory sweep.
- The root-face branch clones the root face handle, reads ordered face-local wire, edge, and vertex handles through narrow C ABI seeds, derives vertex positions through `root_vertex_point_seed_occt()`, builds edge topology from Rust-owned edge geometry/endpoints/length, and packs face wires through the existing ordered wire-occurrence path.
- The branch prepares the single face without calling `subshapes_occt()`, `vertex_point_occt()`, or `topology_occt()` inside `load_root_face_topology_snapshot()`.
- `ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes` now checks public face/wire/edge/vertex counts and handles, topology handles, edge lengths, vertex points, and empty shell inventory for exercised root faces.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI -j 8`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, root-face positive/negative source guards, C ABI symbol guards, and `git diff --check`.

## Target

Replace the root-shell side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` for a supported root `Shell`

Root shells should derive their shell handle, shell-local face handles, face/wire/edge inventory, and vertex positions from a root-shell-specific branch before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Add a narrow shell-face C ABI seed, exposed through a crate-private Rust wrapper, for ordered root-shell face handles.
2. Add `load_root_shell_topology_snapshot()` after the completed root edge, root vertex, face-free root-wire, and root-face branches, but before the generic root loader.
3. Guard it to supported `ShapeKind::Shell` roots with one shell and no solid or compound inventory.
4. Seed the branch with a cloned root shell handle and shell-local face handles; do not use the generic root `subshapes_occt(shape, ...)` sweep.
5. Reuse the M26 root-face preparation machinery for each supported shell face so face-local wires, edges, and vertices still come from narrow seeds and ordered wire occurrences.
6. Assemble unique face, wire, edge, and vertex inventories, then prepare the single shell from those faces so BRep materialization can build shell ranges without `topology_occt()`.
7. Strengthen the multi-face shell/solid topology/BRep regression so it checks public shell, face, wire, edge, and vertex subshape handles for a root shell.
8. Add source guards proving the root-shell branch uses the narrow shell-face seed plus face/wire occurrence seeds and avoids root-level `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23 root-edge, M25 root-vertex, M24 root-wire, or M26 root-face branches.
- Keep solids, compounds, and ambiguous shell-bearing topology on the existing generic path until their own milestones.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- If a root shell cannot be represented by the narrow branch, return `None` instead of entering the generic raw root inventory path for that root.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI -j 8`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the named root-shell topology source guards after implementing the branch.
- `git diff --check`
