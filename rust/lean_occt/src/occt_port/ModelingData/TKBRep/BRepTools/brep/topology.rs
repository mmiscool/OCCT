use super::root_topology::{
    oriented_edge_geometry, root_edge_topology, root_wire_topology, RootEdgeTopology,
    RootWireTopology,
};
use super::*;

use super::swept_face::append_root_edge_sample_points;

struct PortedFaceTopology {
    edge_indices: BTreeSet<usize>,
    face_wire_indices: Vec<usize>,
    face_wire_orientations: Vec<Orientation>,
    face_wire_roles: Vec<LoopRole>,
}

pub(super) fn ported_topology_snapshot(
    context: &Context,
    shape: &Shape,
) -> Result<Option<TopologySnapshot>, Error> {
    let face_shapes = context.subshapes_occt(shape, ShapeKind::Face)?;
    for face_shape in &face_shapes {
        let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
        let geometry = match context.face_geometry(face_shape) {
            Ok(geometry) => geometry,
            Err(_) => context.face_geometry_occt(face_shape)?,
        };
        if face_wire_shapes.len() > 1 && geometry.kind != crate::SurfaceKind::Plane {
            return Ok(None);
        }
    }

    let vertex_shapes = context.subshapes_occt(shape, ShapeKind::Vertex)?;
    let vertex_positions = vertex_shapes
        .iter()
        .map(|vertex_shape| context.vertex_point_occt(vertex_shape))
        .collect::<Result<Vec<_>, Error>>()?;

    let edge_shapes = context.subshapes_occt(shape, ShapeKind::Edge)?;
    let root_edges = edge_shapes
        .iter()
        .map(|edge_shape| root_edge_topology(context, edge_shape, &vertex_positions))
        .collect::<Result<Vec<_>, Error>>()?;
    let edges = root_edges
        .iter()
        .map(|edge| crate::TopologyEdge {
            start_vertex: edge.start_vertex,
            end_vertex: edge.end_vertex,
            length: edge.length,
        })
        .collect::<Vec<_>>();

    let wire_shapes = context.subshapes_occt(shape, ShapeKind::Wire)?;
    let mut root_wires = Vec::with_capacity(wire_shapes.len());
    for wire_shape in &wire_shapes {
        let Some(topology) =
            root_wire_topology(context, wire_shape, &vertex_positions, &root_edges)?
        else {
            return Ok(None);
        };
        root_wires.push(topology);
    }
    let (wires, wire_edge_indices, wire_edge_orientations, wire_vertices, wire_vertex_indices) =
        pack_wire_topology(&root_wires);
    let mut edge_face_lists = vec![Vec::new(); edges.len()];
    let mut faces = Vec::with_capacity(face_shapes.len());
    let mut face_wire_indices = Vec::new();
    let mut face_wire_orientations = Vec::new();
    let mut face_wire_roles = Vec::new();

    for (face_index, face_shape) in face_shapes.iter().enumerate() {
        let Some(face_topology) = ported_face_topology(
            context,
            face_shape,
            &root_wires,
            &root_edges,
            &edge_shapes,
            &vertex_positions,
        )?
        else {
            return Ok(None);
        };

        faces.push(crate::TopologyRange {
            offset: face_wire_indices.len(),
            count: face_topology.face_wire_indices.len(),
        });
        face_wire_indices.extend(face_topology.face_wire_indices);
        face_wire_orientations.extend(face_topology.face_wire_orientations);
        face_wire_roles.extend(face_topology.face_wire_roles);

        for edge_index in face_topology.edge_indices {
            let Some(edge_faces) = edge_face_lists.get_mut(edge_index) else {
                return Ok(None);
            };
            edge_faces.push(face_index);
        }
    }

    let mut edge_faces = Vec::with_capacity(edges.len());
    let mut edge_face_indices = Vec::new();
    for face_indices in edge_face_lists {
        edge_faces.push(crate::TopologyRange {
            offset: edge_face_indices.len(),
            count: face_indices.len(),
        });
        edge_face_indices.extend(face_indices);
    }

    Ok(Some(TopologySnapshot {
        vertex_positions,
        edges,
        edge_faces,
        edge_face_indices,
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
        faces,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

fn pack_wire_topology(
    root_wires: &[RootWireTopology],
) -> (
    Vec<crate::TopologyRange>,
    Vec<usize>,
    Vec<Orientation>,
    Vec<crate::TopologyRange>,
    Vec<usize>,
) {
    let mut wires = Vec::with_capacity(root_wires.len());
    let mut wire_edge_indices = Vec::new();
    let mut wire_edge_orientations = Vec::new();
    let mut wire_vertices = Vec::with_capacity(root_wires.len());
    let mut wire_vertex_indices = Vec::new();

    for wire in root_wires {
        wires.push(crate::TopologyRange {
            offset: wire_edge_indices.len(),
            count: wire.edge_indices.len(),
        });
        wire_edge_indices.extend(&wire.edge_indices);
        wire_edge_orientations.extend(&wire.edge_orientations);
        wire_vertices.push(crate::TopologyRange {
            offset: wire_vertex_indices.len(),
            count: wire.vertex_indices.len(),
        });
        wire_vertex_indices.extend(&wire.vertex_indices);
    }

    (
        wires,
        wire_edge_indices,
        wire_edge_orientations,
        wire_vertices,
        wire_vertex_indices,
    )
}

fn ported_face_topology(
    context: &Context,
    face_shape: &Shape,
    root_wires: &[RootWireTopology],
    root_edges: &[RootEdgeTopology],
    edge_shapes: &[Shape],
    vertex_positions: &[[f64; 3]],
) -> Result<Option<PortedFaceTopology>, Error> {
    let face_wire_shapes = context.subshapes_occt(face_shape, ShapeKind::Wire)?;
    if root_wires.is_empty() || face_wire_shapes.is_empty() {
        return Ok(None);
    }

    let mut used_root_wire_indices = BTreeSet::new();
    let mut used_edges = BTreeSet::new();
    let mut face_wire_indices = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_orientations = Vec::with_capacity(face_wire_shapes.len());
    let mut face_wire_areas = Vec::new();

    let mut planar_face = None;
    if face_wire_shapes.len() > 1 {
        let face_geometry = context.face_geometry_occt(face_shape)?;
        if face_geometry.kind != crate::SurfaceKind::Plane {
            return Ok(None);
        }
        planar_face = Some((context.face_plane_payload_occt(face_shape)?, face_geometry));
    }

    for face_wire_shape in &face_wire_shapes {
        let Some(face_wire_topology) =
            root_wire_topology(context, face_wire_shape, vertex_positions, root_edges)?
        else {
            return Ok(None);
        };
        let Some(root_wire_index) =
            match_root_wire_index(root_wires, &face_wire_topology, &used_root_wire_indices)
        else {
            return Ok(None);
        };
        used_root_wire_indices.insert(root_wire_index);
        used_edges.extend(face_wire_topology.edge_indices.iter().copied());

        face_wire_indices.push(root_wire_index);
        face_wire_orientations.push(context.shape_orientation(face_wire_shape)?);

        if let Some((plane, face_geometry)) = planar_face {
            let Some(wire_area) = planar_wire_area_magnitude(
                context,
                plane,
                face_geometry,
                &root_wires[root_wire_index],
                edge_shapes,
                root_edges,
            )?
            else {
                return Ok(None);
            };
            face_wire_areas.push(wire_area);
        }
    }

    let face_wire_roles = match face_wire_shapes.len() {
        1 => vec![LoopRole::Outer],
        _ => {
            let Some((outer_offset, outer_area)) = face_wire_areas
                .iter()
                .copied()
                .enumerate()
                .max_by(|(_, lhs), (_, rhs)| lhs.total_cmp(rhs))
            else {
                return Ok(None);
            };
            if outer_area <= 1.0e-9 {
                return Ok(None);
            }

            face_wire_areas
                .iter()
                .enumerate()
                .map(|(offset, _)| {
                    if offset == outer_offset {
                        LoopRole::Outer
                    } else {
                        LoopRole::Inner
                    }
                })
                .collect()
        }
    };

    Ok(Some(PortedFaceTopology {
        edge_indices: used_edges,
        face_wire_indices,
        face_wire_orientations,
        face_wire_roles,
    }))
}

fn match_root_wire_index(
    root_wires: &[RootWireTopology],
    face_wire_topology: &RootWireTopology,
    used_root_wire_indices: &BTreeSet<usize>,
) -> Option<usize> {
    root_wires
        .iter()
        .enumerate()
        .find(|(index, root_wire)| {
            !used_root_wire_indices.contains(index) && *root_wire == face_wire_topology
        })
        .map(|(index, _)| index)
}

fn planar_wire_area_magnitude(
    context: &Context,
    plane: PlanePayload,
    face_geometry: FaceGeometry,
    wire: &RootWireTopology,
    edge_shapes: &[Shape],
    root_edges: &[RootEdgeTopology],
) -> Result<Option<f64>, Error> {
    let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
    let mut sampled_points = Vec::new();

    for (&edge_index, &edge_orientation) in wire.edge_indices.iter().zip(&wire.edge_orientations) {
        let Some(root_edge) = root_edges.get(edge_index) else {
            return Ok(None);
        };
        let Some(edge_shape) = edge_shapes.get(edge_index) else {
            return Ok(None);
        };

        let geometry = oriented_edge_geometry(root_edge.geometry, edge_orientation);
        if let Some(curve) =
            PortedCurve::from_context_with_geometry(context, edge_shape, root_edge.geometry)?
        {
            curve_segments.push((curve, geometry));
        }

        append_root_edge_sample_points(
            context,
            edge_shape,
            root_edge,
            geometry,
            &mut sampled_points,
        )?;
    }

    let area = if curve_segments.len() == wire.edge_indices.len() {
        planar_wire_signed_area(plane, &curve_segments).abs()
    } else {
        let Some(area) = analytic_sampled_wire_signed_area(
            PortedSurface::Plane(plane),
            face_geometry,
            &sampled_points,
        ) else {
            return Ok(None);
        };
        area.abs()
    };
    Ok(Some(area))
}
