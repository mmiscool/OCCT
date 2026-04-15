# Next Task

Tighten the face snapshot entry path now that the internal per-face carrier has been removed.

## Focus

- Collapse the remaining `load_ported_face_snapshot_shapes()` preload step into the face snapshot entry if it no longer adds signal.
- Re-evaluate whether `validate_ported_face_snapshot()` should stay separate after that collapse, while keeping `TopologySnapshotFaceFields` as the stage output boundary back to `topology.rs`.
- Keep the current face validation, root-wire matching, planar loop classification, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With `PortedFaceTopology` gone, the remaining face-stage scaffolding is mostly the entry/load split before packing. The next cleanup is trimming that two-step boundary so the stage reads as one face-owned load/validate/pack flow.
