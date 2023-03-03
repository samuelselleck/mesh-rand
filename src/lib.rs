//! This crate provides methods of generating random points on the surface of 3d models.
//!
//! ```
//! use mesh_rand::{MeshSurface, SurfSample};
//! use rand::distributions::Distribution;
//!
//! // Verticies for a non-regular tetrahedron:
//! let verticies = [
//!     [0.0, 0.0, 0.0],
//!     [1.0, 0.0, 0.0],
//!     [0.0, 1.0, 0.0],
//!     [0.0, 0.0, 1.0],
//! ];
//! // Faces, oriented to be pointing outwards:
//! let faces = [[1, 0, 2], [2, 0, 3], [0, 1, 3], [1, 2, 3]];
//! let mesh_dist = MeshSurface::new(&verticies, &faces).unwrap();
//! let mut rng = rand::thread_rng();
//! let SurfSample { position, .. } = mesh_dist.sample(&mut rng);
//! println!("generated point on mesh at {position:?}");
//! ```
mod meshsurface;
mod vecmath;
pub use meshsurface::MeshSurface;
pub use meshsurface::SurfSample;
