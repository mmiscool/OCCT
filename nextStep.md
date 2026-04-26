# Next Task

Current milestone: `M50. Rust-Owned Single-Face Surface Bbox Fallback Narrowing` from `portingMilestones.md`.

## Completed Evidence

- M49 is complete. `Context::ported_face_geometry()` no longer lets metadata-bearing offset, swept, or analytic faces continue to `face_uv_bounds_occt(shape)` after Rust metadata validation returns `None`.
- Offset, swept, and analytic metadata checks now run before the raw UV-bounds seed and immediately return their Rust-owned helper result. Imported, unsupported, metadata-free, and ambiguous faces remain on the existing raw bounds path.
- The constructor metadata source guard now requires the offset/swept/analytic metadata classifiers before `face_uv_bounds_occt(shape)`, requires immediate helper returns, and blocks the old `if let Some(...)` fall-through-to-raw pattern.
- Focused metadata regressions stayed green for torus, sphere, cone, cylinder, box-plane, offset, swept, supported descriptor, and public payload paths; the full Rust suite and C ABI build also stayed green.

## Target

Move the next exercised bbox fallback family from direct OCCT helpers to Rust-owned BRep/ported-surface data.

`summary.rs::single_face_surface_bbox()` still calls `face_surface_bbox_occt()`, `face_pcurve_control_polygon_bbox_occt()`, and `edge_curve_bbox_occt()` from the reconstructed-cap and degenerate-plane single-face bbox paths. Those raw helpers should remain available only for imported BSpline/Bezier/unsupported faces, not for supported analytic, swept, or offset faces that already have `BrepFace` geometry and `PortedFaceSurface` descriptors.

## Next Bounded Cut

1. Read `portingMilestones.md` and `nextStep.md` before editing.
2. Replace the supported-face calls from `reconstructed_cap_surface_bbox()` and `degenerate_plane_cap_surface_bbox()` to `single_face_surface_bbox()` with a Rust-owned path that consumes the loaded `BrepFace`.
3. Expand `ported_face_surface_bbox()` or add an adjacent supported-face helper so exercised analytic, swept, and offset faces can derive their surface bbox contribution from Rust-owned geometry, boundary topology, mesh validation, or sampled `PortedFaceSurface` descriptors.
4. Keep direct `face_surface_bbox_occt()`, `face_pcurve_control_polygon_bbox_occt()`, and `edge_curve_bbox_occt()` only in an explicitly named unsupported/imported raw helper.
5. Strengthen BRep/source coverage so supported `PortedFaceSurface` branches cannot enter that raw helper, while imported/unsupported raw behavior remains available.
6. Update both control files with completed evidence, active milestone, next bounded cut, and exact verification commands.

## Guardrails

- Do not weaken the completed M49 metadata-before-raw-bounds guard in `ported_geometry.rs`.
- Do not remove explicit raw/oracle bbox APIs; narrow their automatic use from supported BRep summary paths.
- Preserve existing bbox source expectations for exact primitives, supported single-face BReps, offset face unions, offset-solid shell unions, and full-shape summaries.
- Do not replace one direct OCCT bbox helper with another direct OCCT helper for supported analytic, swept, or offset faces.
- Keep imported BSpline/Bezier and unsupported face families on an explicit raw path unless they are ported in the same cut.

## Verification

- `cargo fmt --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_topology_for_simple_single_face_shapes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_exact_primitive_bounding_boxes -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_area_for_offset_faces -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows ported_brep_uses_rust_owned_volume_for_offset_solids -- --nocapture`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
- `cargo check --manifest-path rust/lean_occt/Cargo.toml`
- `cargo test --manifest-path rust/lean_occt/Cargo.toml`
- `cmake --build build --target LeanOcctCAPI`
- `git diff --check`
