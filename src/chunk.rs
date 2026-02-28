use core::f32;

use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use noise::{NoiseFn, Perlin};
use rand::random;

pub fn load_chunk(size: u64, x: u64, z: u64, seed: u32) -> Mesh {
    let x_offset = x * (size - 1); // Prevents seams
    let z_offset = z * (size - 1);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();

    let perlin = Perlin::new(seed);

    for i in 0..size {
        for j in 0..size {
            let world_i = x_offset + i; // Offset for perlin noise, absolute coordinates
            let world_j = z_offset + j;

            // Slope terrain using absolute coordinates
            let mut y = perlin.get([world_i as f64 * 1.1, world_j as f64 * 1.1]) * 0.5;
            y += perlin.get([world_i as f64 * 0.01, world_j as f64 * 0.01]) * 10.;
            y += perlin.get([world_i as f64 * 0.0002, world_j as f64 * 0.0002]) * 100.;
            y += perlin.get([world_i as f64 * 0.000003, world_j as f64 * 0.000003]) * 10000.;

            positions.push([i as f32, y as f32, j as f32]); // local coordinates
            uvs.push([i as f32 / size as f32, j as f32 / size as f32]);
            colors.push([0.1, 0.4 * random::<f32>(), 0.1, 1.]); // Push semi-random green color to
                                                                // each face
        }
    }

    for i in 0..size - 1 {
        for j in 0..size - 1 {
            let top_left = i * size + j;
            let top_right = top_left + 1;
            let bottom_left = top_left + size;
            let bottom_right = bottom_left + 1;

            indices.push(top_left as u32);
            indices.push(top_right as u32);
            indices.push(bottom_left as u32);

            indices.push(top_right as u32);
            indices.push(bottom_right as u32);
            indices.push(bottom_left as u32);
        }
    }

    let normals = vec![[0.0, 1.0, 0.0]; (size * size) as usize];
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U32(indices));

    return mesh;
}
