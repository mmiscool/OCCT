# Next Task

Retire the remaining summary-face split in `ported_brep()` so the successful Rust-topology path can hand its Public face inventory directly to `ported_shape_summary()`.

## Current State

- `analytic_face_volume` in [`face_metrics.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_metrics.rs) no longer uses the Raw-only planar shortcut based on `face.area` and `face.sample`.
- `ported_brep_summary_faces` in [`face_surface.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs) now reuses Public faces for:
  - `PortedFaceSurface::Analytic`
  - `PortedFaceSurface::Offset`
  - `PortedFaceSurface::Swept`
- The top-level BRep path in [`brep.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs) still builds a separate `summary_faces` inventory, even though the known Rust-owned face kinds are now already summary-safe on the Public route.

## Remaining Blocker

The leftover duplication is only there for faces where `ported_face_surface == None`. Those faces still get rebuilt on the `Raw` route before summary derivation, even though the successful Rust-topology branch already has a Public `faces` inventory in hand.

## Focus

1. Prove whether `ported_shape_summary()` can consume the existing Public `faces` inventory directly, including the `None` face-surface path.
2. If the unknown-face path is already summary-safe, remove `ported_brep_summary_faces()` and the extra `summary_faces` split from `ported_brep()`.
3. If it is not fully safe yet, narrow the fallback boundary as much as possible so the duplicated summary face rebuild is smaller and explicitly justified.
4. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

The analytic, offset, and swept summary-face split is gone. The next aggressive Rust-first step is to remove the remaining top-level summary-face duplication, so the Rust-topology BRep path stops rebuilding a second face inventory before summary generation.
