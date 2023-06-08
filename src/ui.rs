use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts};

use crate::character::Character;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(show_percentage);
    }
}

fn show_percentage(mut contexts: EguiContexts, character: Query<(&Name, &Character, &Transform)>) {
    egui::Window::new("Percetage").show(contexts.ctx_mut(), |ui| {
        for character in character.iter() {
            ui.label(format!("{}: {}%", character.0, character.1.percentage,));
        }
    });
}
