use bevy::prelude::*;

const CAMERA_INITAL_DISTANCE: f32 = 10.;
const CAMERA_INITAL_HEIGHT: f32 = 2.;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera);
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(0., CAMERA_INITAL_HEIGHT, CAMERA_INITAL_DISTANCE)
            .looking_at(Vec3::new(0., CAMERA_INITAL_HEIGHT, 0.), Vec3::Y),
        ..Default::default()
    });
}
