use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    BoxParams, BrepEdge, BrepFace, BrepShape, ConeParams, CurveKind, CylinderParams,
    CylindricalHoleParams, EdgeGeometry, EllipseEdgeParams, Error, FaceGeometry, FaceSample,
    FilletParams, HelixParams, LoopRole, ModelKernel, OffsetParams, Orientation, PortedCurve,
    PortedFaceSurface, PortedSurface, PrismParams, RevolutionParams, Shape, ShapeKind, ShapeReport,
    ShapeSummary, SphereParams, SurfaceKind, ThroughHoleCut, TopologySnapshot, TorusParams,
};

#[derive(Clone, Copy, Debug)]
pub struct EdgeDescriptor {
    pub index: usize,
    pub geometry: EdgeGeometry,
    pub ported_curve: Option<PortedCurve>,
    pub length: f64,
    pub start_vertex: Option<usize>,
    pub end_vertex: Option<usize>,
    pub start_point: Option<[f64; 3]>,
    pub end_point: Option<[f64; 3]>,
    pub adjacent_face_count: usize,
}

#[derive(Clone, Copy, Debug)]
pub struct FaceDescriptor {
    pub index: usize,
    pub geometry: FaceGeometry,
    pub ported_surface: Option<PortedSurface>,
    pub ported_face_surface: Option<PortedFaceSurface>,
    pub area: f64,
    pub wire_count: usize,
    pub orientation: Orientation,
    pub sample: FaceSample,
    pub outer_wire_index: Option<usize>,
    pub inner_wire_count: usize,
    pub adjacent_face_count: usize,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum EdgeSelector {
    FirstByCurveKind(CurveKind),
    LongestByCurveKind(CurveKind),
    ShortestByCurveKind(CurveKind),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum FaceSelector {
    FirstBySurfaceKind(SurfaceKind),
    LargestBySurfaceKind(SurfaceKind),
    BestAlignedPlane { normal_hint: [f64; 3] },
}

#[derive(Clone, Debug)]
pub enum OperationRecord {
    AddBox {
        output: String,
        params: BoxParams,
    },
    AddCylinder {
        output: String,
        params: CylinderParams,
    },
    AddCone {
        output: String,
        params: ConeParams,
    },
    AddSphere {
        output: String,
        params: SphereParams,
    },
    AddTorus {
        output: String,
        params: TorusParams,
    },
    AddEllipseEdge {
        output: String,
        params: EllipseEdgeParams,
    },
    AddHelix {
        output: String,
        params: HelixParams,
    },
    BoxWithThroughHole {
        output: String,
        spec: ThroughHoleCut,
    },
    Cut {
        output: String,
        lhs: String,
        rhs: String,
    },
    Fuse {
        output: String,
        lhs: String,
        rhs: String,
    },
    Common {
        output: String,
        lhs: String,
        rhs: String,
    },
    Compound {
        output: String,
        inputs: Vec<String>,
    },
    CompSolid {
        output: String,
        inputs: Vec<String>,
    },
    Subshape {
        output: String,
        input: String,
        kind: ShapeKind,
        index: usize,
    },
    Fillet {
        output: String,
        input: String,
        params: FilletParams,
    },
    Offset {
        output: String,
        input: String,
        params: OffsetParams,
    },
    DirectOffsetSurfaceFace {
        output: String,
        input: String,
        selector: FaceSelector,
        params: OffsetParams,
    },
    CylindricalHole {
        output: String,
        input: String,
        params: CylindricalHoleParams,
    },
    Prism {
        output: String,
        input: String,
        params: PrismParams,
    },
    Revolution {
        output: String,
        input: String,
        params: RevolutionParams,
    },
    ImportStep {
        output: String,
        path: PathBuf,
    },
    ExportStep {
        input: String,
        path: PathBuf,
    },
    StepRoundTrip {
        output: String,
        input: String,
    },
}

struct ShapeEntry {
    name: String,
    shape: Shape,
}

pub struct ModelDocument {
    kernel: ModelKernel,
    shapes: Vec<ShapeEntry>,
    history: Vec<OperationRecord>,
}

impl ModelDocument {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            kernel: ModelKernel::new()?,
            shapes: Vec::new(),
            history: Vec::new(),
        })
    }

    pub fn kernel(&self) -> &ModelKernel {
        &self.kernel
    }

    pub fn history(&self) -> &[OperationRecord] {
        &self.history
    }

    pub fn shape_names(&self) -> impl Iterator<Item = &str> {
        self.shapes.iter().map(|entry| entry.name.as_str())
    }

    pub fn contains_shape(&self, name: &str) -> bool {
        self.position_of(name).is_some()
    }

    pub fn shape(&self, name: impl AsRef<str>) -> Result<&Shape, Error> {
        self.shape_ref(name.as_ref())
    }

    pub fn insert_box(
        &mut self,
        output: impl Into<String>,
        params: BoxParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_box(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddBox { output, params });
        Ok(())
    }

    pub fn insert_cylinder(
        &mut self,
        output: impl Into<String>,
        params: CylinderParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_cylinder(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddCylinder { output, params });
        Ok(())
    }

    pub fn insert_cone(
        &mut self,
        output: impl Into<String>,
        params: ConeParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_cone(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddCone { output, params });
        Ok(())
    }

    pub fn insert_sphere(
        &mut self,
        output: impl Into<String>,
        params: SphereParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_sphere(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddSphere { output, params });
        Ok(())
    }

    pub fn insert_torus(
        &mut self,
        output: impl Into<String>,
        params: TorusParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_torus(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddTorus { output, params });
        Ok(())
    }

    pub fn insert_ellipse_edge(
        &mut self,
        output: impl Into<String>,
        params: EllipseEdgeParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_ellipse_edge(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddEllipseEdge { output, params });
        Ok(())
    }

    pub fn insert_helix(
        &mut self,
        output: impl Into<String>,
        params: HelixParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.make_helix(params)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::AddHelix { output, params });
        Ok(())
    }

    pub fn box_with_through_hole(
        &mut self,
        output: impl Into<String>,
        spec: ThroughHoleCut,
    ) -> Result<(), Error> {
        let output = output.into();
        let shape = self.kernel.box_with_through_hole(spec)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::BoxWithThroughHole { output, spec });
        Ok(())
    }

    pub fn cut(
        &mut self,
        output: impl Into<String>,
        lhs: impl AsRef<str>,
        rhs: impl AsRef<str>,
    ) -> Result<(), Error> {
        self.binary_shape_op(
            output,
            lhs,
            rhs,
            |kernel, lhs_shape, rhs_shape| kernel.cut(lhs_shape, rhs_shape),
            |output, lhs, rhs| OperationRecord::Cut { output, lhs, rhs },
        )
    }

    pub fn fuse(
        &mut self,
        output: impl Into<String>,
        lhs: impl AsRef<str>,
        rhs: impl AsRef<str>,
    ) -> Result<(), Error> {
        self.binary_shape_op(
            output,
            lhs,
            rhs,
            |kernel, lhs_shape, rhs_shape| kernel.fuse(lhs_shape, rhs_shape),
            |output, lhs, rhs| OperationRecord::Fuse { output, lhs, rhs },
        )
    }

    pub fn common(
        &mut self,
        output: impl Into<String>,
        lhs: impl AsRef<str>,
        rhs: impl AsRef<str>,
    ) -> Result<(), Error> {
        self.binary_shape_op(
            output,
            lhs,
            rhs,
            |kernel, lhs_shape, rhs_shape| kernel.common(lhs_shape, rhs_shape),
            |output, lhs, rhs| OperationRecord::Common { output, lhs, rhs },
        )
    }

    pub fn compound<S: AsRef<str>>(
        &mut self,
        output: impl Into<String>,
        inputs: &[S],
    ) -> Result<(), Error> {
        let output = output.into();
        let inputs = assembly_input_names("compound", inputs)?;
        let result = {
            let shape_refs = self.shape_refs(&inputs)?;
            self.kernel.context().make_compound_refs(&shape_refs)?
        };
        self.store_shape(output.clone(), result);
        self.history
            .push(OperationRecord::Compound { output, inputs });
        Ok(())
    }

    pub fn compsolid<S: AsRef<str>>(
        &mut self,
        output: impl Into<String>,
        inputs: &[S],
    ) -> Result<(), Error> {
        let output = output.into();
        let inputs = assembly_input_names("compsolid", inputs)?;
        let result = {
            let shape_refs = self.shape_refs(&inputs)?;
            self.kernel.context().make_compsolid_refs(&shape_refs)?
        };
        self.store_shape(output.clone(), result);
        self.history
            .push(OperationRecord::CompSolid { output, inputs });
        Ok(())
    }

    pub fn subshape(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        kind: ShapeKind,
        index: usize,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.context().subshape(input_shape, kind, index)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::Subshape {
            output,
            input,
            kind,
            index,
        });
        Ok(())
    }

    pub fn fillet(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        params: FilletParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.make_fillet(input_shape, params)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::Fillet {
            output,
            input,
            params,
        });
        Ok(())
    }

    pub fn offset(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        params: OffsetParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.make_offset(input_shape, params)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::Offset {
            output,
            input,
            params,
        });
        Ok(())
    }

    pub fn direct_offset_surface_face(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        selector: FaceSelector,
        params: OffsetParams,
    ) -> Result<FaceDescriptor, Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let selected_face = self.select_face(&input, selector)?;
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            let face_shapes = self
                .kernel
                .context()
                .subshapes(input_shape, ShapeKind::Face)?;
            let basis_face = face_shapes.get(selected_face.index).ok_or_else(|| {
                Error::new(format!(
                    "selected face index {} is unavailable for shape '{input}'",
                    selected_face.index
                ))
            })?;
            self.kernel.make_offset_surface_face(basis_face, params)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::DirectOffsetSurfaceFace {
            output,
            input,
            selector,
            params,
        });
        Ok(selected_face)
    }

    pub fn cylindrical_hole(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        params: CylindricalHoleParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.make_cylindrical_hole(input_shape, params)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::CylindricalHole {
            output,
            input,
            params,
        });
        Ok(())
    }

    pub fn prism(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        params: PrismParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.make_prism(input_shape, params)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::Prism {
            output,
            input,
            params,
        });
        Ok(())
    }

    pub fn revolution(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        params: RevolutionParams,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.make_revolution(input_shape, params)?
        };
        self.store_shape(output.clone(), shape);
        self.history.push(OperationRecord::Revolution {
            output,
            input,
            params,
        });
        Ok(())
    }

    pub fn import_step(
        &mut self,
        output: impl Into<String>,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let output = output.into();
        let path = path.as_ref().to_path_buf();
        let shape = self.kernel.read_step(&path)?;
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::ImportStep { output, path });
        Ok(())
    }

    pub fn export_step(
        &mut self,
        input: impl AsRef<str>,
        path: impl AsRef<Path>,
    ) -> Result<(), Error> {
        let input = input.as_ref().to_owned();
        let path = path.as_ref().to_path_buf();
        {
            let shape = self.shape_ref(&input)?;
            self.kernel.write_step(shape, &path)?;
        }
        self.history
            .push(OperationRecord::ExportStep { input, path });
        Ok(())
    }

    pub fn step_round_trip(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
    ) -> Result<(), Error> {
        let output = output.into();
        let input = input.as_ref().to_owned();
        let shape = {
            let input_shape = self.shape_ref(&input)?;
            self.kernel.step_round_trip_temp(input_shape)?
        };
        self.store_shape(output.clone(), shape);
        self.history
            .push(OperationRecord::StepRoundTrip { output, input });
        Ok(())
    }

    pub fn report(&self, name: impl AsRef<str>) -> Result<ShapeReport, Error> {
        self.kernel.inspect(self.shape_ref(name.as_ref())?)
    }

    pub fn summary(&self, name: impl AsRef<str>) -> Result<ShapeSummary, Error> {
        self.kernel.summarize(self.shape_ref(name.as_ref())?)
    }

    pub fn topology(&self, name: impl AsRef<str>) -> Result<TopologySnapshot, Error> {
        self.kernel.topology(self.shape_ref(name.as_ref())?)
    }

    pub fn brep(&self, name: impl AsRef<str>) -> Result<BrepShape, Error> {
        self.kernel.brep(self.shape_ref(name.as_ref())?)
    }

    pub fn edges(&self, name: impl AsRef<str>) -> Result<Vec<EdgeDescriptor>, Error> {
        Ok(self
            .brep(name)?
            .edges
            .iter()
            .map(edge_descriptor_from_brep)
            .collect())
    }

    pub fn faces(&self, name: impl AsRef<str>) -> Result<Vec<FaceDescriptor>, Error> {
        Ok(self
            .brep(name)?
            .faces
            .iter()
            .map(face_descriptor_from_brep)
            .collect())
    }

    pub fn edge_indices_by_curve_kind(
        &self,
        name: impl AsRef<str>,
        kind: CurveKind,
    ) -> Result<Vec<usize>, Error> {
        Ok(self
            .edges(name)?
            .into_iter()
            .filter(|edge| edge.geometry.kind == kind)
            .map(|edge| edge.index)
            .collect())
    }

    pub fn face_indices_by_surface_kind(
        &self,
        name: impl AsRef<str>,
        kind: SurfaceKind,
    ) -> Result<Vec<usize>, Error> {
        Ok(self
            .faces(name)?
            .into_iter()
            .filter(|face| face.geometry.kind == kind)
            .map(|face| face.index)
            .collect())
    }

    pub fn select_edge(
        &self,
        name: impl AsRef<str>,
        selector: EdgeSelector,
    ) -> Result<EdgeDescriptor, Error> {
        let shape_name = name.as_ref().to_owned();
        let brep = self.brep(&shape_name)?;
        let mut edges = brep.edges.iter();
        match selector {
            EdgeSelector::FirstByCurveKind(kind) => edges
                .find(|edge| edge.geometry.kind == kind)
                .map(edge_descriptor_from_brep)
                .ok_or_else(|| Error::new(format!("shape '{shape_name}' has no {:?} edge", kind))),
            EdgeSelector::LongestByCurveKind(kind) => edges
                .filter(|edge| edge.geometry.kind == kind)
                .max_by(|lhs, rhs| compare_edge_length(lhs, rhs))
                .map(edge_descriptor_from_brep)
                .ok_or_else(|| Error::new(format!("shape '{shape_name}' has no {:?} edge", kind))),
            EdgeSelector::ShortestByCurveKind(kind) => edges
                .filter(|edge| edge.geometry.kind == kind)
                .min_by(|lhs, rhs| compare_edge_length(lhs, rhs))
                .map(edge_descriptor_from_brep)
                .ok_or_else(|| Error::new(format!("shape '{shape_name}' has no {:?} edge", kind))),
        }
    }

    pub fn select_face(
        &self,
        name: impl AsRef<str>,
        selector: FaceSelector,
    ) -> Result<FaceDescriptor, Error> {
        let shape_name = name.as_ref().to_owned();
        let brep = self.brep(&shape_name)?;
        let mut faces = brep.faces.iter();
        match selector {
            FaceSelector::FirstBySurfaceKind(kind) => faces
                .find(|face| face.geometry.kind == kind)
                .map(face_descriptor_from_brep)
                .ok_or_else(|| Error::new(format!("shape '{shape_name}' has no {:?} face", kind))),
            FaceSelector::LargestBySurfaceKind(kind) => faces
                .filter(|face| face.geometry.kind == kind)
                .max_by(|lhs, rhs| compare_face_area(lhs, rhs))
                .map(face_descriptor_from_brep)
                .ok_or_else(|| Error::new(format!("shape '{shape_name}' has no {:?} face", kind))),
            FaceSelector::BestAlignedPlane { normal_hint } => {
                let desired = normalize_vector(normal_hint)?;
                faces
                    .filter(|face| face.geometry.kind == SurfaceKind::Plane)
                    .max_by(|lhs, rhs| {
                        compare_dot_alignment(lhs.sample.normal, rhs.sample.normal, desired)
                    })
                    .map(face_descriptor_from_brep)
                    .ok_or_else(|| Error::new(format!("shape '{shape_name}' has no planar faces")))
            }
        }
    }

    pub fn fillet_selected_edge(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        selector: EdgeSelector,
        radius: f64,
    ) -> Result<EdgeDescriptor, Error> {
        let input = input.as_ref().to_owned();
        let edge = self.select_edge(&input, selector)?;
        self.fillet(
            output,
            &input,
            FilletParams {
                radius,
                edge_index: u32::try_from(edge.index)
                    .map_err(|_| Error::new("edge index exceeded u32 range"))?,
            },
        )?;
        Ok(edge)
    }

    pub fn cylindrical_hole_on_selected_face(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        selector: FaceSelector,
        radius: f64,
    ) -> Result<FaceDescriptor, Error> {
        let input = input.as_ref().to_owned();
        let face = self.select_face(&input, selector)?;
        if face.geometry.kind != SurfaceKind::Plane {
            return Err(Error::new(format!(
                "selected face for shape '{input}' must be planar, got {:?}",
                face.geometry.kind
            )));
        }
        self.cylindrical_hole(
            output,
            &input,
            CylindricalHoleParams {
                origin: face.sample.position,
                axis: face.sample.normal,
                radius,
            },
        )?;
        Ok(face)
    }

    pub fn fillet_first_edge_by_curve_kind(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        kind: CurveKind,
        radius: f64,
    ) -> Result<EdgeDescriptor, Error> {
        self.fillet_selected_edge(output, input, EdgeSelector::FirstByCurveKind(kind), radius)
    }

    pub fn cylindrical_hole_from_best_aligned_planar_face(
        &mut self,
        output: impl Into<String>,
        input: impl AsRef<str>,
        normal_hint: [f64; 3],
        radius: f64,
    ) -> Result<FaceDescriptor, Error> {
        self.cylindrical_hole_on_selected_face(
            output,
            input,
            FaceSelector::BestAlignedPlane { normal_hint },
            radius,
        )
    }

    fn binary_shape_op<F, H>(
        &mut self,
        output: impl Into<String>,
        lhs: impl AsRef<str>,
        rhs: impl AsRef<str>,
        operation: F,
        history: H,
    ) -> Result<(), Error>
    where
        F: FnOnce(&ModelKernel, &Shape, &Shape) -> Result<Shape, Error>,
        H: FnOnce(String, String, String) -> OperationRecord,
    {
        let output = output.into();
        let lhs = lhs.as_ref().to_owned();
        let rhs = rhs.as_ref().to_owned();
        let result = {
            let (lhs_shape, rhs_shape) = self.shape_pair(&lhs, &rhs)?;
            operation(&self.kernel, lhs_shape, rhs_shape)?
        };
        self.store_shape(output.clone(), result);
        self.history.push(history(output, lhs, rhs));
        Ok(())
    }

    fn position_of(&self, name: &str) -> Option<usize> {
        self.shapes.iter().position(|entry| entry.name == name)
    }

    fn shape_ref(&self, name: &str) -> Result<&Shape, Error> {
        let index = self
            .position_of(name)
            .ok_or_else(|| Error::new(format!("unknown shape '{name}'")))?;
        Ok(&self.shapes[index].shape)
    }

    fn shape_refs<'a>(&'a self, names: &[String]) -> Result<Vec<&'a Shape>, Error> {
        names
            .iter()
            .map(|name| self.shape_ref(name))
            .collect::<Result<Vec<_>, _>>()
    }

    fn shape_pair(&self, lhs: &str, rhs: &str) -> Result<(&Shape, &Shape), Error> {
        let lhs_index = self
            .position_of(lhs)
            .ok_or_else(|| Error::new(format!("unknown shape '{lhs}'")))?;
        let rhs_index = self
            .position_of(rhs)
            .ok_or_else(|| Error::new(format!("unknown shape '{rhs}'")))?;

        if lhs_index == rhs_index {
            return Err(Error::new(format!(
                "binary operation requires distinct shape inputs, got '{lhs}' twice"
            )));
        }

        if lhs_index < rhs_index {
            let (left, right) = self.shapes.split_at(rhs_index);
            Ok((&left[lhs_index].shape, &right[0].shape))
        } else {
            let (left, right) = self.shapes.split_at(lhs_index);
            Ok((&right[0].shape, &left[rhs_index].shape))
        }
    }

    fn store_shape(&mut self, name: String, shape: Shape) {
        if let Some(index) = self.position_of(&name) {
            self.shapes[index] = ShapeEntry { name, shape };
        } else {
            self.shapes.push(ShapeEntry { name, shape });
        }
    }
}

