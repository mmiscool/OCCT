# Next Task

Extract the planar multi-wire loop-role assignment from the face snapshot packing path.

## Focus

- Pull the multi-wire outer/inner role selection logic out of `append_ported_face_topology()` in `face_snapshot.rs` so the helper reads as topology matching plus accumulation, not topology matching plus area-based role classification.
- Keep the current planar wire area computation, outer-loop selection rule, and error behavior unchanged.
- Keep the current face validation, root-wire matching, planar loop classification, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the planar-face gate now centralized, the heaviest remaining local logic in the face snapshot stage is the area-based outer/inner loop classification embedded inside `append_ported_face_topology()`. Pulling that into its own helper is the next bounded cleanup.
