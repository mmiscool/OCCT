use std::collections::BTreeSet;
use std::f64::consts::PI;

use crate::ported_geometry::{
    analytic_sampled_wire_signed_area, analytic_sampled_wire_signed_volume, extrusion_swept_area,
    planar_wire_signed_area, revolution_swept_area, sample_extrusion_surface_normalized,
    sample_revolution_surface_normalized,
};
use crate::{
    ConePayload, Context, CylinderPayload, EdgeGeometry, Error, FaceGeometry, FaceSample, LoopRole,
    Mesh, MeshParams, Orientation, PlanePayload, PortedCurve, PortedSurface, Shape, ShapeKind,
    ShapeSummary, SpherePayload, TopologySnapshot, TorusPayload,
};

const SUMMARY_VOLUME_MESH_PARAMS: MeshParams = MeshParams {
    linear_deflection: 0.01,
    angular_deflection: 0.05,
    is_relative: false,
};

const SUMMARY_BBOX_MESH_PARAMS: MeshParams = MeshParams {
    linear_deflection: 0.01,
    angular_deflection: 0.05,
    is_relative: false,
};

const UNSUPPORTED_FACE_AREA_MESH_PARAMS: MeshParams = MeshParams {
    linear_deflection: 0.01,
    angular_deflection: 0.05,
    is_relative: false,
};

#[derive(Clone, Copy, Debug)]
pub struct BrepVertex {
    pub index: usize,
    pub position: [f64; 3],
}

#[derive(Clone, Debug)]
pub struct BrepWire {
    pub index: usize,
    pub edge_indices: Vec<usize>,
    pub edge_orientations: Vec<Orientation>,
    pub vertex_indices: Vec<usize>,
}

#[derive(Clone, Copy, Debug)]
pub struct BrepFaceLoop {
    pub wire_index: usize,
    pub orientation: Orientation,
    pub role: LoopRole,
}

#[derive(Clone, Debug)]
pub struct BrepEdge {
    pub index: usize,
    pub geometry: EdgeGeometry,
    pub ported_curve: Option<PortedCurve>,
    pub length: f64,
    pub start_vertex: Option<usize>,
    pub end_vertex: Option<usize>,
    pub start_point: Option<[f64; 3]>,
    pub end_point: Option<[f64; 3]>,
    pub adjacent_face_indices: Vec<usize>,
}

#[derive(Clone, Debug)]
pub struct BrepFace {
    pub index: usize,
    pub geometry: FaceGeometry,
    pub ported_surface: Option<PortedSurface>,
    pub orientation: Orientation,
    pub area: f64,
    pub sample: FaceSample,
    pub loops: Vec<BrepFaceLoop>,
    pub adjacent_face_indices: Vec<usize>,
}

#[derive(Debug)]
pub struct BrepShape {
    pub summary: ShapeSummary,
    pub topology: TopologySnapshot,
    pub vertices: Vec<BrepVertex>,
    pub wires: Vec<BrepWire>,
    pub edges: Vec<BrepEdge>,
    pub faces: Vec<BrepFace>,
}

#[derive(Clone, Copy, Debug)]
struct ExactPrimitiveSummary {
    surface_area: f64,
    volume: f64,
}

#[derive(Clone, Copy, Debug)]
struct ShapeCounts {
    compound_count: usize,
    compsolid_count: usize,
    solid_count: usize,
    shell_count: usize,
    face_count: usize,
    wire_count: usize,
    edge_count: usize,
    vertex_count: usize,
}

#[derive(Clone, Copy, Debug)]
struct FaceCurveCandidate {
    curve: PortedCurve,
    geometry: EdgeGeometry,
    midpoint: [f64; 3],
}

