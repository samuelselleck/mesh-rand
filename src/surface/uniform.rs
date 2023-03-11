use crate::vecmath as m;
use rand_distr::weighted_alias::WeightedAliasIndex;
use rand_distr::Distribution;

use super::{vert_ids_to_pos, MeshRandError, SurfSample, Triangle};

/// A distribution for sampling points uniformly on the surface of a 3d model
///
/// Samples the surface of a model by first randomly picking a triangle with probability
/// proportional to its area, and then uniformly samples a point within that triangle.
///
/// # Example
///
/// ```
/// use mesh_rand::UniformSurface;
/// use rand::distributions::Distribution;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let verticies = [
///     [1.0, 0.0, 0.0],
///     [0.0, 1.0, 0.0],
///     [0.0, 0.0, 1.0],
///     [1.0, 2.0, 0.0],
/// ];
/// let faces = [[0, 1, 2], [0, 1, 3]];
/// let mesh_dist = UniformSurface::new(&verticies, &faces)?;
/// let mut rng = rand::thread_rng();
/// let sample = mesh_dist.sample(&mut rng);
/// println!(
///     "generated point on mesh at {:?} located on face with index {:?} with normal {:?}",
///     sample.position, sample.face_index, sample.normal
/// );
/// # Ok(())
/// # }
/// ```
///
#[derive(Debug, Clone)]
pub struct UniformSurface {
    pub(crate) triangles: Vec<Triangle>,
    pub(crate) triangle_dist: WeightedAliasIndex<f32>,
}

impl UniformSurface {
    /// Initializes a new mesh surface distribution given verticies and faces (triangles)
    ///
    /// # Result
    /// Returns an error if:
    /// * An index defining a face is out of range of the verticies collection
    // * The area of one of the triangles provided is very close to 0 (`f32::is_normal(area) == false`)
    /// * The collection of faces is empty
    pub fn new(verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        let mut triangles = Vec::with_capacity(faces.len());
        let mut triangle_areas = Vec::with_capacity(faces.len());

        for face in faces {
            let [p1, p2, p3] = vert_ids_to_pos(face, verts)?;
            let Ok(triangle) = Triangle::from_points(p1, p2, p3) else {
                continue;
            };
            triangle_areas.push(triangle.area);
            triangles.push(triangle);
        }

        let triangle_dist = WeightedAliasIndex::new(triangle_areas).map_err(|_| {
            //cases of trangle area being close to 0 handled above, must be empty
            MeshRandError::Initialization("faces array is embty".into())
        })?;

        Ok(UniformSurface {
            triangles,
            triangle_dist,
        })
    }
}

impl Distribution<SurfSample> for UniformSurface {
    /// Samples the model surface uniformly, returning an instance of the [SurfSample] struct
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> SurfSample {
        let t_ind = self.triangle_dist.sample(rng);
        let triangle = self.triangles[t_ind];
        let point = triangle.sample(rng);
        SurfSample {
            position: point,
            t_index: t_ind,
            triangle,
        }
    }
}
