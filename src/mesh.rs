use crate::{errors::MeshRandError, vecmath as m};
use std::collections::{HashMap, VecDeque};

pub(crate) struct SpaceQueryMesh {
    pub verticies: Vec<m::Vector>,
    edge_connections: HashMap<[usize; 2], Vec<usize>>,
    pub faces: Vec<[usize; 3]>,
}

impl SpaceQueryMesh {
    pub fn new(r: f32, verts: &[m::Vector], faces: &[[usize; 3]]) -> Result<Self, MeshRandError> {
        //edge vert indicies to triangle indicies;
        let mut edge_connections = HashMap::<[usize; 2], Vec<usize>>::new();
        for (t_ind, &[i, j, k]) in faces.iter().enumerate() {
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
                .find(|i| !edge.contains(i))
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

    pub fn neighbors(&self, tri_ind: usize) -> impl Iterator<Item = usize> + '_ {
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
