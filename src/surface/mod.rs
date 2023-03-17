pub mod poisson_disk;
pub mod uniform;

use crate::vecmath as m;
use rand_distr::Distribution;
use thiserror::Error;

#[derive(Error, Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum MeshRandError {
    #[error("failed to initialize: {0}")]
    Initialization(String),
}

#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct Triangle {
    ///The points that make up the triangle
    pub points: [m::Vector; 3],
    /// Normalized normal vector of the triangle the point resides in.
    ///
    /// OBS: The normal will be pointing out of the positively oriented side of the triangle. As
    /// an example, the triangle
    /// defined by the verticies `[a, b, c]` where `a = [0.0, 0.0, 0.0]`, `b = [1.0, 0.0, 0.0]` and
    /// `c = [0.0, 1.0, 0.0]` has the normal `[0.0, 0.0, 1.0]`. While a triangle defined by
    /// `[b, a, c]` has the normal `[0.0, 0.0, -1.0]`.
    pub normal: m::Vector,
    ///Triangle area
    pub area: f32,
    origin: m::Vector,
    u: m::Vector,
    v: m::Vector,
}

impl Triangle {
    pub fn from_points(p1: m::Vector, p2: m::Vector, p3: m::Vector) -> Result<Self, MeshRandError> {
        let origin = p1;
        let u = m::diff(p2, p1);
        let v = m::diff(p3, p1);
        let normal_dir = m::cross(u, v);
        let len = m::len(normal_dir);
        let area = len / 2.0;
        if !f32::is_normal(area) {
            return Err(MeshRandError::Initialization(
                "area of triangle too close to 0 (f32::is_normal(area) == false)".to_string(),
            ));
        }
        let normal = m::div(normal_dir, len);
        Ok(Triangle {
            points: [p1, p2, p3],
            origin,
            u,
            v,
            normal,
            area,
        })
    }

    fn intersects_sphere(&self, position: m::Vector, r: f32) -> bool {
        assert!(m::dist_sq(self.points[0], self.points[1]) <= r * r);
        assert!(m::dist_sq(self.points[1], self.points[2]) <= r * r);
        assert!(m::dist_sq(self.points[2], self.points[0]) <= r * r);
        self.points
            .iter()
            .any(|&p| m::dist_sq(position, p) <= r * r)
    }
}

impl Distribution<m::Vector> for Triangle {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> m::Vector {
        let mut v_rand = rng.gen_range(0.0..1.0);
        let mut u_rand = rng.gen_range(0.0..1.0);
        if v_rand + u_rand > 1.0 {
            v_rand = 1.0 - v_rand;
            u_rand = 1.0 - u_rand;
        }
        m::add(
            self.origin,
            m::add(m::mul(self.v, v_rand), m::mul(self.u, u_rand)),
        )
    }
}

/// Surface sample returned from surface distributions
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd)]
pub struct SurfSample {
    /// Generated point on the model surface
    pub position: m::Vector,
    /// Triangle the point is contained in
    pub triangle: Triangle,
    // Index of the triangle the point resides in, in the face slice used for initialization
    t_index: usize,
}

//utility functions:

fn vert_ids_to_pos(
    &[i, j, k]: &[usize; 3],
    verts: &[m::Vector],
) -> Result<[m::Vector; 3], MeshRandError> {
    let ind_err = |i| {
        MeshRandError::Initialization(format!(
            "face referenced vert index {} which is out of range (vert.len() = {})",
            i,
            verts.len()
        ))
    };
    Ok([
        *verts.get(i).ok_or_else(|| ind_err(i))?,
        *verts.get(j).ok_or_else(|| ind_err(j))?,
        *verts.get(k).ok_or_else(|| ind_err(k))?,
    ])
}
