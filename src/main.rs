// use std::f32::consts::PI;

mod camera;
mod chunk;
mod utils;
use crate::chunk::{
    build_mesh, finish_chunks, load_chunk, update_chunks, ChunkHeights, ChunkLod, ChunkMap,
    GrassMaterial, LodTask, TerrainParams,
};
use crate::utils::*;
use bevy::{
    dev_tools::fps_overlay::FpsOverlayPlugin,
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    light::CascadeShadowConfigBuilder,
    prelude::*,
    tasks::{futures::check_ready, AsyncComputeTaskPool, ComputeTaskPool, Task},
};
use bevy_inspector_egui::bevy_egui::EguiPlugin;
use bevy_inspector_egui::quick::ResourceInspectorPlugin;
use camera::{FreeCamera, OrbitCamera};
use chunk::Chunk;
use noise::Perlin;
use std::f32::consts::PI;

const DAY_LENGTH: f32 = 300.0; // 24 hour period represented in seconds
const LATITUDE: f32 = 39.74 * PI / 180.; // Latitude used to calculate length of day
const DAY_OF_YEAR: f32 = 172.0;
const WINDOW_WIDTH: u32 = 2440;
const WINDOW_HEIGHT: u32 = 1440;
pub const CHUNK_SIZE: i64 = 129; // Chunk size in points (vertices) must be 2^n + 1
pub const LOAD_RADIUS: i32 = 24;
pub const LOD_DETAIL: f32 = 64.;
pub const MAX_CHUNKS: i32 = 2048;
pub const GRASS_TILE_WORLD_SIZE: f32 = 8.0; // World units covered by one repeat of the grass texture
pub const SPACING: f32 = 1.0;

// Coordinate System reference:
//  +/- X = East/West
//  +/- Y = Up/Down
//  +/- Z = South/North
// Pitch = vertical movement (Pitcher of water goes up/down)
// Yaw = horizontal movement
// 1 unit = 1 meter

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "My City Builder".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                present_mode: bevy::window::PresentMode::Immediate,
                ..default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, startup)
        .init_resource::<ChunkMap>()
        .add_systems(
            Update,
            (
                update_chunks,
                finish_chunks,
                update_daylight,
                update_lod,
                finish_lod,
            ),
        )
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(camera::CameraPlugin)
        .init_resource::<TerrainParams>()
        .add_plugins(EguiPlugin::default())
        .add_plugins(ResourceInspectorPlugin::<TerrainParams>::default())
        .add_plugins(FpsOverlayPlugin::default())
        .add_systems(
            Update,
            regenerate_chunks.run_if(
                resource_changed::<TerrainParams>.and_then(not(resource_added::<TerrainParams>)),
            ),
        )
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // One shared material for the whole terrain: the grayscale grass bitmap
    // tinted green by base_color, rather than 64 copies of the same thing.
    let grass_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.32, 0.62, 0.26), // green tint over the gray bitmap
        base_color_texture: Some(load_grass_texture(&asset_server)),
        perceptual_roughness: 1.0, // Prevents map from looking shiny
        ..Default::default()
    });
    commands.insert_resource(GrassMaterial(grass_material));
    // Sky color
    commands.insert_resource(ClearColor(Color::srgb(0.4, 0.6, 0.9)));
    commands.spawn((
        Camera3d { ..default() }, // Spawn a new camera
        FreeCamera { ..default() },
        OrbitCamera { ..default() },
    ));
    commands.spawn((
        Mesh3d(asset_server.add(Cuboid::new(1., 2., 1.).mesh().into())),
        MeshMaterial3d(asset_server.add(StandardMaterial::default())),
        Transform::from_xyz(100.0, 40.0, 100.0),
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.8), // slightly warm sunlight
            illuminance: 10000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::default().looking_to(Vec3::new(-0.3, -0.5, -0.3), Vec3::Y),
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 50.0,
            maximum_distance: 1000.0,
            ..default()
        }
        .build(),
    ));
}

fn regenerate_chunks(
    params: Res<TerrainParams>,
    chunks: Query<(&Chunk, &Mesh3d)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    // for (chunk, mesh3d) in &chunks {
    //     let mesh = meshes.get_mut(&mesh3d.0).unwrap();
    //     *mesh = load_chunk(CHUNK_SIZE, chunk.x, chunk.z, params.seed, &params);
    // }
    let params = &*params;
    let generated: Vec<(Mesh, Vec<f32>)> = ComputeTaskPool::get().scope(|scope| {
        for (chunk, _) in &chunks {
            scope.spawn(async move { load_chunk(CHUNK_SIZE, chunk.x, chunk.z, params, 1.) });
        }
        // TODO: iterates the query twice and zips the results (once to start the tasks, once to write the meshes). That only works if both passes return entities in the same order. It does today, but nothing guarantees it. Have each task return the mesh paired with what it belongs to, such as a cloned Handle<Mesh> moved into the async move block. Then there's no order to depend on.
    });
    for ((_, mesh3d), (mesh, _)) in chunks.iter().zip(generated) {
        *meshes.get_mut(&mesh3d.0).unwrap() = mesh;
    }
}

