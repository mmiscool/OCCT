# Next Task

Current milestone: `M28. Rust-Owned Root Solid Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M27. Rust-Owned Root Shell Topology Inventory` is complete.
- `load_root_topology_snapshot()` now calls `load_root_shell_topology_snapshot()` for supported root shells after the completed root edge, root vertex, face-free root-wire, and root-face branches and before the generic raw root-inventory sweep.
- The root-shell branch clones the root shell handle, reads shell-local face/wire/edge/vertex handles through narrow C ABI seeds, derives vertex positions through `root_vertex_point_seed_occt()`, builds edge topology from Rust-owned edge geometry/endpoints/length, and packs shell wires through ordered wire occurrences.
- Shell faces are prepared through the shared root-face machinery, so face-local wires still come from `root_face_wire_shapes_occt()` and each wire still uses `wire_edge_occurrences_occt()`.
- The branch prepares the single shell without calling `subshapes_occt()`, `vertex_point_occt()`, or `topology_occt()` inside `load_root_shell_topology_snapshot()`.
- `ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids` now pulls each solid-derived shell as a root shell and checks public shell/face/wire/edge/vertex handles, isolated topology handles, edge lengths, vertex points, and BRep topology parity.
- Verification passed: `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`, `cmake --build build --target LeanOcctCAPI -j 8`, `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`, `cargo test --manifest-path rust/lean_occt/Cargo.toml`, root-shell positive/negative source guards, helper face/wire occurrence guards, C ABI symbol guards, and `git diff --check`.

## Target

Replace the root-solid side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` for a supported root `Solid`

Root solids should derive their solid-local shell handles, shell/face/wire/edge inventory, and vertex positions from a root-solid-specific branch before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Add narrow solid-local C ABI seeds for shell, face, wire, edge, and vertex handles, exposed through crate-private Rust wrappers.
2. Add `load_root_solid_topology_snapshot()` after the completed root edge, root vertex, face-free root-wire, root-face, and root-shell branches, but before the generic root loader.
3. Guard it to supported `ShapeKind::Solid` roots with one solid, one or more shells, and no compound or compsolid inventory.
4. Seed the branch with a cloned root solid only as the root discriminator, then derive solid-local shell handles through the narrow solid-shell seed; do not use the generic root `subshapes_occt(shape, ...)` sweep.
5. Reuse the M27 root-shell preparation approach for each supported solid shell so shell-local faces, face-local wires, edges, and vertices still come from narrow seeds and ordered wire occurrences.
6. Assemble unique solid-wide face, wire, edge, and vertex inventories, then prepare shell ranges from shell-local face handles so BRep materialization and summary/report consumers can build without `topology_occt()`.
7. Strengthen the multi-face solid topology/BRep regression so the root solid itself checks public shell, face, wire, edge, and vertex subshape handles against the Rust-owned inventory.
8. Add source guards proving the root-solid branch uses the narrow solid-local seeds plus shell/face/wire occurrence preparation and avoids root-level `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23 root-edge, M25 root-vertex, M24 root-wire, M26 root-face, or M27 root-shell branches.
- Keep compounds, compsolids, imported solids, and ambiguous solid-bearing topology on explicit raw/oracle escape paths until their own milestones.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- If a guarded root solid cannot be represented by the narrow branch, return `None` instead of entering the generic raw root inventory path for that root.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI -j 8`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test ported_geometry_workflows`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- Add the named root-solid topology source guards after implementing the branch.
- `git diff --check`
