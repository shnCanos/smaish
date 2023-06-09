use bevy::prelude::*;
use bevy_rapier2d::prelude::RapierConfiguration;
use leafwing_input_manager::prelude::*;

use crate::{character::Character, GameStates};

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(editor_setup)
            .add_plugin(InputManagerPlugin::<EditorActions>::default())
            .add_systems((editor_toggle, editor_main, pick_character))
            .insert_resource(EditorOptions::default());
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum EditorActions {
    Toggle,
}

#[derive(Default, Resource)]
pub struct EditorOptions {
    pub editing_character: Option<Entity>,
}

fn editor_setup(mut commands: Commands) {
    commands.spawn(InputManagerBundle::<EditorActions> {
        input_map: InputMap::new([(KeyCode::Escape, EditorActions::Toggle)])
            .insert(GamepadButtonType::Start, EditorActions::Toggle)
            .build(),
        ..default()
    });
}

fn editor_toggle(
    state: Res<State<GameStates>>,
    mut next_state: ResMut<NextState<GameStates>>,
    mut rapier_configuration: ResMut<RapierConfiguration>,
    action_state_query: Query<&ActionState<EditorActions>>,
    mut options: ResMut<EditorOptions>,
) {
    let action_state: &ActionState<EditorActions> = action_state_query.single();

    if action_state.just_pressed(EditorActions::Toggle) {
        *options = EditorOptions::default();
        match state.0 {
            GameStates::Playing => {
                rapier_configuration.physics_pipeline_active = false;
                next_state.set(GameStates::Editor);
            }
            GameStates::Editor => {
                rapier_configuration.physics_pipeline_active = true;
                next_state.set(GameStates::Playing);
            }
        }
    }
}

fn editor_main() {}

fn pick_character(
    window: Query<&Window>,
    camera: Query<(&Camera, &GlobalTransform)>,
    kb: Res<Input<MouseButton>>,
    characters: Query<(Entity, &GlobalTransform), With<Character>>,
    mut options: ResMut<EditorOptions>,
) {
    let window = window.single();
    let (camera, gtansf) = camera.single();

    if let Some(position) = window
        .cursor_position()
        .and_then(|cursor| camera.viewport_to_world(gtansf, cursor))
        .map(|ray| ray.origin.truncate())
    {
        if kb.just_pressed(MouseButton::Left) {
            for (entity, gtransf) in characters.iter() {
                // TODO
                if gtransf.translation().truncate().distance(position) < 50. {
                    options.editing_character = Some(entity);
                }
            }
        }
    }
}