impl Context {
    pub fn ported_brep(&self, shape: &Shape) -> Result<BrepShape, Error> {
        let topology = self.topology(shape)?;
        let vertices = topology
            .vertex_positions
            .iter()
            .copied()
            .enumerate()
            .map(|(index, position)| BrepVertex { index, position })
            .collect::<Vec<_>>();

        let wires = topology
            .wires
            .iter()
            .enumerate()
            .map(|(index, range)| {
                let edge_indices =
                    topology.wire_edge_indices[range.offset..range.offset + range.count].to_vec();
                let edge_orientations = topology.wire_edge_orientations
                    [range.offset..range.offset + range.count]
                    .to_vec();
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
            .collect::<Vec<_>>();

        let edge_shapes = self.subshapes(shape, ShapeKind::Edge)?;
        let edges = edge_shapes
            .iter()
            .enumerate()
            .map(|(index, edge_shape)| {
                let topology_edge = topology_edge(&topology, index)?;
                let geometry = self.edge_geometry(edge_shape)?;
                let ported_curve =
                    PortedCurve::from_context_with_geometry(self, edge_shape, geometry)?;
                let adjacent_face_indices = adjacent_face_indices(&topology, index)?;
                let (start_point, end_point) = edge_points(&topology, index);
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

        let face_shapes = self.subshapes(shape, ShapeKind::Face)?;
        let faces = face_shapes
            .iter()
            .enumerate()
            .map(|(index, face_shape)| {
                let geometry = self.face_geometry(face_shape)?;
                let ported_surface =
                    PortedSurface::from_context_with_geometry(self, face_shape, geometry)?;
                let orientation = self.shape_orientation(face_shape)?;
                let loops = face_loops(&topology, index)?;
                let sample = match ported_surface {
                    Some(surface) => surface.sample_normalized_with_orientation(
                        geometry,
                        [0.5, 0.5],
                        orientation,
                    ),
                    None => analytic_swept_face_sample(
                        self,
                        face_shape,
                        geometry,
                        orientation,
                        &loops,
                        &wires,
                        &edges,
                    )
                    .unwrap_or(self.face_sample_normalized(face_shape, [0.5, 0.5])?),
                };
                let area = match ported_surface {
                    Some(surface) => analytic_face_area(
                        self,
                        surface,
                        geometry,
                        &loops,
                        &wires,
                        &edges,
                        &edge_shapes,
                    )
                    .or_else(|| mesh_face_area(self, face_shape))
                    .unwrap_or(self.describe_shape(face_shape)?.surface_area),
                    None => {
                        analytic_swept_face_area(self, face_shape, geometry, &loops, &wires, &edges)
                            .or_else(|| mesh_face_area(self, face_shape))
                            .unwrap_or(self.describe_shape(face_shape)?.surface_area)
                    }
                };
                let adjacent_face_indices = face_adjacent_face_indices(&topology, &wires, index)?;

                Ok(BrepFace {
                    index,
                    geometry,
                    ported_surface,
                    orientation,
                    area,
                    sample,
                    loops,
                    adjacent_face_indices,
                })
            })
            .collect::<Result<Vec<_>, Error>>()?;

        let summary = ported_shape_summary(
            self,
            shape,
            &vertices,
            &topology,
            &wires,
            &edges,
            &faces,
            &face_shapes,
            &edge_shapes,
        )?;

        Ok(BrepShape {
            summary,
            topology,
            vertices,
            wires,
            edges,
            faces,
        })
    }
}

fn ported_shape_summary(
    context: &Context,
    shape: &Shape,
    vertices: &[BrepVertex],
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Result<ShapeSummary, Error> {
    let counts = shape_counts(context, shape, topology)?;
    let root_kind = classify_root_kind(counts);
    let primary_kind = classify_primary_kind(counts);
    let exact_primitive =
        exact_primitive_shape_summary(primary_kind, counts.solid_count, vertices, edges, faces);
    let fallback_summary = || context.describe_shape(shape).ok();
    let (bbox_min, bbox_max) = ported_shape_bbox(context, shape, vertices, edges, faces)
        .or_else(|| fallback_summary().map(|summary| (summary.bbox_min, summary.bbox_max)))
        .unwrap_or(([0.0; 3], [0.0; 3]));

    Ok(ShapeSummary {
        root_kind,
        primary_kind,
        compound_count: counts.compound_count,
        compsolid_count: counts.compsolid_count,
        solid_count: counts.solid_count,
        shell_count: counts.shell_count,
        face_count: counts.face_count,
        wire_count: counts.wire_count,
        edge_count: counts.edge_count,
        vertex_count: counts.vertex_count,
        // Match OCCT's whole-shape linear properties semantics: when wires are present,
        // length is accumulated over wire-edge occurrences rather than unique topological edges.
        linear_length: if topology.wire_edge_indices.is_empty() {
            edges.iter().map(|edge| edge.length).sum()
        } else {
            topology
                .wire_edge_indices
                .iter()
                .filter_map(|&edge_index| edges.get(edge_index))
                .map(|edge| edge.length)
                .sum()
        },
        surface_area: exact_primitive
            .map(|summary| summary.surface_area)
            .unwrap_or_else(|| faces.iter().map(|face| face.area).sum()),
        volume: exact_primitive
            .map(|summary| summary.volume)
            .or_else(|| {
                analytic_shape_volume(context, wires, edges, faces, face_shapes, edge_shapes)
            })
            .or_else(|| mesh_shape_volume(context, shape, counts))
            .or_else(|| fallback_summary().map(|summary| summary.volume))
            .unwrap_or(0.0),
        bbox_min,
        bbox_max,
    })
}

fn exact_primitive_shape_summary(
    primary_kind: ShapeKind,
    solid_count: usize,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<ExactPrimitiveSummary> {
    if primary_kind != ShapeKind::Solid || solid_count != 1 {
        return None;
    }

    exact_box_summary(vertices, edges, faces)
        .or_else(|| exact_cylinder_summary(faces))
        .or_else(|| exact_cone_summary(faces))
        .or_else(|| exact_sphere_summary(faces))
        .or_else(|| exact_torus_summary(faces))
        .or_else(|| exact_translational_solid_summary(faces))
}

fn exact_box_summary(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<ExactPrimitiveSummary> {
    if vertices.len() != 8 || edges.len() != 12 || faces.len() != 6 {
        return None;
    }
    if !edges
        .iter()
        .all(|edge| matches!(edge.ported_curve, Some(PortedCurve::Line(_))))
    {
        return None;
    }
    if !faces
        .iter()
        .all(|face| matches!(face.ported_surface, Some(PortedSurface::Plane(_))))
    {
        return None;
    }

    for vertex in vertices {
        let incident = edges
            .iter()
            .filter_map(|edge| incident_edge_vector(edge, vertex.index))
            .collect::<Vec<_>>();
        if incident.len() < 3 {
            continue;
        }

        for i in 0..incident.len() {
            for j in i + 1..incident.len() {
                for k in j + 1..incident.len() {
                    let a = incident[i];
                    let b = incident[j];
                    let c = incident[k];
                    let volume = dot3(a, cross3(b, c)).abs();
                    if volume <= 1.0e-9 {
                        continue;
                    }
                    let surface_area =
                        2.0 * (norm3(cross3(a, b)) + norm3(cross3(a, c)) + norm3(cross3(b, c)));
                    return Some(ExactPrimitiveSummary {
                        surface_area,
                        volume,
                    });
                }
            }
        }
    }

    None
}

fn exact_cylinder_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    if faces.len() != 3 {
        return None;
    }

    let (payload, _) = single_cylinder_face(faces)?;
    let axis = normalize3(payload.axis);
    let caps = aligned_plane_faces(faces, axis);
    if caps.len() != 2 {
        return None;
    }

    let height = (dot3(subtract3(caps[0].origin, payload.origin), axis)
        - dot3(subtract3(caps[1].origin, payload.origin), axis))
    .abs();
    let radius = payload.radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 2.0 * PI * radius * (height + radius),
        volume: PI * radius * radius * height,
    })
}

fn exact_cone_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    if !(2..=3).contains(&faces.len()) {
        return None;
    }

    let (payload, _) = single_cone_face(faces)?;
    let axis = normalize3(payload.axis);
    let caps = aligned_plane_faces(faces, axis);
    if caps.is_empty() || caps.len() > 2 || caps.len() + 1 != faces.len() {
        return None;
    }

    let tan_angle = payload.semi_angle.tan();
    if tan_angle.abs() <= 1.0e-12 {
        return None;
    }

    let mut axial_radii = caps
        .iter()
        .map(|plane| {
            let axial = dot3(subtract3(plane.origin, payload.origin), axis);
            let radius = (payload.reference_radius + axial * tan_angle).abs();
            (axial, radius)
        })
        .collect::<Vec<_>>();

    if axial_radii.len() == 1 {
        axial_radii.push((-payload.reference_radius / tan_angle, 0.0));
    }
    if axial_radii.len() != 2 {
        return None;
    }

    let (axial0, radius0) = axial_radii[0];
    let (axial1, radius1) = axial_radii[1];
    let height = (axial0 - axial1).abs();
    let slant = ((radius0 - radius1).powi(2) + height.powi(2)).sqrt();

    Some(ExactPrimitiveSummary {
        surface_area: PI * (radius0 + radius1) * slant
            + PI * (radius0 * radius0 + radius1 * radius1),
        volume: PI * height * (radius0 * radius0 + radius0 * radius1 + radius1 * radius1) / 3.0,
    })
}

fn exact_sphere_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let (payload, _) = single_sphere_face(faces)?;
    let radius = payload.radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 4.0 * PI * radius * radius,
        volume: 4.0 * PI * radius * radius * radius / 3.0,
    })
}