fn assembly_input_names<S: AsRef<str>>(
    operation: &str,
    inputs: &[S],
) -> Result<Vec<String>, Error> {
    if inputs.is_empty() {
        return Err(Error::new(format!(
            "{operation} operation requires at least one input shape"
        )));
    }
    Ok(inputs
        .iter()
        .map(|input| input.as_ref().to_owned())
        .collect())
}

fn dot(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn edge_descriptor_from_brep(edge: &BrepEdge) -> EdgeDescriptor {
    EdgeDescriptor {
        index: edge.index,
        geometry: edge.geometry,
        ported_curve: edge.ported_curve,
        length: edge.length,
        start_vertex: edge.start_vertex,
        end_vertex: edge.end_vertex,
        start_point: edge.start_point,
        end_point: edge.end_point,
        adjacent_face_count: edge.adjacent_face_indices.len(),
    }
}

fn face_descriptor_from_brep(face: &BrepFace) -> FaceDescriptor {
    FaceDescriptor {
        index: face.index,
        geometry: face.geometry,
        ported_surface: face.ported_surface,
        ported_face_surface: face.ported_face_surface,
        area: face.area,
        wire_count: face.loops.len(),
        orientation: face.orientation,
        sample: face.sample,
        outer_wire_index: face
            .loops
            .iter()
            .find(|face_loop| face_loop.role == LoopRole::Outer)
            .map(|face_loop| face_loop.wire_index),
        inner_wire_count: face
            .loops
            .iter()
            .filter(|face_loop| face_loop.role == LoopRole::Inner)
            .count(),
        adjacent_face_count: face.adjacent_face_indices.len(),
    }
}

fn compare_edge_length(lhs: &BrepEdge, rhs: &BrepEdge) -> std::cmp::Ordering {
    lhs.length
        .partial_cmp(&rhs.length)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn compare_face_area(lhs: &BrepFace, rhs: &BrepFace) -> std::cmp::Ordering {
    lhs.area
        .partial_cmp(&rhs.area)
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn compare_dot_alignment(
    lhs_normal: [f64; 3],
    rhs_normal: [f64; 3],
    desired: [f64; 3],
) -> std::cmp::Ordering {
    dot(lhs_normal, desired)
        .partial_cmp(&dot(rhs_normal, desired))
        .unwrap_or(std::cmp::Ordering::Equal)
}

fn normalize_vector(vector: [f64; 3]) -> Result<[f64; 3], Error> {
    let length = dot(vector, vector).sqrt();
    if length <= 1.0e-12 {
        return Err(Error::new("vector must be non-zero"));
    }
    Ok([vector[0] / length, vector[1] / length, vector[2] / length])
}
