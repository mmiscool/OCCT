use super::topology_access::{adjacent_face_indices, edge_points, topology_edge};
use super::*;

pub(super) fn ported_brep_vertices(topology: &TopologySnapshot) -> Vec<BrepVertex> {
    topology
        .vertex_positions
        .iter()
        .copied()
        .enumerate()
        .map(|(index, position)| BrepVertex { index, position })
        .collect()
}

pub(super) fn ported_brep_wires(topology: &TopologySnapshot) -> Vec<BrepWire> {
    topology
        .wires
        .iter()
        .enumerate()
        .map(|(index, range)| {
            let edge_indices =
                topology.wire_edge_indices[range.offset..range.offset + range.count].to_vec();
            let edge_orientations =
                topology.wire_edge_orientations[range.offset..range.offset + range.count].to_vec();
            let vertex_range = topology.wire_vertices[index];
            let vertex_indices = topology.wire_vertex_indices
                [vertex_range.offset..vertex_range.offset + vertex_range.count]
                .to_vec();
            BrepWire {
                index,
                edge_indices,
                edge_orientations,
                vertex_indices,
            }
        })
        .collect()
}

pub(super) fn ported_brep_edges(
    context: &Context,
    edge_shapes: &[Shape],
    topology: &TopologySnapshot,
) -> Result<Vec<BrepEdge>, Error> {
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| {
            let topology_edge = topology_edge(topology, index)?;
            let (geometry, ported_curve) =
                ported_brep_edge_geometry_and_curve(context, edge_shape)?;
            let adjacent_face_indices = adjacent_face_indices(topology, index)?;
            let (start_point, end_point) = edge_points(topology, index);
            let length = match ported_curve {
                Some(curve) => curve.length_with_geometry(geometry),
                None => topology_edge.length,
            };

            Ok(BrepEdge {
                index,
                geometry,
                ported_curve,
                length,
                start_vertex: topology_edge.start_vertex,
                end_vertex: topology_edge.end_vertex,
                start_point,
                end_point,
                adjacent_face_indices,
            })
        })
        .collect::<Result<Vec<_>, Error>>()?;

    Ok(edges)
}

pub(super) fn ported_brep_edge_geometry_and_curve(
    context: &Context,
    edge_shape: &Shape,
) -> Result<(EdgeGeometry, Option<PortedCurve>), Error> {
    let geometry = context.edge_geometry(edge_shape)?;
    let ported_curve =
        PortedCurve::from_context_with_ported_payloads(context, edge_shape, geometry)?;
    require_ported_brep_curve(geometry, ported_curve)?;
    Ok((geometry, ported_curve))
}

fn require_ported_brep_curve(
    geometry: EdgeGeometry,
    ported_curve: Option<PortedCurve>,
) -> Result<(), Error> {
    if rust_owned_brep_curve_required(geometry.kind) && ported_curve.is_none() {
        Err(Error::new(format!(
            "Rust-owned BRep edge materialization did not cover {:?} edge",
            geometry.kind
        )))
    } else {
        Ok(())
    }
}

fn rust_owned_brep_curve_required(kind: CurveKind) -> bool {
    matches!(
        kind,
        CurveKind::Line | CurveKind::Circle | CurveKind::Ellipse
    )
}
