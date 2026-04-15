# Next Task

Split the remaining helper-heavy tail of `rust/lean_occt/src/occt_port/ModelingData/TKBRep/BRepTools/brep.rs` into smaller sibling modules.

## Focus

- Extract the swept-face helper cluster into its own module.
- Extract the topology/navigation helpers into a separate module.
- Leave behavior unchanged and keep `cargo check --manifest-path rust/lean_occt/Cargo.toml` passing after each move.

## Why This Is Next

`brep.rs` is no longer one giant file for face descriptors, summary logic, and face metrics, but it still contains a large block of mixed low-level helpers. Splitting that tail next keeps the OCCT port layout disciplined before more translated logic gets added.
