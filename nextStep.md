# Next Task

Re-enable analytic face reuse in `ported_brep_summary_faces` by removing the remaining Raw-only dependency in analytic face volume, starting with planar faces.

## Current State

- `summary.rs` now gates Rust analytic and mesh whole-shape volume on closed topology. Open or non-manifold solids fall back to OCCT volume instead of taking a bogus Rust mesh volume.
- `ported_brep_summary_faces` in [`face_surface.rs`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_surface.rs) now reuses Public faces for:
  - `PortedFaceSurface::Offset`
  - `PortedFaceSurface::Swept`
- Analytic faces are still forced back onto the `Raw` route.

## Remaining Blocker

The unstable boundary is the plane shortcut in [`analytic_face_volume`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/face_metrics.rs):

```rust
if matches!(surface, PortedSurface::Plane(_)) {
    return Some(face.area * dot3(face.sample.position, face.sample.normal) / 3.0);
}
```

That shortcut depends directly on `face.area` and `face.sample`. On the Raw route those values are still stable enough for summary derivation. On the Public route, holed planar faces can drift enough to change the analytic volume result, which is why analytic summary faces are still rebuilt on `Raw`.

## Focus

1. Replace the planar special-case in `analytic_face_volume` with a Rust-owned computation that derives the plane contribution from the loops/geometry path, not from `face.area` and `face.sample`.
2. Keep the new implementation on the same Rust-first boundaries already used elsewhere in this file:
   - prefer `PortedCurve::from_context_with_ported_payloads()`
   - prefer public face/edge geometry and payload routes
   - only fall back to Raw/OCCT where the existing helper paths already do so
3. Once planar analytic volume is stable on the Public route, flip the analytic reuse arm in `ported_brep_summary_faces` from:
   ```rust
   Some(PortedFaceSurface::Analytic(_)) => true
   ```
   to Public reuse for the safe analytic subset, or all analytic faces if parity holds.
4. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

The swept and offset summary reuse boundary is now open. The next meaningful Rust-first move is to retire the remaining Raw-only analytic summary face split, and the plane-volume shortcut is the concrete function currently blocking that.
