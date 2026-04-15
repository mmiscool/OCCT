use super::face_metrics::{
    analytic_face_area, analytic_offset_face_area, analytic_ported_swept_face_area,
};
use super::summary::{mesh_face_properties, MeshFaceProperties};
use super::swept_face::{
    face_curve_candidates, select_swept_face_basis_curve, FaceCurveCandidate, SweptBasisSelection,
};
use super::topology::{
    face_adjacent_face_indices, face_loops, single_face_topology_with_route, FaceSurfaceRoute,
    SingleFaceTopology,
};
use super::*;

struct PreparedFaceSurface {
    geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
    ported_face_surface: Option<PortedFaceSurface>,
}

struct LazyMeshFaceFallback<'a> {
    context: &'a Context,
    face_shape: &'a Shape,
    orientation: Orientation,
    properties: Option<MeshFaceProperties>,
    loaded: bool,
}

impl<'a> LazyMeshFaceFallback<'a> {
    fn new(
        context: &'a Context,
        face_shape: &'a Shape,
        orientation: Orientation,
        eagerly_load: bool,
    ) -> Self {
        let properties = if eagerly_load {
            mesh_face_properties(context, face_shape, orientation)
        } else {
            None
        };

        Self {
            context,
            face_shape,
            orientation,
            properties,
            loaded: eagerly_load,
        }
    }

    fn resolve_sample(
        &mut self,
        sample: Option<FaceSample>,
        index: usize,
        geometry: FaceGeometry,
    ) -> Result<FaceSample, Error> {
        sample
            .or_else(|| self.load().map(|fallback| fallback.sample))
            .ok_or_else(|| {
                Error::new(format!(
                    "failed to derive a Rust-owned sample for face {index} ({:?})",
                    geometry.kind
                ))
            })
    }

    fn resolve_area(
        &mut self,
        area: Option<f64>,
        index: usize,
        geometry: FaceGeometry,
    ) -> Result<f64, Error> {
        area.or_else(|| self.load().map(|fallback| fallback.area))
            .ok_or_else(|| {
                Error::new(format!(
                    "failed to derive a Rust-owned area for face {index} ({:?})",
                    geometry.kind
                ))
            })
    }

    fn load(&mut self) -> Option<MeshFaceProperties> {
        if !self.loaded {
            self.properties = mesh_face_properties(self.context, self.face_shape, self.orientation);
            self.loaded = true;
        }

        self.properties
    }
}

pub(crate) fn ported_face_surface_descriptor(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
) -> Result<Option<PortedFaceSurface>, Error> {
    Ok(prepare_face_surface_with_geometry(
        context,
        face_shape,
        face_geometry,
        FaceSurfaceRoute::Public,
    )?
    .ported_face_surface)
}

pub(super) fn ported_brep_faces(
    context: &Context,
    shape: &Shape,
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Result<(Vec<Shape>, Vec<BrepFace>), Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    let faces = face_shapes
        .iter()
        .enumerate()
        .map(|(index, face_shape)| {
            ported_brep_face(
                context,
                topology,
                wires,
                edges,
                edge_shapes,
                index,
                face_shape,
            )
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok((face_shapes, faces))
}

fn ported_brep_face(
    context: &Context,
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
    index: usize,
    face_shape: &Shape,
) -> Result<BrepFace, Error> {
    let prepared = prepare_face_surface_with_route(context, face_shape, FaceSurfaceRoute::Raw)?;
    let geometry = prepared.geometry;
    let ported_surface = prepared.ported_surface;
    let ported_face_surface = prepared.ported_face_surface;
    let orientation = context.shape_orientation(face_shape)?;
    let loops = face_loops(topology, index)?;
    let mut mesh_fallback = LazyMeshFaceFallback::new(
        context,
        face_shape,
        orientation,
        ported_face_surface.is_none(),
    );
    let sample = mesh_fallback.resolve_sample(
        ported_face_surface.map(|surface| {
            surface.sample_normalized_with_orientation(geometry, [0.5, 0.5], orientation)
        }),
        index,
        geometry,
    )?;
    let area = ported_face_area_from_surface(
        context,
        ported_face_surface,
        geometry,
        &loops,
        wires,
        edges,
        edge_shapes,
    );
    let area = mesh_fallback.resolve_area(area, index, geometry)?;
    let adjacent_face_indices = face_adjacent_face_indices(topology, wires, index)?;

    Ok(BrepFace {
        index,
        geometry,
        ported_surface,
        ported_face_surface,
        orientation,
        area,
        sample,
        loops,
        adjacent_face_indices,
    })
}

fn ported_face_area_from_surface(
    context: &Context,
    ported_face_surface: Option<PortedFaceSurface>,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    match ported_face_surface {
        Some(PortedFaceSurface::Analytic(surface)) => analytic_face_area(
            context,
            surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        ),
        Some(PortedFaceSurface::Offset(surface)) => analytic_offset_face_area(
            context,
            surface,
            face_geometry,
            loops,
            wires,
            edges,
            edge_shapes,
        ),
        Some(PortedFaceSurface::Swept(surface)) => {
            analytic_ported_swept_face_area(surface, face_geometry)
        }
        None => None,
    }
}

fn ported_face_surface_descriptor_from_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    ported_surface: Option<PortedSurface>,
    route: FaceSurfaceRoute,
) -> Result<Option<PortedFaceSurface>, Error> {
    if let Some(surface) = ported_surface {
        return Ok(Some(PortedFaceSurface::Analytic(surface)));
    }

    if let Some(surface) = context.ported_offset_surface(face_shape)? {
        return Ok(Some(PortedFaceSurface::Offset(surface)));
    }

    Ok(
        ported_swept_face_surface_with_route(context, face_shape, face_geometry, route)?
            .map(PortedFaceSurface::Swept),
    )
}

fn ported_swept_face_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    route: FaceSurfaceRoute,
) -> Result<Option<PortedSweptSurface>, Error> {
    let topology = match single_face_topology_with_route(context, face_shape, route)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    ported_swept_face_surface_from_topology(context, face_shape, face_geometry, topology)
}

