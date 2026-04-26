# Next Task

Current milestone: `M29. Rust-Owned Root Compound and CompSolid Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M28. Rust-Owned Root Solid Topology Inventory` is complete.
- `load_root_topology_snapshot()` now calls `load_root_solid_topology_snapshot()` for supported root solids after the completed root edge, root vertex, face-free root-wire, root-face, and root-shell branches and before the generic raw root-inventory sweep.
- Narrow solid-local C ABI seeds now expose root-solid shell, face, wire, edge, and vertex handles, with crate-private Rust wrappers in `Context`.
- The root-solid branch clones the root solid handle, reads solid-local inventories through those narrow seeds, derives vertex positions through `root_vertex_point_seed_occt()`, builds root edge topology from Rust-owned edge geometry/endpoints/length, and packs solid wires through ordered wire occurrences.
- Solid faces are prepared through the shared root-face machinery, so face-local wires still come from `root_face_wire_shapes_occt()` and each wire still uses `wire_edge_occurrences_occt()`.
- Solid shell handles are prepared from solid-local shell seeds, with shell-local face/edge/vertex handles retained for public shell subshape queries and shell summary/report consumers.
- The branch does not call `subshapes_occt()`, `vertex_point_occt()`, or `topology_occt()` inside `load_root_solid_topology_snapshot()`.
- `ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids` now checks the root solid itself through public shell/face/wire/edge/vertex handles, isolated shell/face/wire topology handles, edge lengths, vertex points, and BRep topology parity.
- Verification passed: `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, `(cd rust/lean_occt && cargo test ported_brep_uses_rust_owned_topology_for_simple_multi_face_solids --test brep_workflows -- --nocapture)`, `(cd rust/lean_occt && cargo test --test brep_workflows -- --nocapture)`, `(cd rust/lean_occt && cargo test --lib)`, `(cd rust/lean_occt && cargo test)`, root-solid positive/negative source guards, shared face/wire occurrence guards, dispatch-order guards, C ABI symbol guards, and `git diff --check`.

## Target

Replace the next exercised compound/compsolid side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` for a supported root `Compound` or `CompSolid`

Root assemblies should derive direct child handles through root-assembly-specific seeds, reuse completed child topology loaders for supported solids/shells/faces, and aggregate root-wide topology before the generic raw inventory loader is considered.

## Next Bounded Cut

1. Identify the exercised root `Compound` or `CompSolid` fallback first, preferably a direct-child assembly whose children are supported solids or shells.
2. Add narrow direct-child C ABI seeds for the active assembly root and expose them through crate-private Rust wrappers.
3. Add a root-assembly topology branch after the completed root edge, root vertex, face-free root-wire, root-face, root-shell, and root-solid branches, but before the generic root loader.
4. Guard it to the supported assembly shape family found in step 1; keep nested, unsupported, or ambiguous assemblies on explicit raw/oracle escape paths.
5. Reuse the completed root solid/shell/face/wire preparation paths per direct child, then aggregate unique root-wide shell/face/wire/edge/vertex inventories without the root-level `subshapes_occt(shape, ...)` sweep.
6. Strengthen a workflow regression so the assembly root itself checks public subshape handles against the Rust-owned aggregate inventory and BRep topology.
7. Add source guards proving the root-assembly branch uses narrow direct-child seeds and avoids root-level `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23 root-edge, M25 root-vertex, M24 root-wire, M26 root-face, M27 root-shell, or M28 root-solid branches.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- If a guarded assembly root cannot be represented by the narrow branch, return `None` instead of entering the generic raw root inventory path for that root.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- Focused workflow regression for the exercised root assembly fallback.
- `(cd rust/lean_occt && cargo test)`
- Add the named root-assembly topology source guards after implementing the branch.
- `git diff --check`
