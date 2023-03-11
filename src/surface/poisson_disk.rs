use rand_distr::Distribution;
use std::collections::HashMap;

use super::uniform::UniformSurface;
use super::MeshRandError;
use crate::{vecmath as m, SurfSample};

pub struct PoissonDiskSurface {
    uniform_sampler: UniformSurface,
    adjacency_list: Vec<Vec<usize>>,
    r: f32,
}

impl PoissonDiskSurface {
    pub fn new(r: f32, verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        let (verts, faces) = Self::split_triangles(verts, faces, r);
        let uniform_sampler = UniformSurface::new(&verts, &faces)?;
        println!("#faces: {}, #verts: {}", faces.len(), verts.len());

        let tri_count = uniform_sampler.triangles.len();
        //The code below constructs a list of neighboring triangles.
        //Not hyper optimized or anything, but should for normal models be O(#triangles).

        //OBS there is some weird double indexing for trianges in the below code,
        //since not all triangles in the original model are necesarily kept

        //from two face inds to vec of triangle indexes that contain those inds
        let mut edge_connections = HashMap::<[usize; 2], Vec<usize>>::new();
        for t_ind in 0..tri_count {
            let [i, j, k] = faces[t_ind];
            let pairs = [[i, j], [j, k], [k, i]];
            for mut pair in pairs {
                pair.sort();
                edge_connections.entry(pair).or_default().push(t_ind);
            }
        }
        println!("edge connections: {:?}", edge_connections);

        //each triangle index should contain a list of all neighboring triangles
        let mut adjacency_list = vec![Vec::new(); tri_count];

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
            r,
        })
    }

    fn split_triangles(
        verts: &[m::Vector],
        faces: &[[usize; 3]],
        max_side_len: f32,
    ) -> (Vec<m::Vector>, Vec<[usize; 3]>) {
        let mut verts = verts.to_vec();
        let mut faces = faces.to_vec();
        let mut i = 0;
        while i < faces.len() {
            if !Self::split_triangle(i, &mut faces, &mut verts, max_side_len) {
                i += 1;
            }
        }
        (verts, faces)
    }

    fn split_triangle(
        ind: usize,
        faces: &mut Vec<[usize; 3]>,
        verts: &mut Vec<m::Vector>,
        max: f32,
    ) -> bool {
        let [i, j, k] = faces[ind];
        let sides = [[j, k], [i, k], [i, j]];
        let (s_ind, max_side_sq) = sides
            .iter()
            .enumerate()
            .map(|(c, &[i, j])| {
                let side_len_sq = m::dist_sq(verts[i], verts[j]);
                (c, side_len_sq)
            })
            .max_by(|(_, len1), (_, len2)| len1.total_cmp(len2))
            .unwrap();
        if max_side_sq <= max * max {
            return false;
        }
        let [l, r] = sides[s_ind];
        let o = faces[ind][s_ind];
        let n = verts.len();
        let new_v = m::midpoint(verts[l], verts[r]);
        verts.push(new_v);
        let l_tri = [l, o, n];
        let r_tri = [n, o, r];
        faces.swap_remove(ind);
        faces.push(l_tri);
        faces.push(r_tri);
        return true;
    }

    pub fn sample_naive<R>(&self, retries: u32, max: u32, rng: &mut R) -> Vec<m::Vector>
    where
        R: rand::Rng + ?Sized,
    {
        let tri_count = self.uniform_sampler.triangles.len();
        let mut tri_buckets = vec![Vec::new(); tri_count];

        let mut count = 0;
        let mut failures = 0;
        while retries > failures && count < max {
            let SurfSample {
                position,
                t_index: t_root,
                ..
            } = self.uniform_sampler.sample(rng);
            let exists_closer = self.exists_point_within_sphere(position, t_root, &tri_buckets);
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
        position: [f32; 3],
        t_index: usize,
        tri_buckets: &Vec<Vec<[f32; 3]>>,
    ) -> bool {
        let mut searching = vec![t_index];
        let mut visited = vec![t_index];
        while let Some(tri_ind) = searching.pop() {
            let tri = self.uniform_sampler.triangles[tri_ind];
            let intersects = tri.intersects_sphere(position, self.r);
            if intersects {
                for &next_ind in &self.adjacency_list[tri_ind] {
                    if !visited.contains(&next_ind) {
                        visited.push(next_ind);
                        searching.push(next_ind);
                    }
                }
                let samples_in_tri = &tri_buckets[tri_ind];
                let exists_closer = samples_in_tri
                    .iter()
                    .any(|&p| m::dist_sq(p, position) < self.r * self.r);
                if exists_closer {
                    return true;
                }
            }
        }
        false
    }
}
