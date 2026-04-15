pub(super) fn add3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] + rhs[0], lhs[1] + rhs[1], lhs[2] + rhs[2]]
}

pub(super) fn subtract3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [lhs[0] - rhs[0], lhs[1] - rhs[1], lhs[2] - rhs[2]]
}

pub(super) fn scale3(vector: [f64; 3], factor: f64) -> [f64; 3] {
    [vector[0] * factor, vector[1] * factor, vector[2] * factor]
}

pub(super) fn dot3(lhs: [f64; 3], rhs: [f64; 3]) -> f64 {
    lhs[0] * rhs[0] + lhs[1] * rhs[1] + lhs[2] * rhs[2]
}

pub(super) fn cross3(lhs: [f64; 3], rhs: [f64; 3]) -> [f64; 3] {
    [
        lhs[1] * rhs[2] - lhs[2] * rhs[1],
        lhs[2] * rhs[0] - lhs[0] * rhs[2],
        lhs[0] * rhs[1] - lhs[1] * rhs[0],
    ]
}

pub(super) fn normalize3(vector: [f64; 3]) -> [f64; 3] {
    let length = dot3(vector, vector).sqrt();
    if length <= 1.0e-18 {
        [0.0; 3]
    } else {
        scale3(vector, length.recip())
    }
}

pub(super) fn norm3(vector: [f64; 3]) -> f64 {
    dot3(vector, vector).sqrt()
}

pub(super) fn approx_eq(
    lhs: f64,
    rhs: f64,
    relative_tolerance: f64,
    absolute_tolerance: f64,
) -> bool {
    let delta = (lhs - rhs).abs();
    if delta <= absolute_tolerance {
        return true;
    }
    let scale = lhs.abs().max(rhs.abs()).max(1.0);
    delta <= relative_tolerance * scale
}