fn exact_torus_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let (payload, _) = single_torus_face(faces)?;
    let major_radius = payload.major_radius.abs();
    let minor_radius = payload.minor_radius.abs();
    Some(ExactPrimitiveSummary {
        surface_area: 4.0 * PI * PI * major_radius * minor_radius,
        volume: 2.0 * PI * PI * major_radius * minor_radius * minor_radius,
    })
}

fn exact_translational_solid_summary(faces: &[BrepFace]) -> Option<ExactPrimitiveSummary> {
    let plane_faces = faces
        .iter()
        .filter_map(|face| match face.ported_surface {
            Some(PortedSurface::Plane(payload)) => Some((face, payload)),
            _ => None,
        })
        .collect::<Vec<_>>();
    if plane_faces.len() < 2 {
        return None;
    }

    for i in 0..plane_faces.len() {
        for j in i + 1..plane_faces.len() {
            let (lhs_face, lhs_plane) = plane_faces[i];
            let (rhs_face, rhs_plane) = plane_faces[j];
            let lhs_normal = normalize3(lhs_plane.normal);
            let rhs_normal = normalize3(rhs_plane.normal);
            if dot3(lhs_normal, rhs_normal) > -1.0 + 1.0e-6 {
                continue;
            }
            if !approx_eq(lhs_face.area, rhs_face.area, 1.0e-6, 1.0e-6) {
                continue;
            }
            if !pair_forms_translational_caps(faces, lhs_face.index, rhs_face.index) {
                continue;
            }

            let span = dot3(subtract3(rhs_plane.origin, lhs_plane.origin), lhs_normal).abs();
            if span <= 1.0e-9 {
                continue;
            }

            return Some(ExactPrimitiveSummary {
                surface_area: faces.iter().map(|face| face.area).sum(),
                volume: lhs_face.area * span,
            });
        }
    }

    None
}

fn single_cylinder_face(faces: &[BrepFace]) -> Option<(CylinderPayload, usize)> {
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Cylinder(payload) => Some(payload),
        _ => None,
    })
}

fn single_cone_face(faces: &[BrepFace]) -> Option<(ConePayload, usize)> {
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Cone(payload) => Some(payload),
        _ => None,
    })
}

fn single_sphere_face(faces: &[BrepFace]) -> Option<(SpherePayload, usize)> {
    if faces.len() != 1 {
        return None;
    }
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Sphere(payload) => Some(payload),
        _ => None,
    })
}

fn single_torus_face(faces: &[BrepFace]) -> Option<(TorusPayload, usize)> {
    if faces.len() != 1 {
        return None;
    }
    single_surface_face(faces, |surface| match surface {
        PortedSurface::Torus(payload) => Some(payload),
        _ => None,
    })
}

fn single_surface_face<T>(
    faces: &[BrepFace],
    extract: impl Fn(PortedSurface) -> Option<T>,
) -> Option<(T, usize)> {
    let mut found = None;
    for face in faces {
        let Some(surface) = face.ported_surface else {
            return None;
        };
        let Some(payload) = extract(surface) else {
            continue;
        };
        if found.is_some() {
            return None;
        }
        found = Some((payload, face.index));
    }
    found
}

fn aligned_plane_faces(faces: &[BrepFace], axis: [f64; 3]) -> Vec<PlanePayload> {
    faces
        .iter()
        .filter_map(|face| match face.ported_surface {
            Some(PortedSurface::Plane(payload))
                if dot3(normalize3(payload.normal), axis).abs() >= 1.0 - 1.0e-6 =>
            {
                Some(payload)
            }
            _ => None,
        })
        .collect()
}

fn pair_forms_translational_caps(faces: &[BrepFace], lhs_index: usize, rhs_index: usize) -> bool {
    faces.iter().all(|candidate| {
        if candidate.index == lhs_index || candidate.index == rhs_index {
            return true;
        }
        candidate.adjacent_face_indices.contains(&lhs_index)
            && candidate.adjacent_face_indices.contains(&rhs_index)
    })
}

fn incident_edge_vector(edge: &BrepEdge, vertex_index: usize) -> Option<[f64; 3]> {
    match (
        edge.start_vertex,
        edge.end_vertex,
        edge.start_point,
        edge.end_point,
    ) {
        (Some(start), Some(_), Some(start_point), Some(end_point)) if start == vertex_index => {
            Some(subtract3(end_point, start_point))
        }
        (Some(_), Some(end), Some(start_point), Some(end_point)) if end == vertex_index => {
            Some(subtract3(start_point, end_point))
        }
        _ => None,
    }
}

fn analytic_shape_volume(
    context: &Context,
    wires: &[BrepWire],
    edges: &[BrepEdge],
    faces: &[BrepFace],
    face_shapes: &[Shape],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if faces.is_empty() {
        return Some(0.0);
    }

    let mut volume = 0.0;
    for face in faces {
        let contribution = match face.ported_surface {
            Some(surface) => analytic_face_volume(
                context,
                face,
                surface,
                face.geometry,
                &face.loops,
                wires,
                edges,
                edge_shapes,
            ),
            None => analytic_swept_face_volume(
                context,
                face_shapes.get(face.index)?,
                face,
                face.geometry,
                &face.loops,
                wires,
                edges,
            ),
        }?;
        volume += contribution;
    }
    Some(volume.abs())
}

fn mesh_shape_volume(context: &Context, shape: &Shape, counts: ShapeCounts) -> Option<f64> {
    if counts.solid_count == 0 && counts.compsolid_count == 0 {
        return Some(0.0);
    }

    let mesh = context.mesh(shape, SUMMARY_VOLUME_MESH_PARAMS).ok()?;
    polyhedral_mesh_volume(&mesh)
}

fn ported_shape_bbox(
    context: &Context,
    shape: &Shape,
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    topological_shape_bbox(vertices, edges, faces).or_else(|| mesh_shape_bbox(context, shape))
}

