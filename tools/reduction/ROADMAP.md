# Lean OCCT Reduction Roadmap

The repository now contains the physically reduced OCCT tree. The next passes should
keep the modeling/BREP core intact and continue trimming around the direct STEP path.

## Current Boundary

Validated by `LeanExchangeSmoke`:

- primitive authoring with `BRepPrimAPI`
- boolean cut with `BRepAlgoAPI`
- representative fillet, offset, cylindrical-hole feature, and helix generation coverage
- tessellation with `BRepMesh_IncrementalMesh`
- STEP read/write with `STEPControl`

## Next Reduction Targets

### Data Exchange Narrowing

- retain direct STEP only
- keep IGES, glTF, VRML, OBJ, STL, XML/Bin XCAF, and related providers out
- continue trimming helper/provider glue from the STEP side when it does not affect direct BREP exchange
- test whether parts of `TKDE` can be collapsed further without breaking `STEPControl` round-trip

### Optional Non-BREP Toolkits

- keep `TKHLR`, `TKXMesh`, and `TKExpress` out unless a concrete retained workflow proves they are required
- treat any reintroduction of these toolkits as an exception that needs an explicit smoke case

### Modeling Boundary Discipline

- keep the full retained authoring stack intact: `TKGeomAlgo`, `TKTopAlgo`, `TKPrim`, `TKBO`, `TKBool`, `TKFillet`, `TKOffset`, `TKFeat`, `TKHelix`, `TKMesh`, `TKShHealing`
- do not hollow out boolean or authoring capability just to shrink file count
- only cut inside retained modeling toolkits after broader regression coverage exists

## Working Rule

For every proposed removal:

1. make one package, toolkit, or layer cut
2. rebuild `LeanExchangeSmoke`
3. rebuild the wasm demo if the cut touches meshing or topology traversal
4. keep the cut only if both still pass

This keeps the reduction effort aligned with the eventual Rust port boundary without accidentally
shrinking the supported BREP authoring feature set.
