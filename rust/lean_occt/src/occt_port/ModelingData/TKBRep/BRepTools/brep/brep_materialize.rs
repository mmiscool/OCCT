use super::topology::{adjacent_face_indices, edge_points, topology_edge};
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
    shape: &Shape,
    topology: &TopologySnapshot,
) -> Result<(Vec<Shape>, Vec<BrepEdge>), Error> {
    let edge_shapes = context.subshapes_occt(shape, ShapeKind::Edge)?;
    let edges = edge_shapes
        .iter()
        .enumerate()
        .map(|(index, edge_shape)| {
            let topology_edge = topology_edge(topology, index)?;
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

    Ok((edge_shapes, edges))
}
