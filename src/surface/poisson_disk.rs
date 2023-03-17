use rand_distr::Distribution;
use std::collections::{HashMap, VecDeque};

use super::uniform::UniformSurface;
use super::MeshRandError;
use crate::{vecmath as m, SurfSample};

pub struct PoissonDiskSurface {
    mesh: TriMeshGraph,
    sampler: UniformSurface,
    r: f32,
}

struct TriMeshGraph {
    verticies: Vec<m::Vector>,
    edge_connections: HashMap<[usize; 2], Vec<usize>>,
    faces: Vec<[usize; 3]>,
}

impl TriMeshGraph {
    fn new(r: f32, verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
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

        let mut tri_mesh_graph = Self {
            faces: faces.to_vec(),
            verticies: verts.to_vec(),
            edge_connections,
        };

        tri_mesh_graph.subdivide(r);

        Ok(tri_mesh_graph)
    }

    fn subdivide(&mut self, r: f32) {
        // let show_edge = |&[i, j]: &[usize; 2], verts: &Vec<m::Vector>| {
        //     format!(
        //         "edge ({i}, {j}) - len: {} |",
        //         m::dist_sq(verts[i], verts[j]).sqrt()
        //     )
        // };
        //is using a heap faster?
        let mut too_long_edges: VecDeque<_> = self
            .edge_connections
            .keys()
            .cloned()
            .filter(|&[i, j]| {
                let len_sq = m::dist_sq(self.verticies[i], self.verticies[j]);
                len_sq > r * r
            })
            .collect();
        while let Some(edge) = too_long_edges.pop_front() {
            let mut new_edges = self.divide_edge(edge);
            // println!(
            //     "{:?} returned: {:?}",
            //     show_edge(&edge, &self.verticies),
            //     new_edges
            //         .iter()
            //         .map(|e| show_edge(e, &self.verticies))
            //         .collect::<String>()
            // );
            new_edges.retain(|e| m::dist_sq(self.verticies[e[0]], self.verticies[e[1]]) > r * r);
            too_long_edges.extend(new_edges);
        }
    }

    fn divide_edge(&mut self, mut edge: [usize; 2]) -> Vec<[usize; 2]> {
        //notes:
        //edge 0 vert = touches no new triangles,
        //edge 1 vert = touches new triangles
        let mut new_edges = vec![];

        //verticie mutations
        let new_vert = m::midpoint(self.verticies[edge[0]], self.verticies[edge[1]]);
        let new_v_ind = self.verticies.len();
        self.verticies.push(new_vert);

        //face mutations
        let tris = self
            .edge_connections
            .remove(&edge)
            .expect("edge had no associated tris");

        let mut new_tris = vec![];
        for &tri_ind in &tris {
            //vertex not connected to edge
            let &oposite_v_ind = self.faces[tri_ind]
                .iter()
                .filter(|i| !edge.contains(i))
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
            new_tris.push(new_tri_ind);
            let new_tri = [edge[1], new_v_ind, oposite_v_ind];
            self.faces.push(new_tri);

            //update triangle references
            let mut split_edge = [edge[1], oposite_v_ind];
            split_edge.sort();
            self.edge_connections
                .entry(split_edge)
                .or_default()
                .iter_mut()
                .for_each(|t| {
                    if *t == tri_ind {
                        *t = new_tri_ind;
                    };
                });
            //new edge/triangle references
            let mut n_edge = [new_v_ind, oposite_v_ind];
            n_edge.sort();
            new_edges.push(n_edge);
            self.edge_connections
                .insert(n_edge, vec![tri_ind, new_tri_ind]);
        }

        let mut new_edge = [new_v_ind, edge[1]];
        new_edge.sort();
        self.edge_connections.insert(new_edge, new_tris);
        new_edges.push(new_edge);
        self.edge_connections.remove(&edge);
        edge[1] = new_v_ind;
        edge.sort();
        self.edge_connections.insert(edge, tris);
        new_edges.push(edge);
        new_edges
    }

    fn neighbors(&self, tri_ind: usize) -> impl Iterator<Item = usize> + '_ {
        let [i, j, k] = self.faces[tri_ind];
        let sides = [[i, j], [j, k], [k, i]];
        sides
            .into_iter()
            .flat_map(|mut s| {
                s.sort();
                self.edge_connections[&s].clone()
            })
            .filter(move |&t| t != tri_ind)
    }
}

impl PoissonDiskSurface {
    pub fn new(r: f32, verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        let tri_mesh_graph = TriMeshGraph::new(r, verts, faces)?;
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
        tri_buckets: &Vec<Vec<[f32; 3]>>,
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
