use bevy::prelude::{Component, Vec2};

#[derive(Component, Default)]
pub struct CameraController {
    pub pitch: f32,
    pub yaw: f32,
    pub sensitivity: f32,
    pub velocity: Vec2,
    pub smoothing: f32,
    pub speed: f32,
    pub zoom: f32,
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
}
