# Next Task

Current milestone: `M29. Rust-Owned Root Compound and CompSolid Topology Inventory` from `portingMilestones.md`.

## Completed Evidence

- `M28. Rust-Owned Root Solid Topology Inventory` is complete, and `M29` now has an exercised root-Compound direct-solid branch.
- `load_root_topology_snapshot()` now checks `root_assembly_topology_inventory_required()` after the completed root-solid branch and before the generic raw root-inventory sweep.
- Narrow root-assembly C ABI seeds now expose root `Compound` and root `CompSolid` typed subshape handles, with crate-private Rust wrappers in `Context`.
- `load_root_assembly_topology_snapshot()` clones the root assembly, reads solid/face/vertex/edge/wire/shell inventories through the root-assembly wrappers, derives vertex positions through `root_vertex_point_seed_occt()`, builds edge topology from Rust-owned edge geometry/endpoints/length, prepares wires and faces through the shared root-wire/root-face machinery, and prepares shell-local face/edge/vertex handles through the completed root-shell seeds.
- Loaded topology now carries child `solid_shapes`, and public `subshapes(Solid)` / `subshape(Solid, i)` for a loaded assembly root are served from the Rust-owned topology inventory instead of a fresh raw root sweep.
- Assembly summary volume now sums loaded child solid summaries for multi-solid compound/compsolid roots, avoiding signed face-contribution cancellation for disjoint solids.
- The assembly branch does not call the raw root-level `subshapes_occt()`, `vertex_point_occt()`, or `topology_occt()` APIs inside `load_root_assembly_topology_snapshot()`.
- `ported_brep_uses_rust_owned_topology_for_root_compound_solids` exercises a disjoint-box boolean result whose root is a `Compound`, verifies public solid/face/wire/edge/vertex handles against OCCT oracles, checks each child solid through Rust-owned topology, checks BRep topology and edge length parity, and verifies summary area/volume parity.
- Verification passed: `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, `(cd rust/lean_occt && cargo test ported_brep_uses_rust_owned_topology_for_root_compound_solids -- --nocapture)`, `(cd rust/lean_occt && cargo test)`, root-assembly positive/negative source guards, dispatch-order guards, C ABI symbol guards, and `git diff --check`.

## Target

Finish the remaining exercised compound/compsolid side of `load_root_topology_snapshot()`:

`subshapes_occt(shape, Vertex/Edge/Wire/Face/Shell)` for a supported root `Compound` or `CompSolid`

The direct-solid root `Compound` case now derives root-wide topology through the root-assembly branch. The remaining M29 cut should add an exercised `CompSolid` or direct-shell `Compound` fixture, tighten unsupported assembly escape paths, and keep root-wide topology aggregation ahead of the generic raw inventory loader.

## Next Bounded Cut

1. Find or add an exercised `CompSolid` or direct-shell `Compound` fixture that currently reaches the generic raw root inventory path.
2. Extend the root-assembly branch only as needed for that fixture, reusing the existing root-assembly C ABI wrappers and completed child topology preparation paths.
3. If a guarded assembly root cannot be represented by the narrow branch, return `None` rather than falling through to the generic raw root inventory path for that root.
4. Strengthen workflow coverage so the new assembly root checks public subshape handles, child shell/solid handles, BRep topology, summary counts, and summary area/volume parity.
5. Add source guards for the new fixture proving the root-assembly branch stays on root-assembly wrappers and avoids raw root `subshapes_occt()`, `vertex_point_occt()`, and `topology_occt()`.

## Guardrails

- Read `portingMilestones.md` and `nextStep.md` at the start of the next turn before editing.
- Do not weaken or reorder the completed M23 root-edge, M25 root-vertex, M24 root-wire, M26 root-face, M27 root-shell, or M28 root-solid branches.
- Keep explicit `*_occt()` helpers available as oracle APIs for tests and unsupported/imported shapes.
- Do not broaden the generic raw root inventory fallback while completing the remaining M29 assembly fixture.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- Focused workflow regression for the exercised root assembly fallback.
- `(cd rust/lean_occt && cargo test)`
- Add the named root-assembly topology source guards after implementing the branch.
- `git diff --check`
