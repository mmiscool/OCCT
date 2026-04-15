# Next Task

Deduplicate the multi-wire planar-face gating inside the face snapshot stage.

## Focus

- Reconcile the duplicated multi-wire plane checks currently split between `validate_ported_face_snapshot()` and `append_ported_face_topology()` in `face_snapshot.rs`.
- Keep `validate_ported_face_snapshot()` as the entry-level preflight if it still improves readability, but avoid re-encoding the same non-planar rejection rule deeper in the per-face packing path.
- Keep the current face validation, root-wire matching, planar loop classification, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the face entry now reading as one load/validate/pack flow, the main remaining local redundancy is the repeated multi-wire planar-face gate. Tightening that rule into one clear boundary is the next bounded cleanup in the same stage.
