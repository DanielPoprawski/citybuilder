mod free;
mod orbit;

use bevy::prelude::*;
pub use free::FreeCamera;
pub use orbit::OrbitCamera;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default, States)]
pub enum CameraMode {
    Free,
    #[default]
    Orbit,
}

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<CameraMode>()
            .add_systems(Update, toggle_camera_mode)
            .add_systems(Update, orbit::update.run_if(in_state(CameraMode::Orbit)))
            .add_systems(Update, free::update.run_if(in_state(CameraMode::Free)));
    }
}

fn toggle_camera_mode() {}
