# Next Task

Keep narrowing the remaining shell-local OCCT bbox fallback in `offset_shell_bbox()`, but stay on the shell-boundary Rust path. The mixed-edge shell-boundary union now includes an adaptive sampled public-edge tier plus four public unsupported-edge extremum passes: bracketed axis-turning polish, a near-flat tangent-dip probe, a local axis-position extremum search, and a broader run-based seeded axis-position extremum search. The next task is refining the new score-driven biased probe placement inside the chosen side of an interval, so one-sided shell-edge extrema can trigger denser public sampling even when the current fixed side-local midpoint still misses them.

## Current State

- [`ported_shape_summary()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) now has these relevant offset bbox tiers:
  - non-solid offset shapes first try [`offset_faces_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), which now validates:
    - the shape-local Rust mesh bbox
    - an offset-distance-expanded shape-local Rust mesh bbox
    - the validated Rust face-BRep union
    - only then the per-face OCCT bbox union over loaded root `face_shapes`
  - non-solid offset shapes keep [`offset_shape_bbox_occt()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) behind that as a narrower raw escape hatch, and only accept the later shape-local Rust mesh tier when it validates against OCCT
  - offset solids and compsolids now use [`offset_solid_shell_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs), and each shell now tries:
    - a validated shell-local Rust face-BRep union built from [`validated_face_brep_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs)
    - a validated shell-local Rust boundary bbox built from loader-owned `shell_vertex_shapes` / `shell_edge_shapes`
    - a validated shell-local Rust mesh bbox
    - an offset-distance-expanded shell-local Rust mesh bbox
    - a validated shell-local Rust `ported_brep(shell).summary`
    - an offset-distance-expanded shell-local Rust `ported_brep(shell).summary`
    - only then the shell-local OCCT bbox
- [`load_ported_topology()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/topology.rs) now preserves `PreparedShellShape { shell_shape, shell_vertex_shapes, shell_edge_shapes, shell_face_shapes }` on the successful Rust-topology path.
- [`shell_boundary_shape_bbox()`](rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep/summary.rs) is now a mixed per-edge union:
  - it always starts from the loader-owned shell vertices
  - it unions any edge that already admits a Rust/public exact boundary bbox
  - unsupported shell edges no longer fail the whole shell-boundary candidate immediately
  - unsupported shell edges now also get a validated adaptive public-edge sampling chance before the later mesh/summary tiers
  - that sampled tier now recursively refines intervals when midpoint samples expand the interval bbox, when sampled tangents indicate an axis turn, when midpoint axis position departs materially from endpoint-linear interpolation, or when the midpoint bends materially away from the chord
  - if that midpoint-only gate declines, the same tier now also runs a small quarter-point probe-driven prepass and subdivides when any `(start, q1, midpoint)`, `(q1, midpoint, q3)`, or `(midpoint, q3, end)` triple shows the same missed-extremum signals
  - if those quarter probes still decline, the same tier now also runs a second outer-probe pass near the interval ends and subdivides when any `(start, o1, q1)`, `(o1, q1, midpoint)`, `(midpoint, q3, o3)`, or `(q3, o3, end)` triple shows the same missed-extremum signals
  - if those fixed outer probes still decline, the same tier now also scores the left and right side triples using the same bbox-expansion, tangent-turn, axis-shoulder, and chord-bend signals, then places one extra public probe on the stronger side before giving up on refinement for that interval
  - after refinement, adjacent sampled intervals now also get a public-edge tangent-root polish pass, so interior axis extrema with a bracketed tangent sign change can contribute directly to the shell boundary bbox
  - adjacent sampled intervals that still have no clean tangent sign bracket now also get a public-edge near-flat tangent-dip probe, so local interior extrema can still contribute when the sampled tangent magnitude drops sharply without a clean sign flip at the current interval endpoints
  - adjacent sampled triples now also get a public-edge local axis-position extremum search seeded from quadratic position fits, so unsupported edges can still contribute interior extrema when the decisive bbox driver is visible in sampled positions even though the public tangent-based solvers never produce a clean bracket or dip
  - those same sampled extrema candidates now also get a broader run-based seeded public axis-position search around the best interior sampled point per axis, so unsupported edges can still contribute bbox-driving extrema even when the decisive position bulge is spread across a shallow shoulder or plateau instead of a strict local sampled triple
