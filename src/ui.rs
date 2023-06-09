use bevy::{prelude::*, utils::HashMap};
use bevy_egui::{egui, EguiContexts};
use serde_json::{json, Value};

use crate::{
    character::{Character, CharacterMovement},
    editor::EditorOptions,
    GameStates,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(editor_ui.in_set(OnUpdate(GameStates::Editor)))
            .add_system(show_percentage.in_set(OnUpdate(GameStates::Playing)));
    }
}

fn show_percentage(mut contexts: EguiContexts, character: Query<(&Name, &Character, &Transform)>) {
    egui::Window::new("Percetage").show(contexts.ctx_mut(), |ui| {
        for character in character.iter() {
            ui.label(format!("{}: {}%", character.0, character.1.percentage,));
        }
    });
}

fn editor_ui(
    mut context: EguiContexts,
    mut options: ResMut<EditorOptions>,
    mut query: Query<(&mut CharacterMovement, &mut Name), With<Character>>,
) {
    let ctx = context.ctx_mut();

    let mut save = false;
    let mut discard = false;

    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .show(ctx, |ui| {
            if options.editing_character.is_none() {
                ui.heading("Click a character to edit");
                return;
            }
            let (mut movement, mut name) =
                query.get_mut(options.editing_character.unwrap()).unwrap();

            ui.heading(format!("Editing: {}", *name));

            let mut new_name = format!("{}", *name);
            ui.horizontal(|ui| {
                ui.label("Name: ");
                ui.text_edit_singleline(&mut new_name);
            });
            name.set(new_name);

            // TODO Change this terrible code
            let mut movement_hashmap: HashMap<String, Value> =
                serde_json::from_str(&serde_json::to_string(&*movement).unwrap()).unwrap();

            for (key, value) in movement_hashmap.iter_mut() {
                ui.horizontal(|ui| {
                    ui.label(format!("{key}"));
                    match value {
                        Value::Bool(mut b) => {
                            ui.checkbox(&mut b, "");
                        }
                        Value::Number(ref n) => {
                            if n.is_u64() {
                                ui.add(egui::Slider::new(&mut n.as_u64().unwrap(), 0..=999999));
                            } else {
                                ui.add(egui::Slider::new(&mut n.as_f64().unwrap(), 0.0..=999999.));
                            }
                        }
                        _ => {}
                    }
                });
            }

            *movement =
                serde_json::from_str(&serde_json::to_string(&movement_hashmap).unwrap()).unwrap();

            ui.horizontal(|ui| {
                ui.label("Air speed: ");
                ui.add(egui::Slider::new(&mut movement.speed_air, 0.0..=f32::MAX));
            });

            ui.horizontal(|ui| {
                ui.label("Floor speed: ");
                ui.add(egui::Slider::new(&mut movement.speed_floor, 0.0..=f32::MAX));
            });

            ui.horizontal(|ui| {
                ui.label("Max air Speed: ");
                ui.add(egui::Slider::new(
                    &mut movement.max_speed_air,
                    0.0..=f32::MAX,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("fastfall_initial_speed: ");
                ui.add(egui::Slider::new(
                    &mut movement.fastfall_initial_speed,
                    0.0..=f32::MAX,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("normal_gravity: ");
                ui.add(egui::Slider::new(
                    &mut movement.normal_gravity,
                    0.0..=f32::MAX,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("fastfalling_gravity: ");
                ui.add(egui::Slider::new(
                    &mut movement.fastfalling_gravity,
                    0.0..=f32::MAX,
                ));
            });

            ui.horizontal(|ui| {
                ui.label("jump_boost: ");
                ui.add(egui::Slider::new(&mut movement.jump_boost, 0.0..=f32::MAX));
            });

            ui.horizontal(|ui| {
                ui.label("max_air_jumps: ");
                ui.add(egui::Slider::new(
                    &mut movement.max_air_jumps,
                    0..=usize::MAX,
                ));
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut movement.can_walljump, "Can Walljump");
            });

            ui.allocate_space(egui::Vec2::new(1.0, 100.0));

            ui.horizontal(|ui| {
                save = ui.button("Save Changes to File").clicked();
                discard = ui.button("Discard Changes").clicked();
            });

            ui.allocate_space(egui::Vec2::new(1.0, 10.0));

            ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                ui.label("Made by a bandana dee")
            });
        });
}
