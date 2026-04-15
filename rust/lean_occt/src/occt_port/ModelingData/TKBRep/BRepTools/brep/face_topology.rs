use super::topology::{face_loops, ported_brep_wires};
use super::*;

#[derive(Clone, Copy)]
pub(super) enum FaceSurfaceRoute {
    Raw,
    Public,
}

pub(super) struct SingleFaceTopology {
    pub(super) loops: Vec<BrepFaceLoop>,
    pub(super) wires: Vec<BrepWire>,
    pub(super) edges: Vec<BrepEdge>,
    pub(super) edge_shapes: Vec<Shape>,
}

pub(super) fn single_face_topology_with_route(
    context: &Context,
    face_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<Option<SingleFaceTopology>, Error> {
    let topology = match single_face_topology_snapshot(context, face_shape)? {
        Some(topology) => topology,
        None => return Ok(None),
    };

    let wires = ported_brep_wires(&topology);
    let edge_shapes = context.subshapes_occt(face_shape, ShapeKind::Edge)?;
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| single_face_edge_with_route(context, index, edge_shape, route))
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(Some(SingleFaceTopology {
        loops: face_loops(&topology, 0)?,
        wires,
        edges,
        edge_shapes,
    }))
}

fn single_face_topology_snapshot(
    context: &Context,
    face_shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let topology = match context.ported_topology(face_shape)? {
        Some(topology) => topology,
        None => context.topology_occt(face_shape)?,
    };
    if topology.faces.len() != 1 {
        return Ok(None);
    }
    Ok(Some(topology))
}

fn single_face_edge_with_route(
    context: &Context,
    index: usize,
    edge_shape: &Shape,
    route: FaceSurfaceRoute,
) -> Result<BrepEdge, Error> {
    let (geometry, ported_curve) = match route {
        FaceSurfaceRoute::Raw => {
            let geometry = context.edge_geometry_occt(edge_shape)?;
            let ported_curve =
                PortedCurve::from_context_with_geometry(context, edge_shape, geometry)?;
            (geometry, ported_curve)
        }
        FaceSurfaceRoute::Public => {
            let geometry = match context.edge_geometry(edge_shape) {
                Ok(geometry) => geometry,
                Err(_) => context.edge_geometry_occt(edge_shape)?,
            };
            let ported_curve =
                match PortedCurve::from_context_with_ported_payloads(context, edge_shape, geometry)
                {
                    Ok(ported_curve) => ported_curve,
                    Err(_) => {
                        PortedCurve::from_context_with_geometry(context, edge_shape, geometry)?
                    }
                };
            (geometry, ported_curve)
        }
    };
    Ok(single_face_edge(index, geometry, ported_curve))
}

fn single_face_edge(
    index: usize,
    geometry: EdgeGeometry,
    ported_curve: Option<PortedCurve>,
) -> BrepEdge {
    BrepEdge {
        index,
        geometry,
        ported_curve,
        length: 0.0,
        start_vertex: None,
        end_vertex: None,
        start_point: None,
        end_point: None,
        adjacent_face_indices: Vec::new(),
    }
}