- The exercised non-solid offset shell fixture stays green on the Rust-first path.
- The exercised closed offset solid fixture stays green, including the direct per-shell parity assertion in [`ported_brep_uses_rust_owned_volume_for_offset_solids()`](rust/lean_occt/tests/brep_workflows.rs).

## Remaining Blocker

`offset_shell_bbox()` still ends at the raw shell-local OCCT bbox for shells that fail all current validated Rust candidates. The new shell-boundary Rust candidate now covers:

- supported boundary edges already determine the shell bbox, or
- every shell edge that matters to the bbox admits a Rust/public exact boundary bbox, or
- the current adaptive public-edge sampling, interval refinement, and tangent-root polish hits the remaining shell-boundary extrema closely enough to validate.

The remaining blocker is shell edges whose decisive bbox extrema still evade the new score-driven side choice because the suspicious side is known but the current side-local probe still lands at a fixed midpoint inside that side. Even after the midpoint axis-shoulder refinement trigger, the quarter-point probe-driven refinement prepass, the outer-probe refinement pass, the new stronger-side biased probe, bracketed tangent-root polish, the near-flat tangent-dip search, the local axis-position extremum search, and the broader run-based seeded axis-position search, those shells still skip straight to the later mesh/summary candidates and eventually the raw shell-local OCCT bbox.

## Focus

1. Keep the non-solid offset bbox win in place.
2. Keep the now-green direct shell parity check for the exercised closed offset solid.
3. Stay on loader-owned shell-local inventories; do not reintroduce fresh raw `subshapes_occt()` traversal.
4. Keep the new shell boundary candidate on the public Rust edge/vertex path.
5. Keep the mixed shell-boundary union in place, and make unsupported shell-edge contributions more accurate rather than reverting to all-or-nothing boundary gating.
6. Validate every new shell candidate against the shell-local OCCT bbox before accepting it.
7. Keep the verification bar unchanged:
   - `cargo check --manifest-path rust/lean_occt/Cargo.toml`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows`
   - `cargo test --manifest-path rust/lean_occt/Cargo.toml`

## Why This Is Next

This turn moved more of the offset bbox path onto Rust-owned data without weakening parity:

- non-solid offset shapes now get an offset-expanded Rust mesh bbox validation chance before the raw face bbox union
- closed offset shells now carry shell-local edge and vertex inventories through `PreparedShellShape`
- closed offset shells now try a validated shell-local Rust boundary bbox before mesh and shell-summary validation
- that shell-boundary path now refines unsupported public-edge samples when midpoint, tangent, or chord-bend checks suggest missed extrema
- that same path now polishes bracketed axis-turning extrema with public `edge_sample()` bisection before falling through to later tiers
- intervals that still do not admit a clean tangent sign bracket now also get a public near-flat tangent-dip search before the shell falls through to later tiers
- sampled triples that already show a local axis bulge now also get a public position-based extremum search before the shell falls through to later tiers
- the same sampled boundary path now also broadens that position search across a monotone run around the best interior sampled point per axis, so more unsupported edges can contribute Rust-owned extrema even when their decisive shoulder is wider than the old fixed seeded window
- the refinement gate now also subdivides intervals when the midpoint axis position departs materially from endpoint-linear interpolation, so shallow one-axis shoulders can seed denser public samples even while staying inside the coarse interval bbox
- if the midpoint still does not justify subdivision, the refinement gate now also runs a public quarter-point probe prepass before giving up on the interval, so off-center interior bulges can still trigger the existing recursive sampling path
- if those quarter probes still decline, the refinement gate now also runs a second outer-probe pass near the interval ends, so shoulder extrema that sit outside the midpoint-plus-quarter triples can still trigger recursive public sampling before the later tangent and position solvers run
- if those fixed outer probes still decline, the refinement gate now also scores the left and right side triples with the same existing refinement signals and places one extra probe on the stronger side, so one-sided intervals can still trigger recursive public sampling before the later tangent and position solvers run

The next step is to make that shell-local Rust boundary path cover more real offset shells by refining where the score-driven extra probe lands inside the already-chosen strong side. The next bounded cut is choosing between the inner and outer half of that strong side from the same signal mix, not widening fallback elsewhere.
