mod camera;
mod character;
mod stage;

use bevy::prelude::*;
// use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy_rapier2d::prelude::*;

fn setup_window(mut window: Query<&mut Window>) {
    let mut window = window.get_single_mut().unwrap();
    window.title = "UwU".to_string();
    window.decorations = true; // So the UwU appears, of course
    window.mode = bevy::window::WindowMode::Windowed;
}

fn main() {
    App::new()
        .add_startup_system(setup_window)
        .add_plugins(
            DefaultPlugins, // .build()
                            // .add_before::<bevy::asset::AssetPlugin, _>(EmbeddedAssetPlugin),
        )
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(camera::CameraPlugin)
        .add_plugin(stage::StagePlugin)
        .add_plugin(character::CharacterPlugin)
        .run();
}