/// Loads the grass tile with a Repeat sampler so UVs past 1.0 wrap around
/// instead of clamping to the edge pixel. The settings closure runs as part
/// of the load itself, so no follow-up system is needed.
fn load_grass_texture(asset_server: &AssetServer) -> Handle<Image> {
    asset_server
        .load_builder()
        .with_settings(|settings: &mut ImageLoaderSettings| {
            settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
                address_mode_u: ImageAddressMode::Repeat,
                address_mode_v: ImageAddressMode::Repeat,
                ..default()
            });
        })
        .load("grass_tile.png")
}

fn update_daylight(
    mut query: Query<(&mut DirectionalLight, &mut Transform)>,
    time: Res<Time>,
    mut sky: ResMut<ClearColor>,
) {
    let time = time.elapsed_secs() + DAY_OF_YEAR * DAY_LENGTH;
    let date = time / DAY_LENGTH;
    let decl = (-23.44 * PI / 180.) * (2. * PI * (date + 10.) / 365.0).cos();
    let time_of_day = (time * 2. * PI) / (DAY_LENGTH);
    let sun = LATITUDE.sin() * decl.sin() + LATITUDE.cos() * decl.cos() * time_of_day.cos();

    let g_value = 0.85;
    let b_value = 0.70;

    let x = -(decl).cos() * time_of_day.sin();
    let y = sun;
    let z = LATITUDE.sin() * decl.cos() * time_of_day.cos() - LATITUDE.cos() * decl.sin();
    let sun_dir = Vec3::new(x, y, z);

    if let Ok((mut light, mut transform)) = query.single_mut() {
        light.illuminance = sun.max(0.0) * 8_000.0 + 2_000.0; // INFO: extra 2k is moonlight, remove when
                                                              // adding moon
        light.color = Color::srgb(
            1.0,
            (1. - g_value) * sun.clamp(0., 1.) + g_value,
            (1. - b_value) * sun.clamp(0., 1.) + b_value,
        );
        transform.look_to(-sun_dir, Vec3::Y);
    }

    let to_horizon = smoothstep(-0.1, 0.0, sun);
    let to_day = smoothstep(0.0, 0.25, sun);

    let night = Color::srgb(0.03, 0.03, 0.12);
    let horizon = Color::srgb(0.01, 0.01, 0.04);
    let daylight = Color::srgb(0.46, 0.66, 0.9);
    sky.0 = night.mix(&horizon, to_horizon).mix(&daylight, to_day);
}

fn update_lod(
    mut commands: Commands,
    params: Res<TerrainParams>,
    camera: Single<&Transform, With<Camera3d>>,
    mut chunks: Query<(Entity, &Transform, &ChunkHeights, &mut ChunkLod), Without<LodTask>>,
) {
    let chunk_len = (CHUNK_SIZE - 1) as f32;
    for (entity, transform, heights, mut lod) in &mut chunks {
        let center = transform.translation + Vec3::new(chunk_len / 2., 0., chunk_len / 2.);
        let dist = camera.translation.distance(center);
        let raw = (dist / LOD_DETAIL as f32).ln().max(0.0);
        let margin = 0.2;
        let current = lod.level as f32;
        if raw > current - margin && raw < current + 1.0 + margin {
            continue;
        }
        lod.level = raw as u32;
        let step = (1usize << lod.level).min(CHUNK_SIZE as usize - 1);

        let heights: Vec<f32> = heights.0.clone();
        let seed = params.seed;
        let x = transform.translation.x as i64;
        let z = transform.translation.z as i64;
        let task: Task<Mesh> = AsyncComputeTaskPool::get().spawn(async move {
            let perlin = Perlin::new(seed);
            return build_mesh(CHUNK_SIZE, x, z, SPACING, step, &heights, &perlin);
        });
        commands.entity(entity).insert(LodTask(task));
    }
}

pub fn finish_lod(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut lod_tasks: Query<(Entity, &Mesh3d, &mut LodTask)>,
) {
    for (entity, mesh3d, mut task) in &mut lod_tasks {
        if let Some(mesh) = check_ready(&mut task.0) {
            *meshes.get_mut(&mesh3d.0).unwrap() = mesh;
            commands.entity(entity).remove::<LodTask>();
        }
    }
}
