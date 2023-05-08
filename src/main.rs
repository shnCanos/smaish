mod camera;
mod player;
mod stage;

use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

fn setup_window(mut window: Query<&mut Window>) {
    let mut window = window.get_single_mut().unwrap();
    window.title = "UwU".to_string();
    window.decorations = true;
    window.mode = bevy::window::WindowMode::Windowed;
}

fn main() {
    App::new()
        .add_startup_system(setup_window)
        .add_plugins(DefaultPlugins)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(camera::CameraPlugin)
        .add_plugin(stage::StagePlugin)
        .add_plugin(player::PlayerPlugin)
        .run();
}
