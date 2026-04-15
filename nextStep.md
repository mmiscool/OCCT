# Next Task

Move the remaining face snapshot packer helpers onto the accumulator type.

## Focus

- Turn the new face snapshot accumulator in `face_snapshot.rs` into the clear owner of the remaining packer helpers, specifically the current `append_face_topology_outputs()` and final edge-face flattening path.
- Keep the extracted planar multi-wire setup, face-wire matching helper, accumulator-backed packing flow, planar wire area computation, and loop-role classification unchanged.
- Keep the current face range offsets, edge-face ordering, validation behavior, and packed snapshot output unchanged.
- Preserve the downstream `Context::ported_topology()` / `Context::ported_brep()` behavior and existing topology snapshot parity.
- Keep `cargo check --manifest-path rust/lean_occt/Cargo.toml`, `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`, and `cargo test --manifest-path rust/lean_occt/Cargo.toml` passing after the cleanup.

## Why This Is Next

With the accumulator now introduced, the remaining free packer helpers are still conceptually owned by that state. Moving them onto the accumulator should tighten module ownership without changing any snapshot behavior.
