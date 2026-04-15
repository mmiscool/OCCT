# Next Task

Tighten the face snapshot stage now that the topology coordinator has been simplified.

## Focus

- Evaluate whether `TopologySnapshotFaceFields` and `PortedFaceTopology` in `face_snapshot.rs` still earn their keep as named carriers, or whether one of them should be destructured or inlined to leave a tighter stage boundary.
- Keep the current face validation, root-wire matching, planar loop classification, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the final `TopologySnapshot` assembly now inlined at the coordinator, the next obvious cleanup is the face-stage carrier layer. That stage is now the main place where temporary topology structs still accumulate before being packed, so it is the right next boundary to tighten.
