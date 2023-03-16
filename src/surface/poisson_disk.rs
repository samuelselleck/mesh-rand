use rand_distr::Distribution;
use std::collections::HashMap;

use super::uniform::UniformSurface;
use super::MeshRandError;
use crate::{vecmath as m, SurfSample};

pub struct PoissonDiskSurface {
    adjacency_list: Vec<Vec<usize>>,
    verticies: Vec<m::Vector>,
    edge_connections: HashMap<[usize; 2], Vec<usize>>,
    faces: Vec<[usize; 3]>,
}

impl PoissonDiskSurface {
    pub fn new(r: f32, verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        let tri_count = faces.len();
        //edge vert indicies to triangle indicies;
        let mut edge_connections = HashMap::<[usize; 2], Vec<usize>>::new();
        for t_ind in 0..tri_count {
            let [i, j, k] = faces[t_ind];
            let pairs = [[i, j], [j, k], [k, i]];
            for mut pair in pairs {
                pair.sort();
                edge_connections.entry(pair).or_default().push(t_ind);
            }
        }

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
        let poisson_disk_surf = Self {
            faces: faces.to_vec(),
            verticies: verts.to_vec(),
            edge_connections,
            adjacency_list,
        };

        poisson_disk_surf.subdivide();

        Ok(poisson_disk_surf)
    }

    fn subdivide(&mut self) {}

    fn divide_edge(&mut self, mut edge: [usize; 2]) {
        //notes:
        //edge 0 vert = touches no new triangles,
        //edge 1 vert = touches new triangles

        //verticie mutations
        let new_vert = m::midpoint(self.verticies[edge[0]], self.verticies[edge[1]]);
        let new_v_ind = self.verticies.len();
        self.verticies.push(new_vert);

        //face mutations
        let tris = self
            .edge_connections
            .get(&edge)
            .expect("edge had no associated tris");

        for &tri_ind in tris {
            //vertex not connected to edge
            let (oposite_t_ind, &oposite_v_ind) = self.faces[tri_ind]
                .iter()
                .enumerate()
                .filter(|(ind, i)| !edge.contains(i))
                .next()
                .expect("triangle contained duplicate verts");
            //replace edge[1] with new_v_ind
            self.faces[tri_ind].iter_mut().for_each(|i| {
                if *i == edge[1] {
                    *i = new_v_ind
                }
            });
            //new triangle
            let new_tri_ind = self.faces.len();
            let new_tri = [edge[1], new_v_ind, oposite_v_ind];
            self.faces.push(new_tri);
            //update all triangle references
            let mut split_edge = [edge[1], oposite_v_ind];
            split_edge.sort();
            let mut to_rem = vec![];
            self.edge_connections[&split_edge].iter_mut().for_each(|t| {
                self.adjacency_list[*t].iter_mut().for_each(|o| {
                    if *o == tri_ind {
                        to_rem.push(t);
                        *o = new_tri_ind;
                    }
                });
                if *t == tri_ind {
                    *t = new_tri_ind;
                };
            });
            //new edge/triangle references
            self.adjacency_list[tri_ind].retain(|t| to_rem.contains(t));
            //

            //update all edge references
        }
        //things to do at end
        self.edge_heap.push(new_edge);
        self.edge_heap.push(edge); //modified edge
    }

    pub fn sample_naive<R>(&self, r: f32, retries: u32, max: u32, rng: &mut R) -> Vec<m::Vector>
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
            let exists_closer = self.exists_point_within_sphere(r, position, t_root, &tri_buckets);
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
        tri_buckets: &Vec<Vec<[f32; 3]>>,
    ) -> bool {
        let mut searching = vec![t_index];
        let mut visited = vec![t_index];
        while let Some(tri_ind) = searching.pop() {
            let tri = self.uniform_sampler.triangles[tri_ind];
            let intersects = tri.intersects_sphere(position, r);
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
                    .any(|&p| m::dist_sq(p, position) < r * r);
                if exists_closer {
                    return true;
                }
            }
        }
        false
    }
}
