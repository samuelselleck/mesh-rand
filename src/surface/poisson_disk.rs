use rand_distr::Distribution;

use super::uniform::UniformSurface;
use crate::errors::MeshRandError;
use crate::mesh::SpaceQueryMesh;
use crate::{vecmath as m, SurfSample};

pub struct PoissonDiskSurface {
    mesh: SpaceQueryMesh,
    sampler: UniformSurface,
    r: f32,
}

impl PoissonDiskSurface {
    pub fn new(r: f32, verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        let tri_mesh_graph = SpaceQueryMesh::new(r, verts, faces)?;
        println!("trimesh constructed");
        let sampler = UniformSurface::new(&tri_mesh_graph.verticies, &tri_mesh_graph.faces)?;
        Ok(Self {
            mesh: tri_mesh_graph,
            sampler,
            r,
        })
    }

    pub fn sample_naive<R>(&self, retries: u32, max: u32, rng: &mut R) -> Vec<m::Vector>
    where
        R: rand::Rng + ?Sized,
    {
        let tri_count = self.sampler.triangles.len();
        let mut tri_buckets = vec![Vec::new(); tri_count];

        let mut count = 0;
        let mut failures = 0;
        while retries > failures && count < max {
            let SurfSample {
                position,
                t_index: t_root,
                ..
            } = self.sampler.sample(rng);
            let exists_closer =
                self.exists_point_within_sphere(self.r, position, t_root, &tri_buckets);
            if !exists_closer {
                tri_buckets[t_root].push(position);
                count += 1;
            } else {
                failures += 1;
            }
        }

        tri_buckets.concat()
    }

    fn exists_point_within_sphere(
        &self,
        r: f32,
        position: [f32; 3],
        t_index: usize,
        tri_buckets: &[Vec<[f32; 3]>],
    ) -> bool {
        let mut searching = vec![t_index];
        let mut visited = vec![t_index];
        while let Some(tri_ind) = searching.pop() {
            let tri = self.sampler.triangles[tri_ind];
            let intersects = tri.intersects_sphere(position, r);
            if intersects {
                for next_ind in self.mesh.neighbors(tri_ind) {
                    if !visited.contains(&next_ind) {
                        visited.push(next_ind);
                        searching.push(next_ind);
                    }
                }
                let samples_in_tri = &tri_buckets[tri_ind];
                let exists_closer = samples_in_tri
                    .iter()
                    .any(|&p| m::dist_sq(p, position) < r * r);
                if exists_closer {
                    return true;
                }
            }
        }
        false
    }
}
