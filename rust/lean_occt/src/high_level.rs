use std::fs;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::{
    BoxParams, BrepShape, ConeParams, Context, CylinderParams, CylindricalHoleParams,
    EllipseEdgeParams, Error, FilletParams, HelixParams, Mesh, MeshParams, OffsetParams,
    PrismParams, RevolutionParams, Shape, ShapeSummary, SphereParams, TopologySnapshot,
    TorusParams,
};

static STEP_ROUND_TRIP_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ThroughHoleCut {
    pub box_params: BoxParams,
    pub tool_params: CylinderParams,
}

#[derive(Debug)]
pub struct ShapeReport {
    pub summary: ShapeSummary,
    pub mesh: Mesh,
    pub topology: TopologySnapshot,
}

impl ShapeReport {
    pub fn triangle_count(&self) -> usize {
        self.mesh.triangle_indices.len() / 3
    }

    pub fn edge_segment_count(&self) -> usize {
        self.mesh.edge_segments.len()
    }
}

pub struct ModelKernel {
    context: Context,
}

impl ModelKernel {
    pub fn new() -> Result<Self, Error> {
        Ok(Self {
            context: Context::new()?,
        })
    }

    pub fn context(&self) -> &Context {
        &self.context
    }

    pub fn make_box(&self, params: BoxParams) -> Result<Shape, Error> {
        self.context.make_box(params)
    }

    pub fn make_cylinder(&self, params: CylinderParams) -> Result<Shape, Error> {
        self.context.make_cylinder(params)
    }

    pub fn make_cone(&self, params: ConeParams) -> Result<Shape, Error> {
        self.context.make_cone(params)
    }

    pub fn make_sphere(&self, params: SphereParams) -> Result<Shape, Error> {
        self.context.make_sphere(params)
    }

    pub fn make_torus(&self, params: TorusParams) -> Result<Shape, Error> {
        self.context.make_torus(params)
    }

    pub fn make_ellipse_edge(&self, params: EllipseEdgeParams) -> Result<Shape, Error> {
        self.context.make_ellipse_edge(params)
    }

    pub fn make_fillet(&self, shape: &Shape, params: FilletParams) -> Result<Shape, Error> {
        self.context.make_fillet(shape, params)
    }

    pub fn make_offset(&self, shape: &Shape, params: OffsetParams) -> Result<Shape, Error> {
        self.context.make_offset(shape, params)
    }

    pub fn make_cylindrical_hole(
        &self,
        shape: &Shape,
        params: CylindricalHoleParams,
    ) -> Result<Shape, Error> {
        self.context.make_cylindrical_hole(shape, params)
    }

    pub fn make_helix(&self, params: HelixParams) -> Result<Shape, Error> {
        self.context.make_helix(params)
    }

    pub fn make_prism(&self, shape: &Shape, params: PrismParams) -> Result<Shape, Error> {
        self.context.make_prism(shape, params)
    }

    pub fn make_revolution(&self, shape: &Shape, params: RevolutionParams) -> Result<Shape, Error> {
        self.context.make_revolution(shape, params)
    }

    pub fn cut(&self, lhs: &Shape, rhs: &Shape) -> Result<Shape, Error> {
        self.context.cut(lhs, rhs)
    }

    pub fn fuse(&self, lhs: &Shape, rhs: &Shape) -> Result<Shape, Error> {
        self.context.fuse(lhs, rhs)
    }

    pub fn common(&self, lhs: &Shape, rhs: &Shape) -> Result<Shape, Error> {
        self.context.common(lhs, rhs)
    }

    pub fn box_with_through_hole(&self, spec: ThroughHoleCut) -> Result<Shape, Error> {
        let base = self.make_box(spec.box_params)?;
        let tool = self.make_cylinder(spec.tool_params)?;
        self.cut(&base, &tool)
    }

    pub fn inspect(&self, shape: &Shape) -> Result<ShapeReport, Error> {
        self.inspect_with_mesh(shape, MeshParams::default())
    }

    pub fn summarize(&self, shape: &Shape) -> Result<ShapeSummary, Error> {
        self.context.describe_shape(shape)
    }

    pub fn topology(&self, shape: &Shape) -> Result<TopologySnapshot, Error> {
        self.context.topology(shape)
    }

    pub fn brep(&self, shape: &Shape) -> Result<BrepShape, Error> {
        self.context.ported_brep(shape)
    }

    pub fn mesh(&self, shape: &Shape, params: MeshParams) -> Result<Mesh, Error> {
        self.context.mesh(shape, params)
    }

    pub fn inspect_with_mesh(
        &self,
        shape: &Shape,
        mesh_params: MeshParams,
    ) -> Result<ShapeReport, Error> {
        Ok(ShapeReport {
            summary: self.summarize(shape)?,
            mesh: self.mesh(shape, mesh_params)?,
            topology: self.topology(shape)?,
        })
    }

    pub fn read_step(&self, path: impl AsRef<Path>) -> Result<Shape, Error> {
        self.context.read_step(path)
    }

    pub fn write_step(&self, shape: &Shape, path: impl AsRef<Path>) -> Result<(), Error> {
        self.context.write_step(shape, path)
    }

    pub fn step_round_trip_temp(&self, shape: &Shape) -> Result<Shape, Error> {
        let path = unique_temp_step_path("lean_occt-roundtrip");
        self.context.write_step(shape, &path)?;
        let result = self.context.read_step(&path);
        let _ = fs::remove_file(&path);
        result
    }
}

fn unique_temp_step_path(prefix: &str) -> PathBuf {
    let counter = STEP_ROUND_TRIP_COUNTER.fetch_add(1, Ordering::Relaxed);
    let timestamp_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or(0);
    let filename = format!(
        "{prefix}-{}-{counter}-{timestamp_nanos}.step",
        process::id()
    );
    std::env::temp_dir().join(filename)
}