fn ported_swept_face_surface_from_topology(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: SingleFaceTopology,
) -> Result<Option<PortedSweptSurface>, Error> {
    match face_geometry.kind {
        crate::SurfaceKind::Extrusion => {
            ported_extrusion_face_surface(context, face_shape, face_geometry, &topology).map(Some)
        }
        crate::SurfaceKind::Revolution => {
            ported_revolution_face_surface(context, face_shape, face_geometry, &topology).map(Some)
        }
        _ => Ok(None),
    }
}

fn ported_extrusion_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: &SingleFaceTopology,
) -> Result<PortedSweptSurface, Error> {
    let payload = context.face_extrusion_payload_occt(face_shape)?;
    let basis = select_swept_face_basis(
        topology,
        face_geometry,
        payload.basis_curve_kind,
        SweptBasisSelection::Extrusion {
            direction: payload.direction,
        },
        "extrusion",
    )?;

    Ok(PortedSweptSurface::Extrusion {
        payload,
        basis_curve: basis.curve,
        basis_geometry: basis.geometry,
    })
}

fn ported_revolution_face_surface(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    topology: &SingleFaceTopology,
) -> Result<PortedSweptSurface, Error> {
    let payload = context.face_revolution_payload_occt(face_shape)?;
    let basis = select_swept_face_basis(
        topology,
        face_geometry,
        payload.basis_curve_kind,
        SweptBasisSelection::Revolution {
            axis_origin: payload.axis_origin,
            axis_direction: payload.axis_direction,
        },
        "revolution",
    )?;

    Ok(PortedSweptSurface::Revolution {
        payload,
        basis_curve: basis.curve,
        basis_geometry: basis.geometry,
    })
}

fn select_swept_face_basis(
    topology: &SingleFaceTopology,
    face_geometry: FaceGeometry,
    basis_curve_kind: crate::CurveKind,
    selection: SweptBasisSelection,
    face_kind: &'static str,
) -> Result<FaceCurveCandidate, Error> {
    let candidates = face_curve_candidates(
        &topology.loops,
        &topology.wires,
        &topology.edges,
        basis_curve_kind,
    )
    .ok_or_else(|| {
        Error::new(format!(
            "failed to identify a Rust-owned basis curve for {face_kind} face"
        ))
    })?;

    select_swept_face_basis_curve(candidates, face_geometry, selection).ok_or_else(|| {
        Error::new(format!(
            "failed to select a Rust-owned basis curve for {face_kind} face"
        ))
    })
}

fn prepare_face_surface_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<PreparedFaceSurface, Error> {
    let face_geometry = face_geometry_with_route(context, face_shape, route)?;
    prepare_face_surface_with_geometry(context, face_shape, face_geometry, route)
}

fn prepare_face_surface_with_geometry(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    route: FaceSurfaceRoute,
) -> Result<PreparedFaceSurface, Error> {
    let ported_surface =
        PortedSurface::from_context_with_geometry(context, face_shape, face_geometry)?;
    let ported_face_surface = ported_face_surface_descriptor_from_surface_with_route(
        context,
        face_shape,
        face_geometry,
        ported_surface,
        route,
    )?;

    Ok(PreparedFaceSurface {
        geometry: face_geometry,
        ported_surface,
        ported_face_surface,
    })
}

fn face_geometry_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<FaceGeometry, Error> {
    match route {
        FaceSurfaceRoute::Raw => context.face_geometry_occt(face_shape),
        FaceSurfaceRoute::Public => context.face_geometry(face_shape),
    }
}

pub(crate) fn ported_face_area(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<f64>, Error> {
    let topology =
        match single_face_topology_with_route(context, face_shape, FaceSurfaceRoute::Public)? {
            Some(topology) => topology,
            None => return Ok(None),
        };

    let prepared = prepare_face_surface_with_route(context, face_shape, FaceSurfaceRoute::Public)?;
    let face_geometry = prepared.geometry;
    Ok(ported_face_area_from_surface(
        context,
        prepared.ported_face_surface,
        face_geometry,
        &topology.loops,
        &topology.wires,
        &topology.edges,
        &topology.edge_shapes,
    ))
}
