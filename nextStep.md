# Next Task

Current milestone: `M48. Rust-Owned Swept Face Geometry Classifier Narrowing` from `portingMilestones.md`.

## Completed Evidence

- `M47. Rust-Owned Offset Classifier Fallback Narrowing` is complete.
- `Context::ported_face_geometry()` no longer has the post-raw `(SurfaceKind::Offset, Some(PortedFaceSurface::Offset(_)))` classifier arm.
- Offset faces now need retained metadata before the raw face-geometry read. `ported_offset_surface_from_metadata_face_geometry()` remains installed before `self.face_geometry_occt(shape)?`, validates retained metadata through `Context::ported_offset_surface_from_metadata()`, and returns offset `FaceGeometry` without re-entering the generic raw offset classifier.
- `offset_result_face_shapes()` attaches validated offset metadata to enumerated generated/direct offset face handles before strict BRep support checks and public subshape queries.
- Generated offset metadata records generated `FaceGeometry` after Rust offset descriptor validation, so generated UV bounds and periodic flags are available through the pre-raw metadata path.
- Direct offset-surface roots propagate retained metadata to their enumerated face handle, and `ported_brep_uses_rust_owned_area_for_offset_faces` now asserts that direct offset subshapes classify as offsets through retained metadata.
- BRep summary gained sampled swept/offset single-face surface bboxes, restored offset-face source precedence for non-solid offset roots, and isolates reconstructed non-offset cap extents from offset classification so offset-solid shell bboxes remain validated through the BRep path.
- Verification passed with `(cd rust/lean_occt && cargo fmt)`, `cmake --build build --target LeanOcctCAPI`, `(cd rust/lean_occt && cargo check)`, focused offset geometry/BRep regressions, source guards proving the raw offset classifier is absent and the pre-raw metadata path remains present, full `(cd rust/lean_occt && cargo test)`, and `git diff --check`.

## Target

Narrow the remaining swept classifier in `Context::ported_face_geometry()`.

After M47, this post-raw branch remains:

```rust
if matches!(
    raw_geometry.kind,
    SurfaceKind::Revolution | SurfaceKind::Extrusion
) {
    let descriptor = ported_face_surface_descriptor_value(self, shape, raw_geometry)?;
    return match (raw_geometry.kind, descriptor) {
        (
            SurfaceKind::Revolution,
            Some(PortedFaceSurface::Swept(PortedSweptSurface::Revolution { .. })),
        )
        | (
            SurfaceKind::Extrusion,
            Some(PortedFaceSurface::Swept(PortedSweptSurface::Extrusion { .. })),
        ) => Ok(Some(raw_geometry)),
        _ => Ok(None),
    };
}
```

That still uses direct raw face geometry as the support classifier for exercised revolution and extrusion faces.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Trace the exercised revolution/extrusion faces that still reach the post-raw swept classifier in `Context::ported_face_geometry()`.
3. Derive swept `FaceGeometry` from Rust-owned swept descriptors/topology before `self.face_geometry_occt(shape)?`.
4. Remove or strictly narrow the `SurfaceKind::Revolution | SurfaceKind::Extrusion` post-raw classifier only after swept geometry, swept payload, swept BRep summary, swept-offset, and offset-solid regressions stay green.
5. Strengthen coverage so an exercised swept face fails if it silently falls back through the raw swept classifier again.
6. Add a source guard proving the raw swept classifier is removed or explicitly conditioned on a documented unsupported/raw path.
7. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not call `face_geometry_occt()` before serving exercised Rust-owned swept face geometry.
- Do not classify metadata-free, imported, invalid, or unsupported swept-like faces as Rust-owned unless a Rust descriptor/topology path proves support.
- Preserve explicit raw/oracle face geometry, swept payload, offset payload, basis, and sampling APIs.
- Keep direct swept, swept BRep solid, swept-offset metadata, multi-source swept offset, and offset-solid volume regressions green.

## Verification

- `(cd rust/lean_occt && cargo fmt)`
- `cmake --build build --target LeanOcctCAPI`
- `(cd rust/lean_occt && cargo check)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_face_surface_descriptors_cover_supported_faces -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows public_swept_and_offset_payload_queries_match_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test ported_geometry_workflows ported_swept_surface_sampling_matches_occt -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_summarizes_swept_revolution_solids_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_maps_multi_source_swept_offsets_in_rust -- --nocapture)`
- `(cd rust/lean_occt && cargo test --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture)`
- A source guard proving the raw swept classifier is removed or explicitly narrowed to the documented unsupported path.
- `(cd rust/lean_occt && cargo test)`
- `git diff --check`
