use crate::{
    smoothstep, CHUNK_SIZE, GRASS_TILE_WORLD_SIZE, LOAD_RADIUS, LOD_DETAIL, MAX_CHUNKS, SPACING,
};
use bevy::{
    asset::RenderAssetUsages,
    mesh::{Indices, Mesh, PrimitiveTopology},
    platform::collections::HashMap,
    tasks::{futures::check_ready, AsyncComputeTaskPool, Task},
};
use core::f32;
use noise::{NoiseFn, Perlin};

use bevy::prelude::*;
use bevy_inspector_egui::inspector_options::std_options::NumberDisplay;
use bevy_inspector_egui::prelude::*;
use rand::random;

#[derive(Resource, Reflect, InspectorOptions, Clone, Copy)]
#[reflect(Resource, InspectorOptions)]
pub struct TerrainParams {
    #[inspector(min = 500.0, max = 5000.0, display = NumberDisplay::Slider, speed = 50_f32)]
    pub continentality: f64,
    pub seed: u32,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider, speed = 0.01_f32)]
    pub edge0: f64,
    #[inspector(min = 0.0, max = 1.0, display = NumberDisplay::Slider, speed = 0.01_f32)]
    pub edge1: f64,
    pub crease: f64,
    pub weight_gain: f64,
    pub plains: Biome,
    pub mountains: Biome,
}

#[derive(Reflect, InspectorOptions, Clone, Copy)]
#[reflect(InspectorOptions)]
pub struct Biome {
    #[inspector(min = 1, max = 12, display = NumberDisplay::Slider)]
    pub octaves: u32,
    #[inspector(min = 1., max = 300., display = NumberDisplay::Slider)]
    pub amplitude: f32,
    #[inspector(min = 0., max = 1., display = NumberDisplay::Slider, speed = 0.01_f32)]
    pub gain: f64,
    #[inspector(min = 1.5, max = 3.0, display = NumberDisplay::Slider, speed = 0.01_f32)]
    pub lacunarity: f64,
    #[inspector(min = 0., max = 0.02, display = NumberDisplay::Slider, speed = 0.001_f32)]
    pub frequency: f64,
    pub noise_shape: NoiseShape,
}
#[derive(Clone, Copy, Reflect, PartialEq)]
pub enum NoiseShape {
    Standard,
    Ridged,
}

#[derive(Component)]
pub struct Chunk {
    pub x: i64,
    pub z: i64,
}

#[derive(Resource, Clone)]
pub struct GrassMaterial(pub Handle<StandardMaterial>);

#[derive(Resource, Default)]
pub struct ChunkMap(HashMap<IVec2, Entity>);

#[derive(Component)]
pub struct ChunkTask(pub Task<(Mesh, Vec<f32>)>);

#[derive(Component)]
pub struct ChunkHeights(pub Vec<f32>);

#[derive(Component)]
pub struct ChunkLod {
    pub level: u32,
}
#[derive(Component)]
pub struct LodTask(pub Task<Mesh>);

impl Default for TerrainParams {
    fn default() -> Self {
        Self {
            continentality: 2500.,
            plains: Biome {
                octaves: 6,
                gain: 0.42,
                frequency: 0.0012,
                lacunarity: 2.0,
                amplitude: 18.,
                noise_shape: NoiseShape::Standard,
            },
            mountains: Biome {
                octaves: 8,
                gain: 0.42,
                frequency: 0.0008,
                lacunarity: 2.1,
                amplitude: 160.,
                noise_shape: NoiseShape::Ridged,
            },
            edge0: 0.52,
            edge1: 0.78,
            crease: 0.0001,
            weight_gain: 2.0,
            seed: random(),
        }
    }
}

pub fn load_chunk(
    size: i64,
    x: i64,
    z: i64,
    params: &TerrainParams,
    spacing: f32,
) -> (Mesh, Vec<f32>) {
    let perlin = Perlin::new(params.seed);
    let heights = sample_heights(size, x * (size - 1), z * (size - 1), params, &perlin);
    let mut dist: usize = ((x * x + z * z) as f32).powf(0.5) as usize;
    dist /= LOD_DETAIL as usize;
    dist = (dist as f32).ln().max(0.0) as usize;
    let step = 1usize << dist.min(CHUNK_SIZE as usize - 1);
    let mesh = build_mesh(
        size,
        x * (size - 1),
        z * (size - 1),
        spacing,
        step,
        &heights,
        &perlin,
    );
    (mesh, heights)
}

