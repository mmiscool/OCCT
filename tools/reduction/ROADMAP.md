# OCCT Reduction Roadmap

`LeanAuthoringExchange` is the first usable reduction boundary, not the final minimum.
Given the current goal, the next passes should keep the modeling/BREP core intact and cull
surrounding layers first.

## Current Boundary

Validated by `LeanExchangeSmoke`:

- primitive authoring with `BRepPrimAPI`
- boolean cut with `BRepAlgoAPI`
- representative fillet, offset, cylindrical-hole feature, and helix generation coverage
- tessellation with `BRepMesh_IncrementalMesh`
- STEP read/write with `STEPControl`

## Next Reduction Targets

### Visualization / Draw / Viewer Stack

- keep `BUILD_MODULE_Visualization=OFF`
- keep `BUILD_MODULE_Draw=OFF`
- continue removing viewer-specific scripts, launchers, and assets from the reduction branch
- keep wasm output limited to the explicit demo surface extraction path instead of OCCT viewers

### Application Framework / CAF / XCAF Layers

- keep `BUILD_MODULE_ApplicationFramework=OFF`
- keep XCAF document/material/color/assembly layers out of the retained subset
- keep STEP translation on the direct `STEPControl` path rather than CAF providers

### Data Exchange Narrowing

- retain direct STEP only
- keep IGES, glTF, VRML, OBJ, STL, XML/Bin XCAF, and related providers out
- continue trimming CAF/provider glue from the STEP side when it does not affect direct BREP exchange

### Optional Non-BREP Toolkits

- keep `TKHLR`, `TKXMesh`, and `TKExpress` out unless a concrete retained workflow proves they are required
- treat any reintroduction of these toolkits as an exception that needs an explicit smoke case

### Physical Subset Extraction

- once the build boundary is stable, extract the retained code into a dedicated subset tree or package
- preserve upstream file layout long enough to keep diffing against stock OCCT practical
- only split inside retained modeling toolkits after a broader regression suite exists for the full authoring surface

## Working Rule

For every proposed removal:

1. make one package, toolkit, or layer cut
2. rebuild `LeanExchangeSmoke`
3. rebuild the wasm demo if the cut touches meshing or topology traversal
4. keep the cut only if both still pass

This keeps the reduction effort aligned with the eventual Rust port boundary without accidentally
shrinking the supported BREP authoring feature set.