fn shape_counts(
    context: &Context,
    shape: &Shape,
    topology: &TopologySnapshot,
) -> Result<ShapeCounts, Error> {
    Ok(ShapeCounts {
        compound_count: context.subshape_count(shape, ShapeKind::Compound)?,
        compsolid_count: context.subshape_count(shape, ShapeKind::CompSolid)?,
        solid_count: context.subshape_count(shape, ShapeKind::Solid)?,
        shell_count: context.subshape_count(shape, ShapeKind::Shell)?,
        face_count: topology.faces.len(),
        wire_count: topology.wires.len(),
        edge_count: topology.edges.len(),
        vertex_count: topology.vertex_positions.len(),
    })
}

fn classify_root_kind(counts: ShapeCounts) -> ShapeKind {
    if counts.compound_count > 0 {
        ShapeKind::Compound
    } else if counts.compsolid_count > 0 {
        ShapeKind::CompSolid
    } else if counts.solid_count > 0 {
        ShapeKind::Solid
    } else if counts.shell_count > 0 {
        ShapeKind::Shell
    } else if counts.face_count > 0 {
        ShapeKind::Face
    } else if counts.wire_count > 0 {
        ShapeKind::Wire
    } else if counts.edge_count > 0 {
        ShapeKind::Edge
    } else if counts.vertex_count > 0 {
        ShapeKind::Vertex
    } else {
        ShapeKind::Unknown
    }
}

fn classify_primary_kind(counts: ShapeCounts) -> ShapeKind {
    if counts.solid_count > 0 {
        ShapeKind::Solid
    } else if counts.shell_count > 0 {
        ShapeKind::Shell
    } else if counts.face_count > 0 {
        ShapeKind::Face
    } else if counts.wire_count > 0 {
        ShapeKind::Wire
    } else if counts.edge_count > 0 {
        ShapeKind::Edge
    } else if counts.vertex_count > 0 {
        ShapeKind::Vertex
    } else if counts.compsolid_count > 0 {
        ShapeKind::CompSolid
    } else if counts.compound_count > 0 {
        ShapeKind::Compound
    } else {
        ShapeKind::Unknown
    }
}

fn topological_shape_bbox(
    vertices: &[BrepVertex],
    edges: &[BrepEdge],
    faces: &[BrepFace],
) -> Option<([f64; 3], [f64; 3])> {
    let use_topology_points = if !faces.is_empty() {
        !vertices.is_empty()
            && faces
                .iter()
                .all(|face| matches!(face.ported_surface, Some(PortedSurface::Plane(_))))
            && edges
                .iter()
                .all(|edge| matches!(edge.ported_curve, Some(PortedCurve::Line(_))))
    } else {
        !edges.is_empty()
            && edges.iter().all(|edge| {
                matches!(edge.ported_curve, Some(PortedCurve::Line(_)))
                    && edge.start_point.is_some()
                    && edge.end_point.is_some()
            })
    };

    if !use_topology_points {
        return None;
    }

    let mut points = vertices
        .iter()
        .map(|vertex| vertex.position)
        .collect::<Vec<_>>();
    for edge in edges {
        if let Some(start_point) = edge.start_point {
            points.push(start_point);
        }
        if let Some(end_point) = edge.end_point {
            points.push(end_point);
        }
    }

    bbox_from_points(points)
}

fn mesh_shape_bbox(context: &Context, shape: &Shape) -> Option<([f64; 3], [f64; 3])> {
    let mesh = context.mesh(shape, SUMMARY_BBOX_MESH_PARAMS).ok()?;
    mesh_bbox(&mesh)
}

fn mesh_face_area(context: &Context, face_shape: &Shape) -> Option<f64> {
    let mesh = context
        .mesh(face_shape, UNSUPPORTED_FACE_AREA_MESH_PARAMS)
        .ok()?;
    polyhedral_mesh_area(&mesh)
}

fn analytic_face_area(
    context: &Context,
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if loops.is_empty() {
        return match surface {
            PortedSurface::Sphere(payload) => Some(4.0 * PI * payload.radius.abs().powi(2)),
            PortedSurface::Torus(payload) => {
                Some(4.0 * PI * PI * payload.major_radius.abs() * payload.minor_radius.abs())
            }
            _ => Some(0.0),
        };
    }

    let plane = match surface {
        PortedSurface::Plane(plane) => Some(plane),
        _ => None,
    };

    let mut area = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut curve_segments = Vec::with_capacity(wire.edge_indices.len());
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            let edge = edges.get(edge_index)?;
            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            if let Some(curve) = edge.ported_curve {
                curve_segments.push((curve, geometry));
            }
            append_edge_sample_points(
                context,
                edge_shapes.get(edge_index)?,
                edge,
                geometry,
                &mut sampled_points,
            )
            .ok()?;
        }

        let wire_area = match plane {
            Some(plane) if curve_segments.len() == wire.edge_indices.len() => {
                planar_wire_signed_area(plane, &curve_segments).abs()
            }
            Some(_) => {
                analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs()
            }
            None => {
                analytic_sampled_wire_signed_area(surface, face_geometry, &sampled_points)?.abs()
            }
        };
        match face_loop.role {
            LoopRole::Inner => area -= wire_area,
            LoopRole::Outer | LoopRole::Unknown => area += wire_area,
        }
    }
    Some(area.abs())
}

fn analytic_swept_face_area(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
) -> Option<f64> {
    match face_geometry.kind {
        crate::SurfaceKind::Extrusion => {
            let payload = context.face_extrusion_payload(face_shape).ok()?;
            let candidates = face_curve_candidates(loops, wires, edges, payload.basis_curve_kind)?;
            let basis = *candidates.first()?;
            let span = extrusion_span(&candidates, payload.direction)?;
            Some(extrusion_swept_area(
                basis.curve,
                basis.geometry,
                payload.direction,
                span,
            ))
        }
        crate::SurfaceKind::Revolution => {
            let payload = context.face_revolution_payload(face_shape).ok()?;
            let candidates = face_curve_candidates(loops, wires, edges, payload.basis_curve_kind)?;
            let basis = *candidates.first()?;
            let sweep_angle = revolution_sweep_angle(
                &candidates,
                face_geometry,
                payload.axis_origin,
                payload.axis_direction,
            )?;
            Some(revolution_swept_area(
                basis.curve,
                basis.geometry,
                payload.axis_origin,
                payload.axis_direction,
                sweep_angle,
            ))
        }
        _ => None,
    }
}

