use core::f32;

use crate::GRASS_TILE_WORLD_SIZE;
use bevy::{
    asset::RenderAssetUsages,
    render::mesh::{Indices, Mesh, PrimitiveTopology},
};
use noise::{NoiseFn, Perlin};

use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use rand::random;

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct TerrainParams {
    #[inspector(min = 500.0, max = 5000.0, display = NumberDisplay::Slider, speed = 50.)]
    pub continentality: f64,
    pub seed: u32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider, speed = 0.01)]
    pub edge0: f64,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider, speed = 0.01)]
    pub edge1: f64,
    pub plains: Biome,
    pub mountains: Biome,
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Biome {
    #[inspector(min = 1, max = 12, display = NumberDisplay::Slider)]
    pub octaves: u32,
    #[inspector(min = 1., max = 100., display = NumberDisplay::Slider)]
    pub amplitude: f32,
    #[inspector(min = 0., max = 1., display = NumberDisplay::Slider, speed = 0.01)]
    pub gain: f64,
    #[inspector(min = 1.5, max = 3.0, display = NumberDisplay::Slider, speed = 0.01)]
    pub lacunarity: f64,
    #[inspector(min = 0., max = 0.02, display = NumberDisplay::Slider, speed = 0.001)]
    pub frequency: f64,
    pub noise_shape: NoiseShape,
}
#[derive(Clone, Copy, Reflect)]
pub enum NoiseShape {
    Standard,
    Ridged,
}

#[derive(Component)]
pub struct Chunk {
    pub x: u64,
    pub z: u64,
}

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            continentality: 1000.,
            plains: Biome {
                octaves: 6,
                gain: 0.45,
                frequency: 0.001,
                lacunarity: 1.8,
                amplitude: 24.,
                noise_shape: NoiseShape::Standard,
            },
            mountains: Biome {
                octaves: 8,
                gain: 0.5,
                frequency: 0.0023,
                lacunarity: 2.0,
                amplitude: 60.,
                noise_shape: NoiseShape::Ridged,
            },
            edge0: 0.80,
            edge1: 0.90,
            seed: random(),
        }
    }
}

pub fn load_chunk(size: u64, x: u64, z: u64, params: &TerrainParams) -> Mesh {
    let x_offset = x * (size - 1); // Prevents seams
    let z_offset = z * (size - 1);

    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();

    let perlin = Perlin::new(params.seed);

    for i in 0..size {
        for j in 0..size {
            let world_i = x_offset + i; // Offset for perlin noise, absolute coordinates
            let world_j = z_offset + j;

            let neighbor_north = height_at(&perlin, params, world_i as f64, world_j as f64 - 1.);
            let neighbor_south = height_at(&perlin, params, world_i as f64, world_j as f64 + 1.);
            let neighbor_east = height_at(&perlin, params, world_i as f64 + 1., world_j as f64);
            let neighbor_west = height_at(&perlin, params, world_i as f64 - 1., world_j as f64);
            let normal_vec = Vec3::new(
                (neighbor_west - neighbor_east) as f32,
                2.0,
                (neighbor_north - neighbor_south) as f32,
            )
            .normalize();
            normals.push(normal_vec.into());

            positions.push([
                i as f32,
                height_at(&perlin, params, world_i as f64, world_j as f64) as f32,
                j as f32,
            ]); // local coordinates

            uvs.push([
                world_i as f32 / GRASS_TILE_WORLD_SIZE,
                world_j as f32 / GRASS_TILE_WORLD_SIZE,
            ]);

            let patch = perlin.get([world_i as f64 * 0.02, world_j as f64 * 0.02]) as f32;
            let shade = 0.88 + 0.12 * (patch * 0.5 + 0.5);
            colors.push([shade, shade, shade, 1.]);
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

pub fn height_at(perlin: &Perlin, params: &TerrainParams, x: f64, z: f64) -> f32 {
    let mask: f64 =
        1. - ((perlin.get([x / params.continentality, z / params.continentality]) / 2.) + 0.5);
    let t: f64 = ((mask - params.edge0) / (params.edge1 - params.edge0)).clamp(0., 1.);

    let plains = fbm(perlin, &params.plains, x, z);
    let mountains = fbm(perlin, &params.mountains, x, z);
    let y: f64 = plains + (t * t * (3. - 2. * t)) * (mountains - plains);

    y as f32
}

fn fbm(perlin: &Perlin, biome: &Biome, x: f64, z: f64) -> f64 {
    let mut y: f64 = 0.0;
    let mut frequency = biome.frequency;
    let mut amplitude: f64 = biome.amplitude as f64;
    let mut weight: f64 = 1.0;

    for i in 0..biome.octaves {
        let o: f64 = i as f64 * 503.2099;
        let perlin_x = x * frequency + o;
        let perlin_z = z * frequency - o;
        let perlin_result = perlin.get([perlin_x, perlin_z]);
        y += (1. - (perlin_result).abs()).powi(2) * weight * amplitude;
        weight *= y / amplitude;
        weight = weight.clamp(0., 1.);
        amplitude *= biome.gain;
        frequency *= biome.lacunarity;
    }
    y
}
