# Next Task

Current milestone: `M39. Rust-Owned Strict BRep Root Edge Support Gate` from `portingMilestones.md`.

## Completed Evidence

- `M38. Close Remaining Public Edge Geometry Raw Classifier` is complete.
- `Context::ported_edge_geometry()` now contains only the M37 root-edge topology branch followed by `Ok(None)`.
- The removed post-root branch no longer calls `edge_geometry_occt()`, `edge_endpoints()`, raw line/circle/ellipse payload reconstruction, or shape length as a ported edge geometry classifier.
- The now-unused generic raw-domain edge-geometry reconstruction helpers were removed from `ported_geometry/payloads.rs`.
- Supported root line/circle/ellipse public geometry still comes from `brep::ported_root_edge_geometry()`.
- Unsupported helix root edges still return `Ok(None)` from `ported_edge_geometry()` and continue to resolve through explicit public/raw `edge_geometry()` fallback.
- Non-edge inputs now have regression coverage proving `ported_edge_geometry()` returns `Ok(None)` instead of entering a raw edge classifier, while public `edge_geometry()` still rejects the invalid query through the explicit raw API.
- Source guard passed for the whole `ported_edge_geometry()` body: no `edge_geometry_occt(` call remains.
- Verification passed with `cargo check`, targeted geometry tests, `ported_geometry_workflows`, `brep_workflows`, `cmake --build build --target LeanOcctCAPI`, full `cargo test`, and `git diff --check`.

## Target

Remove the next raw edge geometry support classifier:

`strict_brep_root_edge_requires_ported_topology() -> edge_geometry_occt()` in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs`.

After M36/M37/M38, root edge support classification is already owned by the Rust root topology inventory and `ported_root_edge_geometry()`. The strict BRep gate should use that Rust-owned classification instead of direct raw edge geometry.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Replace the `edge_geometry_occt()` call in `strict_brep_root_edge_requires_ported_topology()` with a root topology inventory or `ported_root_edge_geometry()` classifier.
3. Preserve the supported requirement for root line/circle/ellipse BRep materialization.
4. Preserve unsupported helix/root-edge behavior as explicit raw/oracle-only, not strict supported topology.
5. Strengthen or add regression coverage around supported root edge strict materialization and unsupported helix exclusion.
6. Add a source guard proving the strict root-edge gate contains no `edge_geometry_occt(` call.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not reintroduce direct `edge_geometry_occt()` into `ported_edge_geometry()` or root-edge topology bootstrap.
- Do not call public `edge_geometry()` from the strict BRep root-edge support gate; use the Rust-owned root topology/geometry inventory.
- Keep explicit `Context::edge_geometry_occt()` available as the raw oracle API.
- Keep unsupported root edge kinds out of the strict supported-edge requirement unless a Rust-owned replacement is landed in the same turn.
- Preserve root edge topology, BRep materialization, public geometry, endpoint, payload, sample, and raw-oracle parity behavior.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test brep_workflows supported_brep_materialization_requires_ported_topology -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_topology_for_face_free_shapes -- --nocapture)`
- `! perl -0ne 'print $1 if /(fn strict_brep_root_edge_requires_ported_topology[\s\S]*?)\nfn strict_brep_face_inventory_requires_ported_topology/' rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs | rg -n 'edge_geometry_occt\('`
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
