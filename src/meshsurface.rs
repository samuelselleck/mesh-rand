use crate::vecmath as m;
use rand_distr::weighted_alias::WeightedAliasIndex;
use rand_distr::Distribution;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MError {
    #[error("failed to initialize: {0}")]
    Initialization(String),
}

#[derive(Debug)]
struct Triangle {
    origin: m::Vector,
    normal: m::Vector,
    u: m::Vector,
    v: m::Vector,
}

/// A distribution for sampling uniformly distributed points on the surface of a 3d model
///
/// Uniformly samples the surface of a model by first randomly picking a triangle with probability
/// proportional to its area, and then uniformly samples a point within that triangle.
///
/// # Example
///
/// ```
/// use mesh_rand::MeshSurface;
/// use rand::distributions::Distribution;
///
/// let verticies = [
///     [1.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0],
///     [0.0, 0.0, 1.0],
///     [1.0, 2.0, 0.0],
/// ];
/// let faces = [[0, 1, 2], [0, 1, 3]];
/// let mesh_dist = MeshSurface::new(&verticies, &faces).unwrap();
/// let mut rng = rand::thread_rng();
/// let sample = mesh_dist.sample(&mut rng);
/// println!(
///     "generated point on mesh at {:?} located on face with index {:?} with normal {:?}",
///     sample.position, sample.face_index, sample.normal
/// );
/// ```
///

#[derive(Debug)]
pub struct MeshSurface {
    triangles: Vec<Triangle>,
    triangle_dist: WeightedAliasIndex<f32>,
}

impl MeshSurface {
    /// Initializes a new mesh surface distribution given verticies and faces (triangles)
    ///
    /// # Result
    /// Returns an error if:
    /// * An index defining a face is out of range of the verticies collection
    /// * The area of one of the triangles provided is very close to 0 (`f32::is_normal(area) == false`)
    /// * The collection of faces is embty
    pub fn new(verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MError> {
        let mut triangles = Vec::with_capacity(faces.len());
        let mut triangle_areas = Vec::with_capacity(faces.len());
        let ind_err = |f_id, i| {
            MError::Initialization(format!(
                "face at index {} referenced vert index {} which is out of range (vert.len() = {})",
                f_id,
                i,
                verts.len()
            ))
        };
        for (f_id, &[i1, i2, i3]) in faces.iter().enumerate() {
            let (p1, p2, p3) = (
                *verts.get(i1).ok_or_else(|| ind_err(f_id, i1))?,
                *verts.get(i2).ok_or_else(|| ind_err(f_id, i2))?,
                *verts.get(i3).ok_or_else(|| ind_err(f_id, i3))?,
            );
            let origin = p1;
            let u = m::diff(p2, p1);
            let v = m::diff(p3, p1);
            let normal_dir = m::cross(u, v);
            let len = m::len(normal_dir);
            let area = len / 2.0;
            if !f32::is_normal(area) {
                return Err(MError::Initialization(format!(
                    "area of face at index {} too close to 0 (f32::is_normal(area) == false)",
                    f_id
                )));
            }
            let normal = m::div(normal_dir, len);
            triangle_areas.push(area);
            triangles.push(Triangle {
                origin,
                u,
                v,
                normal,
            })
        }
        let triangle_dist = WeightedAliasIndex::new(triangle_areas).map_err(|_| {
            //cases of trangle area being close to 0 handled above, must be empty
            MError::Initialization("faces array is embty".into())
        })?;
        Ok(MeshSurface {
            triangles,
            triangle_dist,
        })
    }
}

/// Surface sample returned from surface distributions
pub struct SurfSample {
    /// Generated point on the model surface
    pub position: m::Vector,
    /// Normalized normal vector of the triangle the point resides in.
    ///
    /// OBS: The normal will be pointing out of the positively oriented side of the triangle. As
    /// an example, the triangle
    /// defined by the verticies `[a, b, c]` where `a = [0.0, 0.0, 0.0]`, `b = [1.0, 0.0, 0.0]` and
    /// `c = [0.0, 1.0, 0.0]` has the normal [0.0, 0.0, 1.0]. While a triangle defined by
    /// `[b, a, c]` has the normal [0.0, 0.0, -1.0].
    pub normal: m::Vector,
    /// Index of the triangle the point resides in, in the face slice used for initialization
    pub face_index: usize,
}

impl Distribution<SurfSample> for MeshSurface {
    /// Samples the model surface uniformly, returning an instance of the [SurfSample] struct
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> SurfSample {
        let t_ind = self.triangle_dist.sample(rng);
        let Triangle {
            origin,
            u,
            v,
            normal,
        } = self.triangles[t_ind];
        let mut v_rand = rng.gen_range(0.0..=1.0);
        let mut u_rand = rng.gen_range(0.0..=1.0);
        if v_rand + u_rand > 1.0 {
            v_rand = 1.0 - v_rand;
            u_rand = 1.0 - u_rand;
        }
        let point = m::add(origin, m::add(m::mul(v, v_rand), m::mul(u, u_rand)));
        SurfSample {
            position: point,
            face_index: t_ind,
            normal,
        }
    }
}
