mod vecmath;
use anyhow::Result;
use rand::distributions::weighted::WeightedIndex;
use rand::distributions::Distribution;
use vecmath as m;

struct Triangle {
    origin: m::Vector,
    normal: m::Vector,
    u: m::Vector,
    v: m::Vector,
}

pub struct MeshRand {
    triangles: Vec<Triangle>,
    triangle_dist: WeightedIndex<f32>,
}

impl MeshRand {
    pub fn new(verts: Vec<m::Vector>, faces: &[[usize; 3]]) -> Result<Self> {
        let mut triangles = Vec::with_capacity(faces.len());
        let mut triangle_areas = Vec::with_capacity(faces.len());
        for &[i1, i2, i3] in faces {
            let (p1, p2, p3) = (verts[i1], verts[i2], verts[i3]);
            let origin = p1;
            let u = m::diff(p2, p1);
            let v = m::diff(p3, p1);
            let normal_dir = m::cross(u, v);
            let len = m::len(normal_dir);
            let area = len / 2.0;
            let normal = m::div(normal_dir, len);
            triangle_areas.push(area);
            triangles.push(Triangle {
                origin,
                u,
                v,
                normal,
            })
        }
        let triangle_dist = WeightedIndex::new(triangle_areas)?;
        Ok(MeshRand {
            triangles,
            triangle_dist,
        })
    }
}

impl Distribution<(m::Vector, usize, m::Vector)> for MeshRand {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> (m::Vector, usize, m::Vector) {
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
        (point, t_ind, normal)
    }
}

impl Distribution<m::Vector> for MeshRand {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> m::Vector {
        Distribution::<(m::Vector, usize, m::Vector)>::sample(&self, rng).0
    }
}
