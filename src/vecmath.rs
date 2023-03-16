pub(crate) type Vector = [f32; 3];

pub(crate) fn div(vec: Vector, num: f32) -> Vector {
    vec.map(|v| v / num)
}

pub(crate) fn len_sq([a, b, c]: Vector) -> f32 {
    a * a + b * b + c * c
}

pub(crate) fn len(vec: Vector) -> f32 {
    len_sq(vec).sqrt()
}

pub(crate) fn diff([v1, v2, v3]: Vector, [u1, u2, u3]: Vector) -> Vector {
    [v1 - u1, v2 - u2, v3 - u3]
}

pub(crate) fn cross([a1, a2, a3]: Vector, [b1, b2, b3]: Vector) -> Vector {
    [a2 * b3 - a3 * b2, a3 * b1 - a1 * b3, a1 * b2 - a2 * b1]
}

pub(crate) fn add([u1, u2, u3]: Vector, [v1, v2, v3]: Vector) -> Vector {
    [u1 + v1, u2 + v2, u3 + v3]
}

pub(crate) fn mul(v: Vector, factor: f32) -> Vector {
    v.map(|v| v * factor)
}

pub(crate) fn dist_sq(p1: Vector, p2: Vector) -> f32 {
    len_sq(diff(p1, p2))
}

#[allow(unused)]
pub(crate) fn midpoint(pl: Vector, pr: Vector) -> Vector {
    div(add(pl, pr), 2.0)
}

pub(crate) fn dot([v1, v2, v3]: Vector, [u1, u2, u3]: Vector) -> f32 {
    v1 * u1 + v2 * u2 + v3 * u3
}
