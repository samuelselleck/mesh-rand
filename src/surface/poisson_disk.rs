use rand_distr::Distribution;
use std::collections::HashMap;

use super::uniform::UniformSurface;
use super::MeshRandError;
use crate::{vecmath as m, SurfSample};

pub struct PoissonDiskSurface {
    uniform_sampler: UniformSurface,
    adjacency_list: Vec<Vec<usize>>,
}

impl PoissonDiskSurface {
    pub fn new(verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        let uniform_sampler = UniformSurface::new(verts, faces)?;

        //The code below constructs a list of neighboring triangles.
        //Not hyper optimized or anything, but should for normal models be O(#triangles).

        //OBS there is some weird double indexing for trianges in the below code,
        //since not all triangles in the original model are necesarily kept

        //from two face inds to vec of triangle indexes that contain those inds
        let mut edge_connections = HashMap::<[usize; 2], Vec<usize>>::new();
        for t_ind in 0..uniform_sampler.triangles.len() {
            let [i, j, k] = faces[t_ind];
            let pairs = [[i, j], [j, k], [k, i]];
            for mut pair in pairs {
                pair.sort();
                edge_connections.entry(pair).or_default().push(i);
            }
        }

        //each triangle index should contain a list of all neighboring triangles
        let mut adjacency_list = vec![Vec::new(); uniform_sampler.triangles.len()];

        for adj in edge_connections.values() {
            for &v in adj {
                for &u in adj {
                    if v != u {
                        adjacency_list[v].push(u);
                    }
                }
            }
        }
        Ok(Self {
            uniform_sampler,
            adjacency_list,
        })
    }

    pub fn sample_naive<R>(&self, r: f32, retries: u32, max: u32, rng: &mut R) -> Vec<m::Vector>
    where
        R: rand::Rng + ?Sized,
    {
        let tris = self.uniform_sampler.triangles.len();
        let mut tri_buckets = vec![Vec::new(); tris];

        let mut count = 0;
        let mut failures = 0;
        while retries > failures && count < max {
            let SurfSample {
                position, t_index, ..
            } = self.uniform_sampler.sample(rng);
            let r_closest = self.find_closest(r, position, t_index, &tri_buckets);
            if r_closest > r {
                tri_buckets[t_index].push(position);
                count += 1;
            } else {
                failures += 1;
            }
        }

        tri_buckets.concat()
    }

    fn find_closest(
        &self,
        r: f32,
        position: [f32; 3],
        t_index: usize,
        tri_buckets: &Vec<Vec<[f32; 3]>>,
    ) -> f32 {
        let mut search_r = r * r * 1.1;
        let mut searching = vec![t_index];
        let mut visited = vec![];
        while let Some(tri_ind) = searching.pop() {
            let tri = self.uniform_sampler.triangles[tri_ind];
            let intersects = tri.intersects_sphere(position, search_r);
            if intersects {
                for &next_ind in &self.adjacency_list[tri_ind] {
                    if !visited.contains(&next_ind) {
                        visited.push(next_ind);
                        searching.push(next_ind);
                    }
                }
                let samples_in_tri = &tri_buckets[tri_ind];
                let closest = samples_in_tri
                    .iter()
                    .map(|&p| m::dist_sq(p, position))
                    .reduce(f32::min)
                    .unwrap_or(f32::MAX);
                search_r = search_r.min(closest);
            }
        }
        search_r.sqrt()
    }
}
