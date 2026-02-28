use std::f32::consts::PI;

use bevy::{
    math::Quat,
    prelude::{Component, Transform, Vec2},
};

#[derive(Component)]
pub struct CameraController {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
    pub velocity: Vec2,
    pub speed: f32,
}

impl Default for CameraController {
    fn default() -> Self {
        Self {
            pitch: -PI / 6.,
            yaw: 0.,
            sensitivity: 0.002,
            velocity: Vec2::ZERO,
            speed: 50.0,
            x: 0.0,
            y: 100.0,
            z: 100.0,
        }
    }
}
impl CameraController {
    pub fn forward(&mut self, delta: f32) {
        self.x -= delta * self.speed * self.yaw.sin();
        self.z -= delta * self.speed * self.yaw.cos();
    }

    pub fn up(&mut self, delta: f32) {
        self.y += delta * self.speed;
    }

    pub fn down(&mut self, delta: f32) {
        self.y -= delta * self.speed;
    }

    pub fn left(&mut self, delta: f32) {
        self.x -= delta * self.speed * self.yaw.cos();
        self.z += delta * self.speed * self.yaw.sin();
    }

    pub fn right(&mut self, delta: f32) {
        self.x += delta * self.speed * self.yaw.cos();
        self.z -= delta * self.speed * self.yaw.sin();
    }

    pub fn back(&mut self, delta: f32) {
        self.x += delta * self.speed * self.yaw.sin();
        self.z += delta * self.speed * self.yaw.cos();
    }

    pub fn rotate_mouse(&mut self, multiplier: Vec2) {
        self.yaw -= multiplier.x * self.sensitivity;
        self.pitch -= multiplier.y * self.sensitivity;
    }

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