pub fn sample_heights(
    size: i64,
    x: i64,
    z: i64,
    params: &TerrainParams,
    perlin: &Perlin,
) -> Vec<f32> {
    let w = size as usize + 2;
    let mut heights = vec![0.0_f32; w * w];
    for i in 0..w {
        for j in 0..w {
            heights[i * w + j] =
                height_at(perlin, params, x as f64 + i as f64, z as f64 + j as f64);
        }
    }
    return heights;
}

pub fn build_mesh(
    size: i64,
    x: i64,
    z: i64,
    spacing: f32,
    step: usize,
    heights: &Vec<f32>,
    perlin: &Perlin,
) -> Mesh {
    let mut positions: Vec<[f32; 3]> = Vec::new();
    let mut uvs: Vec<[f32; 2]> = Vec::new();
    let mut indices: Vec<u16> = Vec::new();
    let mut colors: Vec<[f32; 4]> = Vec::new();
    let mut normals: Vec<[f32; 3]> = Vec::new();
    let h = |i: usize, j: usize| -> f32 { heights[i * (size as usize + 2) + j] };
    for i in (0..size).step_by(step) {
        for j in (0..size).step_by(step) {
            let world_i = x + i; // Offset for perlin noise, absolute coordinates
            let world_j = z + j;

            let gi = i as usize + 1;
            let gj = j as usize + 1;

            let neighbor_north = h(gi, gj - 1);
            let neighbor_south = h(gi, gj + 1);
            let neighbor_east = h(gi + 1, gj);
            let neighbor_west = h(gi - 1, gj);
            let normal_vec = Vec3::new(
                (neighbor_west - neighbor_east) as f32,
                2.0 * spacing,
                (neighbor_north - neighbor_south) as f32,
            )
            .normalize();
            normals.push(normal_vec.into());

            positions.push([i as f32, h(gi, gj), j as f32]); // local coordinates

            uvs.push([
                world_i as f32 / GRASS_TILE_WORLD_SIZE,
                world_j as f32 / GRASS_TILE_WORLD_SIZE,
            ]);

            let patch =
                perlin.get([world_i as f64 * 0.011 + 97.3, world_j as f64 * 0.011 + 30.9]) as f32;
            let shade = 0.88 + 0.12 * (patch * 0.5 + 0.5);
            let dry = Vec3::new(1.0, 0.85, 0.6);
            let lush = Vec3::new(0.8, 1.0, 0.8);
            let t: f32 = perlin.get([
                world_i as f64 * 0.0017 + 596.2,
                world_j as f64 * 0.0017 + 13.9,
            ]) as f32;
            let tint = dry.lerp(lush, t) * shade as f32;
            colors.push([tint.x, tint.y, tint.z, 1.]);
        }
    }

    let row_width: i64 = (size - 1) / step as i64 + 1;

    for i in 0..row_width - 1 {
        for j in 0..row_width - 1 {
            let top_left = i * row_width + j;
            let top_right = top_left + 1;
            let bottom_left = top_left + row_width;
            let bottom_right = bottom_left + 1;

            indices.push(top_left as u16);
            indices.push(top_right as u16);
            indices.push(bottom_left as u16);

            indices.push(top_right as u16);
            indices.push(bottom_right as u16);
            indices.push(bottom_left as u16);
        }
    }
    let mut mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::RENDER_WORLD,
    );

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
    mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, colors);
    mesh.insert_indices(Indices::U16(indices));
    return mesh;
}

