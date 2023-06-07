mod camera;
mod character;
mod stage;
mod ui;

use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPlugin};
// use bevy_embedded_assets::EmbeddedAssetPlugin;
use bevy::window::Window;
use bevy_rapier2d::prelude::*;

fn setup_window(mut window: Query<&mut Window>) {
    let mut window = window.single_mut();
    window.title = "SUwUssy PeidrOwO".to_string();
    window.decorations = true; // So the SUwUssy PeidrOwO appears, of course
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
        .add_plugin(EguiPlugin)
        .add_plugin(camera::CameraPlugin)
        .add_plugin(stage::StagePlugin)
        .add_plugin(character::CharacterPlugin)
        .add_plugin(ui::UiPlugin)
        // .add_plugin(WorldInspectorPlugin::new())
        .run();
}
