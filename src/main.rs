use std::f32::consts::PI;

mod camera;
mod chunk;
use crate::chunk::load_chunk;
use bevy::{
    image::{ImageAddressMode, ImageLoaderSettings, ImageSampler, ImageSamplerDescriptor},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
};
use camera::{FreeCamera, OrbitCamera};

use iyes_perf_ui::{prelude::PerfUiAllEntries, PerfUiPlugin};
use rand::random;

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;
const WORLD_SIZE: u8 = 8; // World size in chunks
const CHUNK_SIZE: u64 = 512; // Chunk size in points (vertices)
pub const GRASS_TILE_WORLD_SIZE: f32 = 8.0; // World units covered by one repeat of the grass texture

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
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(camera::CameraPlugin)
        .add_plugins(PerfUiPlugin::default())
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

    let random = random(); // Create random seed and pass it into each chunk
    for i in 0..WORLD_SIZE {
        for j in 0..WORLD_SIZE {
            commands.spawn((
                // Create chunks, letting  each one know its offset in the perlin noise seed
                // i = x offset
                // j = z offset
                // random = seed
                Mesh3d(asset_server.add(load_chunk(CHUNK_SIZE, i as u64, j as u64, random))),
                MeshMaterial3d(grass_material.clone()),
                Transform::from_xyz(
                    i as f32 * (CHUNK_SIZE - 1) as f32,
                    0.0,
                    j as f32 * (CHUNK_SIZE - 1) as f32,
                    // Transform each chunk to appropriate spot, but offset by one to get rid of
                    // seams.
                ),
            ));
        }
    }
    // Sky color
    commands.insert_resource(ClearColor(Color::srgb(0.4, 0.6, 0.9)));
    commands.spawn((
        Camera3d { ..default() }, // Spawn a new camera
        FreeCamera { ..default() },
        OrbitCamera { ..default() },
    ));
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.8), // slightly warm sunlight
            illuminance: 10000.0,
            shadows_enabled: false,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 200.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
        CascadeShadowConfigBuilder {
            first_cascade_far_bound: 500.0,
            ..default()
        }
        .build(),
    ));

    commands.spawn(PerfUiAllEntries::default()); // FPS helper
}

/// Loads the grass tile with a Repeat sampler so UVs past 1.0 wrap around
/// instead of clamping to the edge pixel. The settings closure runs as part
/// of the load itself, so no follow-up system is needed.
fn load_grass_texture(asset_server: &AssetServer) -> Handle<Image> {
    asset_server.load_with_settings("grass_tile.png", |settings: &mut ImageLoaderSettings| {
        settings.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..default()
        });
    })
}