pub fn height_at(perlin: &Perlin, params: &TerrainParams, x: f64, z: f64) -> f32 {
    let warp = 120.0;
    let (x, z) = (
        x + warp * perlin.get([x * 0.0007 + 13.7, z * 0.0007 + 5.1]),
        z + warp * perlin.get([z * 0.0007 + 41.3, x * 0.0007 + 71.9]),
    );
    let mask: f64 =
        1. - ((perlin.get([x / params.continentality, z / params.continentality]) / 2.) + 0.5);
    let t: f64 = ((mask - params.edge0) / (params.edge1 - params.edge0)).clamp(0., 1.);

    let plains = fbm(perlin, &params.plains, x, z, params);
    let blend = t * t * (3. - 2. * t);
    let y = if blend > 0. {
        let mountains = fbm(perlin, &params.mountains, x, z, params);
        plains + blend * (mountains - plains)
    } else {
        plains
    };

    y as f32
}

fn fbm(perlin: &Perlin, biome: &Biome, x: f64, z: f64, params: &TerrainParams) -> f64 {
    let mut y: f64 = 0.0;
    let mut frequency = biome.frequency;
    let mut amplitude: f64 = biome.amplitude as f64;
    let mut weight: f64 = 1.0;

    for i in 0..biome.octaves {
        let o: f64 = i as f64 * 503.2099;
        let perlin_x = x * frequency + o;
        let perlin_z = z * frequency - o;
        let perlin_result = perlin.get([perlin_x, perlin_z]);
        if biome.noise_shape == NoiseShape::Standard {
            y += perlin_result * amplitude;
        } else if biome.noise_shape == NoiseShape::Ridged {
            if i < 2 {
                let soft_abs =
                    (perlin_result * perlin_result + params.crease * (i + 1) as f64).sqrt();
                let ridge = (1. - soft_abs).powi(2);
                let signal = ridge * weight;
                y += signal * amplitude;
                weight = (signal * params.weight_gain).clamp(0., 1.);
            } else {
                y += perlin_result * weight * amplitude;
            }
        }
        amplitude *= biome.gain;
        frequency *= biome.lacunarity;
    }
    y
}
pub fn update_chunks(
    mut commands: Commands,
    params: Res<TerrainParams>,
    mut chunk_map: ResMut<ChunkMap>,
    camera: Single<&Transform, With<Camera3d>>,
    chunks: Query<(Entity, &Chunk)>,
) {
    let chunk_len = (CHUNK_SIZE - 1) as f32;
    let center = IVec2::new(
        (camera.translation.x / chunk_len).floor() as i32,
        (camera.translation.z / chunk_len).floor() as i32, // IVec2's .y holds z
    );

    for (entity, chunk) in chunks.iter() {
        let coord = IVec2::new(chunk.x as i32, chunk.z as i32);
        let distance = (center - coord).abs().max_element();
        if (distance > (LOAD_RADIUS * 4)) & (chunk_map.0.len() > MAX_CHUNKS as usize) {
            commands.entity(entity).despawn();
            chunk_map.0.remove(&coord);
        }
    }

    for i in -LOAD_RADIUS..=LOAD_RADIUS {
        for j in -LOAD_RADIUS..=LOAD_RADIUS {
            let coord = center + IVec2::new(i, j);
            if chunk_map.0.contains_key(&coord) {
                continue;
            }
            let (x, z) = (coord.x as i64, coord.y as i64);
            let params = *params;
            let task = AsyncComputeTaskPool::get()
                .spawn(async move { load_chunk(CHUNK_SIZE, x as i64, z as i64, &params, SPACING) });
            let entity = commands
                .spawn((
                    Chunk { x, z },
                    ChunkTask(task),
                    Transform::from_xyz(
                        coord.x as f32 * chunk_len,
                        0.0,
                        coord.y as f32 * chunk_len,
                    ),
                ))
                .id();
            chunk_map.0.insert(coord, entity);
        }
    }
}

pub fn finish_chunks(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut chunk_tasks: Query<(Entity, &mut ChunkTask)>,
    grass_material: Res<GrassMaterial>,
) {
    for (entity, mut task) in &mut chunk_tasks {
        if let Some((mesh, heights)) = check_ready(&mut task.0) {
            commands.entity(entity).remove::<ChunkTask>();
            commands.entity(entity).insert((
                Mesh3d(meshes.add(mesh)),
                MeshMaterial3d(grass_material.0.clone()),
                ChunkHeights(heights),
                ChunkLod { level: 1 },
            ));
        }
    }
}
