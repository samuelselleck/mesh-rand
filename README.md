# mesh-rand
Rust library for generating random points on the surface of a mesh

# Example

```rust
use mesh_rand::{MeshSurface, SurfSample};
use rand::distributions::Distribution;
// Verticies and faces for a non-regular tetrahedron:
 let verticies = [
    [0.0, 0.0, 0.0],
    [1.0, 0.0, 0.0],
    [0.0, 1.0, 0.0],
    [0.0, 0.0, 1.0],
];
let faces = [[1, 0, 2], [2, 0, 3], [0, 1, 3], [1, 2, 3]];
let mesh_dist = MeshSurface::new(&verticies, &faces)?;
let mut rng = rand::thread_rng();
let SurfSample { position, .. } = mesh_dist.sample(&mut rng);
println!("generated point on mesh at {position:?}");
```

# Future Additions

* edge/curve distribution
* Poisson distributed surface points
* Randomly sample mesh volume
