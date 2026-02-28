use std::f32::consts::PI;

mod camera_controller;
mod chunk;
use crate::{camera_controller::*, chunk::load_chunk};
use bevy::{
    color::palettes::css::*, input::mouse::MouseMotion, pbr::CascadeShadowConfigBuilder,
    prelude::*, window::CursorGrabMode,
};

use iyes_perf_ui::{
    prelude::{PerfUiAllEntries, PerfUiEntryFPS},
    PerfUiPlugin,
};
use rand::random;

const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;
const WORLD_SIZE: u8 = 8; // World size in chunks
const CHUNK_SIZE: u64 = 512; // Chunk size in points (vertices)

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
        // .add_systems(Update, update)
        .add_systems(Update, handle_input)
        .add_plugins(bevy::diagnostic::FrameTimeDiagnosticsPlugin::default())
        .add_plugins(bevy::diagnostic::SystemInformationDiagnosticsPlugin)
        .add_plugins(bevy::render::diagnostic::RenderDiagnosticsPlugin)
        .add_plugins(PerfUiPlugin::default())
        .run();
}

fn startup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut gizmo_assets: ResMut<Assets<GizmoAsset>>,
) {
    let random = random(); // Create random seed and pass it into each chunk
    for i in 0..WORLD_SIZE {
        for j in 0..WORLD_SIZE {
            commands.spawn((
                // Create chunks, letting  each one know its offset in the perlin noise seed
                // i = x offset
                // j = z offset
                // random = seed
                Mesh3d(asset_server.add(load_chunk(CHUNK_SIZE, i as u64, j as u64, random))),
                MeshMaterial3d(materials.add(StandardMaterial {
                    base_color: GREEN.into(),  // Base color but this is overwritten in chunk.rs
                    perceptual_roughness: 1.0, // Prevents map from looking shiny
                    ..Default::default()
                })),
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
        CameraController { ..default() },
    ));
    //
    // Spawns lighting
    //
    // commands.spawn(Bloom {
    //     intensity: 0.3,
    //     low_frequency_boost: 0.7,
    //     low_frequency_boost_curvature: 0.95,
    //     high_pass_frequency: 1.0,
    //     composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::Additive,
    //     ..default()
    // });
    commands.spawn((
        DirectionalLight {
            color: Color::srgb(1.0, 0.95, 0.8), // slightly warm sunlight
            illuminance: 10000.0,
            shadows_enabled: true,
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

    commands.spawn((
        Text::new("Testing \nTesting 2"),
        TextColor(Color::WHITE),
        TextFont {
            font: Default::default(),
            font_size: 24.0,
            ..default()
        },
        TextLayout::new_with_justify(JustifyText::Left),
    ));

    //
    // Spawns all gizmos and helpers
    //

    let mut gizmo = GizmoAsset::new();
    gizmo
        .sphere(Isometry3d::IDENTITY, 0.5, CRIMSON)
        .resolution(10_000);

    commands.spawn((
        Gizmo {
            handle: gizmo_assets.add(gizmo),
            line_config: GizmoLineConfig {
                width: 5.,
                ..default()
            },
            ..default()
        },
        Transform::from_xyz(0., 50., 0.),
    ));

    commands.spawn(PerfUiAllEntries::default()); // FPS helper
}

// MAIN LOOP
//
//  TODO: fix this debug string
//
// fn update(
//     mut text_query: Query<&mut Text>,
//     camera_query: Query<(&Transform, &CameraController), With<Camera3d>>,
// ) {
//     for (transform, camera_controller) in camera_query {
//         let trans = transform.translation;
//         for mut text in text_query.iter_mut() {
//             text.clear();
//             text.push_str(&format!(
//                 "x: {}\ny: {}\nz: {}\npitch: {}\nyaw: {}",
//                 trans.x,
//                 trans.y,
//                 trans.z,
//                 camera_controller.pitch * 57.29578,
//                 camera_controller.yaw * 57.29578,
//             ));
//         }
//     }
// }

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_movement: EventReader<MouseMotion>,
    // mousewheel_event: EventReader<MouseWheel>,
    // DON'T DELETE THESE TWO YET
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    time: Res<Time>,
    mut windows: Query<&mut Window>,
    // text: Query<&mut Text>,
) {
    for (mut transform, mut camera_controller) in query.iter_mut() {
        //TODO: Create a normalized movement vector. Declare a new Vec3 here at zero, then add movements to
        //the movement vector, then at the bottom of the function normalize the vector and THEN add to the
        //movement translation

        if keyboard_input.pressed(KeyCode::KeyF) {
            camera_controller.speed = 300.0;
        } else {
            camera_controller.speed = 100.0;
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            camera_controller.forward(time.delta_secs());
        }

        if keyboard_input.pressed(KeyCode::Space) {
            camera_controller.up(time.delta_secs());
        }

        if keyboard_input.pressed(KeyCode::ControlLeft) {
            camera_controller.down(time.delta_secs());
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            camera_controller.left(time.delta_secs());
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            camera_controller.back(time.delta_secs());
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            camera_controller.right(time.delta_secs());
        }

        camera_controller.update_transform(&mut transform);

        if mouse_input.pressed(MouseButton::Right) {
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
            }

            let mut cumulative_movement = Vec2::ZERO;
            for event in mouse_movement.read() {
                cumulative_movement += event.delta;
            }

            camera_controller.rotate_mouse(cumulative_movement);

            if camera_controller.yaw.abs() > 2.0 * PI {
                camera_controller.yaw -= 2.0 * PI * camera_controller.yaw.signum();
            }
        } else {
            mouse_movement.clear();
            camera_controller.velocity *= 0.2;
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
            }
        }
    }
}