fn analytic_swept_face_sample(
    context: &Context,
    face_shape: &Shape,
    face_geometry: FaceGeometry,
    orientation: Orientation,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
) -> Option<FaceSample> {
    match face_geometry.kind {
        crate::SurfaceKind::Extrusion => {
            let payload = context.face_extrusion_payload(face_shape).ok()?;
            let basis = select_swept_face_basis_curve(
                face_curve_candidates(loops, wires, edges, payload.basis_curve_kind)?,
                face_geometry,
                SweptBasisSelection::Extrusion {
                    direction: payload.direction,
                },
            )?;
            Some(sample_extrusion_surface_normalized(
                basis.curve,
                face_geometry,
                basis.geometry,
                [0.5, 0.5],
                payload.direction,
                orientation,
            ))
        }
        crate::SurfaceKind::Revolution => {
            let payload = context.face_revolution_payload(face_shape).ok()?;
            let basis = select_swept_face_basis_curve(
                face_curve_candidates(loops, wires, edges, payload.basis_curve_kind)?,
                face_geometry,
                SweptBasisSelection::Revolution {
                    axis_origin: payload.axis_origin,
                    axis_direction: payload.axis_direction,
                },
            )?;
            Some(sample_revolution_surface_normalized(
                basis.curve,
                face_geometry,
                basis.geometry,
                [0.5, 0.5],
                payload.axis_origin,
                payload.axis_direction,
                orientation,
            ))
        }
        _ => None,
    }
}

fn analytic_swept_face_volume(
    context: &Context,
    face_shape: &Shape,
    face: &BrepFace,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
) -> Option<f64> {
    match face_geometry.kind {
        crate::SurfaceKind::Extrusion => {
            let payload = context.face_extrusion_payload(face_shape).ok()?;
            let candidates = face_curve_candidates(loops, wires, edges, payload.basis_curve_kind)?;
            let basis = *candidates.first()?;
            let sweep = scale3(
                normalize3(payload.direction),
                extrusion_span(&candidates, payload.direction)?,
            );
            let midpoint_parameter =
                0.5 * (basis.geometry.start_parameter + basis.geometry.end_parameter);
            let midpoint = basis.curve.evaluate(midpoint_parameter);
            let midpoint_position = add3(midpoint.position, scale3(sweep, 0.5));
            let midpoint_du = midpoint.derivative;
            let midpoint_dv = sweep;
            let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);
            Some(
                sign * signed_scalar_integral(
                    basis.geometry.start_parameter,
                    basis.geometry.end_parameter,
                    |parameter| {
                        let evaluation = basis.curve.evaluate(parameter);
                        dot3(evaluation.position, cross3(evaluation.derivative, sweep)) / 3.0
                    },
                ),
            )
        }
        crate::SurfaceKind::Revolution => {
            let payload = context.face_revolution_payload(face_shape).ok()?;
            let candidates = face_curve_candidates(loops, wires, edges, payload.basis_curve_kind)?;
            let (basis, sweep_angle) = revolution_basis_and_sweep(
                &candidates,
                face_geometry,
                payload.axis_origin,
                payload.axis_direction,
            )?;
            let midpoint_parameter =
                0.5 * (basis.geometry.start_parameter + basis.geometry.end_parameter);
            let midpoint_evaluation = basis.curve.evaluate(midpoint_parameter);
            let midpoint_position = rotate_point_about_axis(
                midpoint_evaluation.position,
                payload.axis_origin,
                payload.axis_direction,
                0.5 * sweep_angle,
            );
            let midpoint_du = rotate_vector_about_axis(
                midpoint_evaluation.derivative,
                payload.axis_direction,
                0.5 * sweep_angle,
            );
            let midpoint_dv = revolution_surface_dv(
                midpoint_position,
                payload.axis_origin,
                payload.axis_direction,
            );
            let sign = oriented_surface_sign(face, midpoint_position, midpoint_du, midpoint_dv);

            Some(
                sign * signed_scalar_integral(
                    basis.geometry.start_parameter,
                    basis.geometry.end_parameter,
                    |parameter| {
                        let evaluation = basis.curve.evaluate(parameter);
                        signed_scalar_integral(0.0, sweep_angle, |angle| {
                            let position = rotate_point_about_axis(
                                evaluation.position,
                                payload.axis_origin,
                                payload.axis_direction,
                                angle,
                            );
                            let du = rotate_vector_about_axis(
                                evaluation.derivative,
                                payload.axis_direction,
                                angle,
                            );
                            let dv = revolution_surface_dv(
                                position,
                                payload.axis_origin,
                                payload.axis_direction,
                            );
                            dot3(position, cross3(du, dv)) / 3.0
                        })
                    },
                ),
            )
        }
        _ => None,
    }
}

fn analytic_face_volume(
    context: &Context,
    face: &BrepFace,
    surface: PortedSurface,
    face_geometry: FaceGeometry,
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    edge_shapes: &[Shape],
) -> Option<f64> {
    if matches!(surface, PortedSurface::Plane(_)) {
        return Some(face.area * dot3(face.sample.position, face.sample.normal) / 3.0);
    }

    let mut volume = 0.0;
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        let mut sampled_points = Vec::new();
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            let edge = edges.get(edge_index)?;
            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            append_edge_sample_points(
                context,
                edge_shapes.get(edge_index)?,
                edge,
                geometry,
                &mut sampled_points,
            )
            .ok()?;
        }
        let loop_volume =
            analytic_sampled_wire_signed_volume(surface, face_geometry, &sampled_points)?;
        match face_loop.role {
            LoopRole::Inner => volume -= loop_volume,
            LoopRole::Outer | LoopRole::Unknown => volume += loop_volume,
        }
    }
    Some(volume)
}

fn oriented_surface_sign(face: &BrepFace, position: [f64; 3], du: [f64; 3], dv: [f64; 3]) -> f64 {
    let _ = position;
    let normal = normalize3(cross3(du, dv));
    if dot3(normal, face.sample.normal) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

fn face_curve_candidates(
    loops: &[BrepFaceLoop],
    wires: &[BrepWire],
    edges: &[BrepEdge],
    basis_kind: crate::CurveKind,
) -> Option<Vec<FaceCurveCandidate>> {
    let mut seen = BTreeSet::new();
    let mut candidates = Vec::new();

    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index)?;
        for (edge_index, edge_orientation) in oriented_wire_edges(wire, face_loop.orientation) {
            if !seen.insert(edge_index) {
                continue;
            }
            let edge = edges.get(edge_index)?;
            if edge.geometry.kind != basis_kind {
                continue;
            }
            let curve = edge.ported_curve?;

            let geometry = oriented_edge_geometry(edge.geometry, edge_orientation);
            let midpoint_parameter = 0.5 * (geometry.start_parameter + geometry.end_parameter);
            let midpoint = curve
                .sample_with_geometry(geometry, midpoint_parameter)
                .position;
            candidates.push(FaceCurveCandidate {
                curve,
                geometry,
                midpoint,
            });
        }
    }

    if candidates.is_empty() {
        None
    } else {
        Some(candidates)
    }
}

