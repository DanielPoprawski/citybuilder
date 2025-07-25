use std::f32::consts::PI;

mod camera_controller;
use crate::camera_controller::*;
use bevy::{
    color::palettes::css::*,
    core_pipeline::bloom::Bloom,
    input::mouse::{MouseMotion, MouseWheel},
    pbr::CascadeShadowConfigBuilder,
    prelude::*,
    transform,
    window::CursorGrabMode,
};
const WINDOW_WIDTH: f32 = 1920.0;
const WINDOW_HEIGHT: f32 = 1080.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "My City Builder".into(),
                resolution: (WINDOW_WIDTH, WINDOW_HEIGHT).into(),
                present_mode: bevy::window::PresentMode::AutoVsync,
                ..default()
            }),
            ..Default::default()
        }))
        .add_systems(Startup, startup)
        .add_systems(Update, update)
        .add_systems(Update, handle_input)
        // .add_systems(Update, handle_light)
        .run();
}

fn startup(mut commands: Commands, asset_server: Res<AssetServer>) {
    let pitch: f32 = -PI / 4.0;
    let yaw: f32 = PI / 4.0;
    commands.spawn((
        Camera3d { ..default() },
        CameraController {
            pitch: pitch,
            yaw: yaw,
            sensitivity: 0.002,
            velocity: Vec2::ZERO,
            smoothing: 0.15,
            speed: 50.0,
            zoom: 50.0,
            focus_point: Vec3::ZERO,
            orbit: 0.0,
            zoom2: 0.0,
        },
        Transform {
            translation: Vec3 {
                x: 77.0,
                y: 100.0,
                z: 77.0,
            },
            rotation: Quat::from_rotation_x(pitch) * Quat::from_rotation_y(yaw),
            ..default()
        },
    ));
    commands.spawn(Bloom {
        intensity: 0.3,
        low_frequency_boost: 0.7,
        low_frequency_boost_curvature: 0.95,
        high_pass_frequency: 1.0,
        composite_mode: bevy::core_pipeline::bloom::BloomCompositeMode::Additive,
        ..default()
    });
    commands.spawn((SceneRoot(
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("map.glb")),
    ),));
    commands.insert_resource(AmbientLight {
        color: WHITE.into(),
        brightness: 10.,
        ..default()
    });
    commands.spawn((
        DirectionalLight {
            illuminance: light_consts::lux::OVERCAST_DAY,
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
}
fn update(
    mut text_query: Query<&mut Text>,
    camera_query: Query<(&Transform, &CameraController), With<Camera3d>>,
) {
    for (transform, camera_controller) in camera_query {
        let trans = transform.translation;
        for mut text in text_query.iter_mut() {
            text.clear();
            text.push_str(&format!(
                "x: {}\ny: {}\nz: {}\npitch: {}\nyaw: {}\norbit: {}",
                trans.x,
                trans.y,
                trans.z,
                camera_controller.pitch,
                camera_controller.yaw,
                camera_controller.orbit
            ));
        }
    }
}

fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_movement: EventReader<MouseMotion>,
    mut mousewheel_event: EventReader<MouseWheel>,
    mut query: Query<(&mut Transform, &mut CameraController), With<Camera3d>>,
    // light: ResMut<AmbientLight>,
    time: Res<Time>,
    mut windows: Query<&mut Window>,
) {
    for (mut transform, mut camera_controller) in query.iter_mut() {
        //TODO: Create a normalized movement vector. Declare a new Vec3 here at zero, then add movements to
        //the movement vector, then at the bottom of the function normalize the vector and THEN add to the
        //movement translation

        if keyboard_input.pressed(KeyCode::ShiftLeft) {
            camera_controller.speed = 100.0;
        } else {
            camera_controller.speed = 50.0;
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            camera_controller.forward(time.delta_secs());
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

        // for event in mousewheel_event.read() {
        //     if event.y < 0.0 {
        //         camera_controller.zoom_in(10.0);
        //     } else if event.y > 0.0 {
        //         camera_controller.zoom_out(10.0);
        //     }
        //     transform.translation.y = 10.0 + camera_controller.zoom;
        // }
        //
        if mouse_input.pressed(MouseButton::Right) {
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
            }

            let mut cumulative_movement = Vec2::ZERO;
            for event in mouse_movement.read() {
                cumulative_movement += event.delta;
            }

            camera_controller.rotate_x(cumulative_movement.x, transform);

            // camera_controller.velocity = camera_controller.velocity * camera_controller.smoothing
            //     + cumulative_movement * (1.0 - camera_controller.smoothing);
            // camera_controller.yaw -= camera_controller.velocity.x * camera_controller.sensitivity;
            // camera_controller.pitch -= camera_controller.velocity.y * camera_controller.sensitivity;
            //
            // camera_controller.pitch = camera_controller.pitch.clamp(-PI / 2.0, 0.0);

            if camera_controller.yaw > 2.0 * PI {
                camera_controller.yaw -= 2.0 * PI;
            }
            if camera_controller.yaw < 0.0 {
                camera_controller.yaw += 2.0 * PI;
            }
        } else {
            mouse_movement.clear();
            camera_controller.velocity *= 0.2;
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
            }
        }
        // let yaw_quat = Quat::from_rotation_y(camera_controller.yaw);
        // let pitch_quat = Quat::from_rotation_x(camera_controller.pitch);
        // transform.rotation = yaw_quat * pitch_quat;
    }
}
