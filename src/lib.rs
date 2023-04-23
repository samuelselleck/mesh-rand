//! This crate provides methods of generating random points on the surface of 3d models.
//!
//! ```
//! use mesh_rand::{UniformSurface, SurfSample};
//! use rand::distributions::Distribution;
//!
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // Verticies for a non-regular tetrahedron:
//! let verticies = [
//!     [0.0, 0.0, 0.0],
//!     [1.0, 0.0, 0.0],
//!     [0.0, 1.0, 0.0],
//!     [0.0, 0.0, 1.0],
//! ];
//! // Faces, oriented to be pointing outwards:
//! let faces = [[1, 0, 2], [2, 0, 3], [0, 1, 3], [1, 2, 3]];
//! let mesh_dist = UniformSurface::new(&verticies, &faces)?;
//! let mut rng = rand::thread_rng();
//! let SurfSample { position, .. } = mesh_dist.sample(&mut rng);
//! println!("generated point on mesh at {position:?}");
//! # Ok(())
//! # }
//! ```
mod errors;
mod mesh;
mod surface;
mod vecmath;
pub use surface::poisson_disk::PoissonDiskSurface;
pub use surface::uniform::UniformSurface;
pub use surface::SurfSample;
