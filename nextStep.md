# Next Task

Finish trimming the summary-specific scaffolding that still lives in `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs`.

## Focus

- Move `ExactPrimitiveSummary` into `brep/summary.rs`, where every constructor and consumer already lives.
- Add a small helper for building `ShapeCounts` from topology plus the raw OCCT root counts, instead of open-coding the same struct assembly in multiple `Context` entry points.
- Replace the duplicated `ShapeCounts` setup in `ported_vertex_point()` and `ported_edge_endpoints()` with that helper.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the move.

## Why This Is Next

The helper-only carrier structs are now out of `brep.rs`, so the remaining private items there are mostly summary plumbing. Pulling `ExactPrimitiveSummary` into `summary.rs` and centralizing `ShapeCounts` construction keeps the parent module focused on public BRep types and `Context` entry points, which is the next clean boundary after the recent `topology`, `swept_face`, `mesh`, `math`, and helper-struct extractions.
