use std::f32::consts::PI;

use bevy::input::{mouse::MouseMotion, ButtonInput};
use bevy::window::{CursorGrabMode, Window};
use bevy::{math::Quat, prelude::*};

#[derive(Component)]
pub struct OrbitCamera {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub distance: f32,
    pub sensitivity: f32,
    pub velocity: Vec2,
    pub speed: f32,
}

impl OrbitCamera {
    pub fn update_transform(&mut self, transform: &mut Transform) {
        self.pitch = self.pitch.clamp(-PI / 2., PI / 2.);
        // Lock camera from going to high, or too
        // low, ensures camera doesn't flip upside
        // down

        transform.translation.x = self.x;
        transform.translation.y = self.y;
        transform.translation.z = self.z;

        let yaw = Quat::from_rotation_y(self.yaw);
        let pitch = Quat::from_rotation_x(self.pitch);

        transform.rotation = yaw * pitch;
    }
}

impl Default for OrbitCamera {
    fn default() -> Self {
        Self {
            pitch: -PI / 6.,
            yaw: 0.,
            distance: 50.0,
            sensitivity: 0.002,
            velocity: Vec2::ZERO,
            speed: 50.0,
            x: 0.0,
            y: 100.0,
            z: 100.0,
        }
    }
}
pub(super) fn update(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mouse_input: Res<ButtonInput<MouseButton>>,
    mut mouse_movement: EventReader<MouseMotion>,
    mut query: Query<(&mut Transform, &mut OrbitCamera), With<Camera3d>>,
    time: Res<Time>,
    mut windows: Query<&mut Window>,
) {
    let delta = time.delta_secs();
    for (mut transform, mut controller) in query.iter_mut() {
        if keyboard_input.pressed(KeyCode::KeyF) {
            controller.speed = 300.0;
        } else {
            controller.speed = 100.0;
        }

        if keyboard_input.pressed(KeyCode::KeyW) {
            controller.x -= delta * controller.speed * controller.yaw.sin();
            controller.z -= delta * controller.speed * controller.yaw.cos();
        }

        if keyboard_input.pressed(KeyCode::Space) {
            controller.y += delta * controller.speed;
        }

        if keyboard_input.pressed(KeyCode::ControlLeft) {
            controller.y -= delta * controller.speed;
        }

        if keyboard_input.pressed(KeyCode::KeyA) {
            controller.x -= delta * controller.speed * controller.yaw.cos();
            controller.z += delta * controller.speed * controller.yaw.sin();
        }

        if keyboard_input.pressed(KeyCode::KeyS) {
            controller.x += delta * controller.speed * controller.yaw.sin();
            controller.z += delta * controller.speed * controller.yaw.cos();
        }

        if keyboard_input.pressed(KeyCode::KeyD) {
            controller.x += delta * controller.speed * controller.yaw.cos();
            controller.z -= delta * controller.speed * controller.yaw.sin();
        }

        controller.update_transform(&mut transform);

        if mouse_input.pressed(MouseButton::Right) {
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.grab_mode = CursorGrabMode::Locked;
                window.cursor_options.visible = false;
            }

            let mut cumulative_movement = Vec2::ZERO;
            for event in mouse_movement.read() {
                cumulative_movement += event.delta;
            }

            controller.yaw -= cumulative_movement.x * controller.sensitivity;
            controller.pitch -= cumulative_movement.y * controller.sensitivity;

            if controller.yaw.abs() > 2.0 * PI {
                controller.yaw -= 2.0 * PI * controller.yaw.signum();
            }
        } else {
            mouse_movement.clear();
            controller.velocity *= 0.2;
            if let Ok(mut window) = windows.single_mut() {
                window.cursor_options.grab_mode = CursorGrabMode::None;
                window.cursor_options.visible = true;
            }
        }
    }
}
