use std::f32::consts::PI;

use bevy::{
    math::Quat,
    prelude::{Component, Transform, Vec2, Vec3},
};

#[derive(Component, Default)]
pub struct CameraController {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
    pub velocity: Vec2,
    pub smoothing: f32,
    pub speed: f32,
    pub zoom: f32,
    // New controller methods:
    pub focus_point: Vec3,
    pub zoom2: f32,
    pub orbit: f32,
}

impl CameraController {
    pub fn zoom_in(&mut self, multiplier: f32) {
        let mut tentative_zoom: f32 = self.zoom;
        tentative_zoom += 1.0 * multiplier;
        self.zoom = tentative_zoom.clamp(0.0, 100.0);
    }

    pub fn zoom_out(&mut self, multiplier: f32) {
        let mut tentative_zoom: f32 = self.zoom;
        tentative_zoom -= 1.0 * multiplier;
        self.zoom = tentative_zoom.clamp(0.0, 100.0);
    }

    pub fn forward(&mut self, delta: f32) {
        self.focus_point.x -= delta * self.speed * self.yaw.sin();
        self.focus_point.y -= delta * self.speed * self.yaw.cos();
    }

    pub fn left(&mut self, delta: f32) {
        // Swap the cos() and sin() functions here and then negate the sin()
        // Found this trick online, do the opposite for D key to strafe right
        self.focus_point.x -= delta * self.speed * self.yaw.cos();
        self.focus_point.y += delta * self.speed * self.yaw.sin();
    }

    pub fn right(&mut self, delta: f32) {
        self.focus_point.x += delta * self.speed * self.yaw.cos();
        self.focus_point.y -= delta * self.speed * self.yaw.sin();
    }

    pub fn back(&mut self, delta: f32) {
        self.focus_point.x += delta * self.speed * self.yaw.sin();
        self.focus_point.y += delta * self.speed * self.yaw.cos();
    }

    pub fn rotate_x(&mut self, multiplier: f32, current_location: Transform) {
        self.orbit += multiplier * self.sensitivity;
        self.yaw = Transform::looking_at(current_location, self.focus_point, Vec3::Y)
            .rotation
            .x;
    }

    pub fn update_transform(&mut self, transform: &mut Transform) {
        let x = self.focus_point.x - self.orbit.sin() * self.zoom;
        let y = self.zoom;
        let z = self.focus_point.y - self.orbit.cos() * self.zoom;

        *transform = Transform::from_xyz(x, y, z).looking_at(self.focus_point, Vec3::Y);
        self.yaw = transform.rotation.x;
        self.pitch = transform.rotation.z;
    }
}
