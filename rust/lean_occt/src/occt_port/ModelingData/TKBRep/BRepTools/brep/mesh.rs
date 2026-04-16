use super::math::{add3, cross3, dot3, norm3, normalize3, scale3, subtract3};
use super::*;

pub(super) fn polyhedral_mesh_volume(mesh: &Mesh) -> Option<f64> {
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

pub(super) fn polyhedral_mesh_area(mesh: &Mesh) -> Option<f64> {
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

pub(super) fn polyhedral_mesh_sample(mesh: &Mesh) -> Option<FaceSample> {
    if mesh.positions.is_empty() {
        return None;
    }

    let mut weighted_area = 0.0;
    let mut weighted_centroid = [0.0; 3];
    let mut weighted_normal = [0.0; 3];

    for triangle in mesh.triangle_indices.chunks_exact(3) {
        let i0 = usize::try_from(triangle[0]).ok()?;
        let i1 = usize::try_from(triangle[1]).ok()?;
        let i2 = usize::try_from(triangle[2]).ok()?;
        let a = *mesh.positions.get(i0)?;
        let b = *mesh.positions.get(i1)?;
        let c = *mesh.positions.get(i2)?;
        let face_cross = cross3(subtract3(b, a), subtract3(c, a));
        let triangle_area = 0.5 * norm3(face_cross);
        if triangle_area <= 1.0e-12 {
            continue;
        }

        let averaged_normal = add3(
            add3(
                mesh.normals.get(i0).copied().unwrap_or([0.0; 3]),
                mesh.normals.get(i1).copied().unwrap_or([0.0; 3]),
            ),
            mesh.normals.get(i2).copied().unwrap_or([0.0; 3]),
        );
        let triangle_normal = if norm3(averaged_normal) > 1.0e-12 {
            normalize3(averaged_normal)
        } else {
            normalize3(face_cross)
        };
        let centroid = scale3(add3(add3(a, b), c), 1.0 / 3.0);
        weighted_area += triangle_area;
        weighted_centroid = add3(weighted_centroid, scale3(centroid, triangle_area));
        weighted_normal = add3(weighted_normal, scale3(triangle_normal, triangle_area));
    }

    if weighted_area > 1.0e-12 {
        return Some(FaceSample {
            position: scale3(weighted_centroid, weighted_area.recip()),
            normal: normalize3(weighted_normal),
        });
    }

    let position = scale3(
        mesh.positions.iter().copied().fold([0.0; 3], add3),
        (mesh.positions.len() as f64).recip(),
    );
    let normal = normalize3(mesh.normals.iter().copied().fold([0.0; 3], add3));
    Some(FaceSample { position, normal })
}

pub(super) fn mesh_bbox(mesh: &Mesh) -> Option<([f64; 3], [f64; 3])> {
    let mut points = mesh.positions.clone();
    for segment in &mesh.edge_segments {
        points.push(segment[0]);
        points.push(segment[1]);
    }
    bbox_from_points(points).or(Some((mesh.bbox_min, mesh.bbox_max)))
}

pub(super) fn bbox_from_points(points: Vec<[f64; 3]>) -> Option<([f64; 3], [f64; 3])> {
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

pub(super) fn union_bbox(
    lhs: ([f64; 3], [f64; 3]),
    rhs: ([f64; 3], [f64; 3]),
) -> ([f64; 3], [f64; 3]) {
    let mut min = lhs.0;
    let mut max = lhs.1;
    for axis in 0..3 {
        min[axis] = min[axis].min(rhs.0[axis]);
        max[axis] = max[axis].max(rhs.1[axis]);
    }
    (min, max)
}
