# Next Task

Split the remaining mesh and math helper tail of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` into one or more sibling modules.

## Focus

- Extract the polyhedral mesh helpers (`polyhedral_mesh_volume`, `polyhedral_mesh_area`, `polyhedral_mesh_sample`, `mesh_bbox`, `bbox_from_points`, `union_bbox`) into a dedicated module.
- Extract the shared vector/scalar math helpers (`add3`, `subtract3`, `scale3`, `dot3`, `cross3`, `normalize3`, `norm3`, `approx_eq`) into a small utility module if that keeps the mesh module cleaner.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` and `cargo test --manifest-path rust/lean_occt/Cargo.toml --test brep_workflows` passing after the move.

## Why This Is Next

The swept-face and topology helper clusters are now out of `brep.rs`, but the file still ends with a dense block of mesh reduction and vector math that is shared across the new sibling modules. Moving that tail next keeps the OCCT-port BRep layout consistent before more translated helpers land here.