fn select_swept_face_basis_curve(
    candidates: Vec<FaceCurveCandidate>,
    face_geometry: FaceGeometry,
    selection: SweptBasisSelection,
) -> Option<FaceCurveCandidate> {
    let basis_geometry = candidates.first()?.geometry;
    let use_u_for_basis = basis_parameter_on_u(face_geometry, basis_geometry);
    let (sweep_min, sweep_max) = if use_u_for_basis {
        (face_geometry.v_min, face_geometry.v_max)
    } else {
        (face_geometry.u_min, face_geometry.u_max)
    };
    let target_is_min = sweep_min.abs() <= sweep_max.abs();

    match selection {
        SweptBasisSelection::Extrusion { direction } => {
            let direction = normalize3(direction);
            if target_is_min {
                candidates.into_iter().min_by(|lhs, rhs| {
                    dot3(lhs.midpoint, direction)
                        .partial_cmp(&dot3(rhs.midpoint, direction))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            } else {
                candidates.into_iter().max_by(|lhs, rhs| {
                    dot3(lhs.midpoint, direction)
                        .partial_cmp(&dot3(rhs.midpoint, direction))
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
            }
        }
        SweptBasisSelection::Revolution {
            axis_origin,
            axis_direction,
        } => {
            if periodic_face_span(face_geometry).is_some() {
                return candidates.into_iter().next();
            }
            let axis_direction = normalize3(axis_direction);
            let reference_radial = candidates.iter().find_map(|candidate| {
                radial_direction(candidate.midpoint, axis_origin, axis_direction)
            })?;
            let tangent = normalize3(cross3(axis_direction, reference_radial));
            let angular_candidates = candidates
                .into_iter()
                .filter_map(|candidate| {
                    let radial = radial_direction(candidate.midpoint, axis_origin, axis_direction)?;
                    Some((
                        candidate,
                        dot3(radial, tangent).atan2(dot3(radial, reference_radial)),
                    ))
                })
                .collect::<Vec<_>>();

            if target_is_min {
                angular_candidates
                    .into_iter()
                    .min_by(|lhs, rhs| {
                        lhs.1
                            .partial_cmp(&rhs.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(candidate, _)| candidate)
            } else {
                angular_candidates
                    .into_iter()
                    .max_by(|lhs, rhs| {
                        lhs.1
                            .partial_cmp(&rhs.1)
                            .unwrap_or(std::cmp::Ordering::Equal)
                    })
                    .map(|(candidate, _)| candidate)
            }
        }
    }
}

#[derive(Clone, Copy)]
enum SweptBasisSelection {
    Extrusion {
        direction: [f64; 3],
    },
    Revolution {
        axis_origin: [f64; 3],
        axis_direction: [f64; 3],
    },
}

fn basis_parameter_on_u(face_geometry: FaceGeometry, basis_geometry: EdgeGeometry) -> bool {
    let basis_span = (basis_geometry.end_parameter - basis_geometry.start_parameter).abs();
    let u_span = (face_geometry.u_max - face_geometry.u_min).abs();
    let v_span = (face_geometry.v_max - face_geometry.v_min).abs();
    (u_span - basis_span).abs() <= (v_span - basis_span).abs()
}

fn extrusion_span(candidates: &[FaceCurveCandidate], direction: [f64; 3]) -> Option<f64> {
    if candidates.len() < 2 {
        return None;
    }

    let direction = normalize3(direction);
    let mut min_projection = f64::INFINITY;
    let mut max_projection = f64::NEG_INFINITY;
    for candidate in candidates {
        let projection = dot3(candidate.midpoint, direction);
        min_projection = min_projection.min(projection);
        max_projection = max_projection.max(projection);
    }

    let span = max_projection - min_projection;
    if span <= 1.0e-9 {
        None
    } else {
        Some(span)
    }
}

fn revolution_sweep_angle(
    candidates: &[FaceCurveCandidate],
    face_geometry: FaceGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<f64> {
    if let Some(span) = periodic_face_span(face_geometry) {
        return Some(span.abs());
    }
    if candidates.len() < 2 {
        return None;
    }

    let axis_direction = normalize3(axis_direction);
    let reference_radial = candidates.iter().find_map(|candidate| {
        let radial = subtract3(
            candidate.midpoint,
            add3(
                axis_origin,
                scale3(
                    axis_direction,
                    dot3(subtract3(candidate.midpoint, axis_origin), axis_direction),
                ),
            ),
        );
        if norm3(radial) > 1.0e-9 {
            Some(normalize3(radial))
        } else {
            None
        }
    })?;
    let tangent = normalize3(cross3(axis_direction, reference_radial));

    let mut min_angle = 0.0;
    let mut max_angle = 0.0;
    let mut initialized = false;
    for candidate in candidates {
        let radial = subtract3(
            candidate.midpoint,
            add3(
                axis_origin,
                scale3(
                    axis_direction,
                    dot3(subtract3(candidate.midpoint, axis_origin), axis_direction),
                ),
            ),
        );
        if norm3(radial) <= 1.0e-9 {
            continue;
        }
        let angle = dot3(radial, tangent).atan2(dot3(radial, reference_radial));
        if !initialized {
            min_angle = angle;
            max_angle = angle;
            initialized = true;
        } else {
            min_angle = min_angle.min(angle);
            max_angle = max_angle.max(angle);
        }
    }

    if !initialized || (max_angle - min_angle).abs() <= 1.0e-9 {
        None
    } else {
        Some(max_angle - min_angle)
    }
}

fn revolution_basis_and_sweep(
    candidates: &[FaceCurveCandidate],
    face_geometry: FaceGeometry,
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<(FaceCurveCandidate, f64)> {
    if let Some(span) = periodic_face_span(face_geometry) {
        return Some((*candidates.first()?, span.abs()));
    }
    if candidates.len() < 2 {
        return None;
    }

    let axis_direction = normalize3(axis_direction);
    let reference_radial = candidates
        .iter()
        .find_map(|candidate| radial_direction(candidate.midpoint, axis_origin, axis_direction))?;
    let tangent = normalize3(cross3(axis_direction, reference_radial));

    let mut angular_candidates = candidates
        .iter()
        .copied()
        .filter_map(|candidate| {
            let radial = radial_direction(candidate.midpoint, axis_origin, axis_direction)?;
            Some((
                candidate,
                dot3(radial, tangent).atan2(dot3(radial, reference_radial)),
            ))
        })
        .collect::<Vec<_>>();
    angular_candidates.sort_by(|lhs, rhs| lhs.1.total_cmp(&rhs.1));

    let (basis, min_angle) = *angular_candidates.first()?;
    let (_, max_angle) = *angular_candidates.last()?;
    let sweep = max_angle - min_angle;
    if sweep.abs() <= 1.0e-9 {
        None
    } else {
        Some((basis, sweep))
    }
}

fn radial_direction(
    point: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> Option<[f64; 3]> {
    let radial = subtract3(
        point,
        add3(
            axis_origin,
            scale3(
                axis_direction,
                dot3(subtract3(point, axis_origin), axis_direction),
            ),
        ),
    );
    (norm3(radial) > 1.0e-9).then_some(normalize3(radial))
}

fn rotate_point_about_axis(
    point: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
    angle: f64,
) -> [f64; 3] {
    add3(
        axis_origin,
        rotate_vector_about_axis(subtract3(point, axis_origin), axis_direction, angle),
    )
}

fn rotate_vector_about_axis(vector: [f64; 3], axis_direction: [f64; 3], angle: f64) -> [f64; 3] {
    let axis_direction = normalize3(axis_direction);
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    add3(
        add3(
            scale3(vector, cos_angle),
            scale3(cross3(axis_direction, vector), sin_angle),
        ),
        scale3(
            axis_direction,
            dot3(axis_direction, vector) * (1.0 - cos_angle),
        ),
    )
}

fn revolution_surface_dv(
    position: [f64; 3],
    axis_origin: [f64; 3],
    axis_direction: [f64; 3],
) -> [f64; 3] {
    cross3(normalize3(axis_direction), subtract3(position, axis_origin))
}

fn signed_scalar_integral<F>(start: f64, end: f64, integrand: F) -> f64
where
    F: Fn(f64) -> f64,
{
    if (end - start).abs() <= 1.0e-15 {
        return 0.0;
    }

    let (a, b, sign) = if start <= end {
        (start, end, 1.0)
    } else {
        (end, start, -1.0)
    };
    let fa = integrand(a);
    let fm = integrand(0.5 * (a + b));
    let fb = integrand(b);
    sign * adaptive_simpson(&integrand, a, b, fa, fm, fb, 1.0e-8, 12)
}

fn adaptive_simpson<F>(
    integrand: &F,
    a: f64,
    b: f64,
    fa: f64,
    fm: f64,
    fb: f64,
    tolerance: f64,
    depth: u32,
) -> f64
where
    F: Fn(f64) -> f64,
{
    let midpoint = 0.5 * (a + b);
    let left_mid = 0.5 * (a + midpoint);
    let right_mid = 0.5 * (midpoint + b);
    let flm = integrand(left_mid);
    let frm = integrand(right_mid);

    let whole = simpson_step(a, b, fa, fm, fb);
    let left = simpson_step(a, midpoint, fa, flm, fm);
    let right = simpson_step(midpoint, b, fm, frm, fb);
    let delta = left + right - whole;

    if depth == 0 || delta.abs() <= 15.0 * tolerance {
        return left + right + delta / 15.0;
    }

    adaptive_simpson(
        integrand,
        a,
        midpoint,
        fa,
        flm,
        fm,
        0.5 * tolerance,
        depth - 1,
    ) + adaptive_simpson(
        integrand,
        midpoint,
        b,
        fm,
        frm,
        fb,
        0.5 * tolerance,
        depth - 1,
    )
}

fn simpson_step(a: f64, b: f64, fa: f64, fm: f64, fb: f64) -> f64 {
    (b - a) * (fa + 4.0 * fm + fb) / 6.0
}

fn periodic_face_span(face_geometry: FaceGeometry) -> Option<f64> {
    if face_geometry.is_u_periodic && !face_geometry.is_v_periodic {
        let span = face_geometry.u_max - face_geometry.u_min;
        return (span.abs() > 1.0e-9).then_some(span.abs());
    }
    if face_geometry.is_v_periodic && !face_geometry.is_u_periodic {
        let span = face_geometry.v_max - face_geometry.v_min;
        return (span.abs() > 1.0e-9).then_some(span.abs());
    }
    None
}

fn append_edge_sample_points(
    context: &Context,
    edge_shape: &Shape,
    edge: &BrepEdge,
    geometry: EdgeGeometry,
    out_points: &mut Vec<[f64; 3]>,
) -> Result<(), Error> {
    let segment_count = edge_sample_count(edge, geometry);
    for step in 0..=segment_count {
        if !out_points.is_empty() && step == 0 {
            continue;
        }
        let t = step as f64 / segment_count as f64;
        let parameter = interpolate_range(geometry.start_parameter, geometry.end_parameter, t);
        let position = match edge.ported_curve {
            Some(curve) => curve.sample_with_geometry(geometry, parameter).position,
            None => {
                context
                    .edge_sample_at_parameter(edge_shape, parameter)?
                    .position
            }
        };
        out_points.push(position);
    }
    Ok(())
}

fn oriented_wire_edges(
    wire: &BrepWire,
    wire_orientation: Orientation,
) -> Vec<(usize, Orientation)> {
    let reverse_wire = matches!(wire_orientation, Orientation::Reversed);
    let mut uses = wire
        .edge_indices
        .iter()
        .copied()
        .zip(wire.edge_orientations.iter().copied())
        .collect::<Vec<_>>();
    if reverse_wire {
        uses.reverse();
        for (_, orientation) in &mut uses {
            *orientation = reversed_orientation(*orientation);
        }
    }
    uses
}

fn reversed_orientation(orientation: Orientation) -> Orientation {
    match orientation {
        Orientation::Forward => Orientation::Reversed,
        Orientation::Reversed => Orientation::Forward,
        other => other,
    }
}

fn edge_sample_count(edge: &BrepEdge, geometry: EdgeGeometry) -> usize {
    let span = (geometry.end_parameter - geometry.start_parameter).abs();
    let base = match edge.geometry.kind {
        crate::CurveKind::Line => 8,
        crate::CurveKind::Circle | crate::CurveKind::Ellipse => {
            (span / (std::f64::consts::TAU / 32.0)).ceil() as usize
        }
        _ => 48,
    };
    base.clamp(8, 256)
}

fn interpolate_range(start: f64, end: f64, t: f64) -> f64 {
    start + (end - start) * t
}

fn polyhedral_mesh_volume(mesh: &Mesh) -> Option<f64> {
    if mesh.triangle_indices.is_empty() {
        return Some(0.0);
    }

    let origin = [
        0.5 * (mesh.bbox_min[0] + mesh.bbox_max[0]),
        0.5 * (mesh.bbox_min[1] + mesh.bbox_max[1]),
        0.5 * (mesh.bbox_min[2] + mesh.bbox_max[2]),
    ];
    let mut signed_volume = 0.0;

    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;

        let face_cross = cross3(subtract3(b, a), subtract3(c, a));
        let face_cross_length = dot3(face_cross, face_cross).sqrt();
        if face_cross_length <= 1.0e-12 {
            continue;
        }

        let averaged_normal = add3(
            add3(
                mesh.normals.get(i0).copied().unwrap_or([0.0; 3]),
                mesh.normals.get(i1).copied().unwrap_or([0.0; 3]),
            ),
            mesh.normals.get(i2).copied().unwrap_or([0.0; 3]),
        );
        let outward_normal = if dot3(averaged_normal, averaged_normal) > 1.0e-18 {
            normalize3(averaged_normal)
        } else {
            let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
            let fallback_normal = normalize3(face_cross);
            if dot3(fallback_normal, subtract3(centroid, origin)) >= 0.0 {
                fallback_normal
            } else {
                scale3(fallback_normal, -1.0)
            }
        };
        let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
        let area = 0.5 * face_cross_length;
        signed_volume += area * dot3(subtract3(centroid, origin), outward_normal) / 3.0;
    }

    Some(signed_volume.abs())
}

fn polyhedral_mesh_area(mesh: &Mesh) -> Option<f64> {
    if mesh.triangle_indices.is_empty() {
        return Some(0.0);
    }

    let mut area = 0.0;
    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;
        area += 0.5 * norm3(cross3(subtract3(b, a), subtract3(c, a)));
    }

    Some(area)
}

fn mesh_bbox(mesh: &Mesh) -> Option<([f64; 3], [f64; 3])> {
    let mut points = mesh.positions.clone();
    for segment in &mesh.edge_segments {
        points.push(segment[0]);
        points.push(segment[1]);
    }
    bbox_from_points(points)
}

fn bbox_from_points(points: Vec<[f64; 3]>) -> Option<([f64; 3], [f64; 3])> {
    let mut iter = points.into_iter();
    let first = iter.next()?;
    let mut min = first;
    let mut max = first;

    for point in iter {
        for axis in 0..3 {
            min[axis] = min[axis].min(point[axis]);
            max[axis] = max[axis].max(point[axis]);
        }
    }

    Some((min, max))
}

fn add3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

fn subtract3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

fn scale3(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

fn dot3(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

fn cross3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

fn normalize3(vector: [f64; 3]) -> [f64; 3] {
    let length = dot3(vector, vector).sqrt();
    if length <= 1.0e-18 {
        [0.0; 3]
    } else {
        scale3(vector, length.recip())
    }
}

fn norm3(vector: [f64; 3]) -> f64 {
    dot3(vector, vector).sqrt()
}

fn approx_eq(lhs: f64, rhs: f64, relative_tolerance: f64, absolute_tolerance: f64) -> bool {
    let delta = (lhs - rhs).abs();
    if delta <= absolute_tolerance {
        return true;
    }
    let scale = lhs.abs().max(rhs.abs()).max(1.0);
    delta <= relative_tolerance * scale
}

fn oriented_edge_geometry(mut geometry: EdgeGeometry, orientation: Orientation) -> EdgeGeometry {
    if matches!(orientation, Orientation::Reversed) {
        std::mem::swap(&mut geometry.start_parameter, &mut geometry.end_parameter);
    }
    geometry
}

fn topology_edge(topology: &TopologySnapshot, index: usize) -> Result<crate::TopologyEdge, Error> {
    topology
        .edges
        .get(index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing edge index {index}")))
}

fn adjacent_face_indices(
    topology: &TopologySnapshot,
    edge_index: usize,
) -> Result<Vec<usize>, Error> {
    let range = topology
        .edge_faces
        .get(edge_index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing edge-face range {edge_index}")))?;
    Ok(topology.edge_face_indices[range.offset..range.offset + range.count].to_vec())
}

fn edge_points(
    topology: &TopologySnapshot,
    edge_index: usize,
) -> (Option<[f64; 3]>, Option<[f64; 3]>) {
    let Some(edge) = topology.edges.get(edge_index) else {
        return (None, None);
    };
    (
        optional_vertex_position(topology, edge.start_vertex),
        optional_vertex_position(topology, edge.end_vertex),
    )
}

fn optional_vertex_position(
    topology: &TopologySnapshot,
    vertex_index: Option<usize>,
) -> Option<[f64; 3]> {
    vertex_index.and_then(|index| topology.vertex_positions.get(index).copied())
}

fn face_loops(topology: &TopologySnapshot, face_index: usize) -> Result<Vec<BrepFaceLoop>, Error> {
    let range = topology
        .faces
        .get(face_index)
        .copied()
        .ok_or_else(|| Error::new(format!("topology is missing face range {face_index}")))?;
    let mut loops = Vec::with_capacity(range.count);
    for offset in range.offset..range.offset + range.count {
        loops.push(BrepFaceLoop {
            wire_index: topology
                .face_wire_indices
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing face-wire index {offset}"))
                })?,
            orientation: topology
                .face_wire_orientations
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!(
                        "topology is missing face-wire orientation {offset}"
                    ))
                })?,
            role: topology
                .face_wire_roles
                .get(offset)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing face-wire role {offset}"))
                })?,
        });
    }
    Ok(loops)
}

fn face_adjacent_face_indices(
    topology: &TopologySnapshot,
    wires: &[BrepWire],
    face_index: usize,
) -> Result<Vec<usize>, Error> {
    let loops = face_loops(topology, face_index)?;
    let mut adjacent = BTreeSet::new();
    for face_loop in loops {
        let wire = wires.get(face_loop.wire_index).ok_or_else(|| {
            Error::new(format!(
                "topology is missing wire index {}",
                face_loop.wire_index
            ))
        })?;
        for &edge_index in &wire.edge_indices {
            let range = topology
                .edge_faces
                .get(edge_index)
                .copied()
                .ok_or_else(|| {
                    Error::new(format!("topology is missing edge-face range {edge_index}"))
                })?;
            for &candidate in &topology.edge_face_indices[range.offset..range.offset + range.count]
            {
                if candidate != face_index {
                    adjacent.insert(candidate);
                }
            }
        }
    }
    Ok(adjacent.into_iter().collect())
}
