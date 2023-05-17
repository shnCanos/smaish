use std::dbg;

use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use super::Character;

// Most of these constants will change according to the character.
// They're placeholders
// const PLAYER_JUMP: f32 = 10.;
// const PLAYER_FAST_FALL: f32 = 5.;
// const PLAYER_SPEED_AIR: f32 = 5.;
// const PLAYER_SPEED_FLOOR: f32 = 10.;
const FASTFALL_THRESHOLD: f32 = 0.5;
// How fast you need to move the stick to fastfall
const STICK_MOVEMENT_NEEDED_TO_FASTFALL: f32 = 0.1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_plugin(InputManagerPlugin::<PlayerActions>::default())
            .add_system(player_movement);
    }
}

#[derive(Component)]
struct Player;

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum PlayerActions {
    /// For the keyboard
    MoveRight,
    MoveLeft,
    /// For the controller
    MoveStick,
    Jump,
    NormalAttack,
    SpecialAttack,
    FastFall,
}

fn setup_player(mut commands: Commands, asset_server: Res<AssetServer>) {
    let character = Character::default();
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(100., 100.)),
                ..default()
            },
            texture: asset_server.load("bandanadee.png"),
            ..default()
        },
        RigidBody::Dynamic,
        Player,
        Collider::cuboid(50., 50.),
        // GravityScale(10.),
        Velocity {
            linvel: Vec2::ZERO,
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        character.clone(),
        GravityScale(character.movement.normal_gravity),
    ));

    commands
        .spawn(InputManagerBundle::<PlayerActions> {
            // Stores "which actions are currently pressed"
            action_state: ActionState::default(),
            input_map: InputMap::new([
                (KeyCode::Space, PlayerActions::Jump),
                (KeyCode::W, PlayerActions::Jump),
                (KeyCode::S, PlayerActions::FastFall),
                (KeyCode::A, PlayerActions::MoveLeft),
                (KeyCode::D, PlayerActions::MoveRight),
            ])
            .insert(DualAxis::left_stick(), PlayerActions::MoveStick)
            .insert(GamepadButtonType::West, PlayerActions::Jump)
            .insert(GamepadButtonType::North, PlayerActions::Jump)
            .build(),
        })
        .insert(Player);
}

fn player_movement(
    action_state_query: Query<&ActionState<PlayerActions>, With<Player>>,
    mut player_query: Query<&mut Character, With<Player>>,
    mut last_stick_position: Local<f32>,
    mut last_stick_position_but_keyboard: Local<bool>,
    time: Res<Time>,
) {
    // Controller Movement
    let action_state = action_state_query.single();
    let axis_pair = action_state
        .clamped_axis_pair(PlayerActions::MoveStick)
        .unwrap();
    let mut character = player_query.single_mut();

    if action_state.pressed(PlayerActions::MoveStick) {
        // Sides
        character.movement.x = axis_pair.x().clamp(-1., 1.);

        // Fast Fall
        if !character.movement.is_fastfalling
            && axis_pair.y() < -FASTFALL_THRESHOLD
            // Honestly I have no clue
            && axis_pair.y() - *last_stick_position
                < -STICK_MOVEMENT_NEEDED_TO_FASTFALL * time.delta_seconds()
        {
            character.movement.wants_to_fastfall = true;
        }
    } else {
        character.movement.x = 0.;
    }
    *last_stick_position = axis_pair.y();

    // Keyboard Movement
    let mut movement = 0.;
    if action_state.pressed(PlayerActions::MoveLeft) {
        movement += -1.;
    }
    if action_state.pressed(PlayerActions::MoveRight) {
        movement += 1.;
    }
    if action_state.pressed(PlayerActions::FastFall) && !*last_stick_position_but_keyboard {
        character.movement.wants_to_fastfall = true;
    }
    character.movement.x = movement;
    *last_stick_position_but_keyboard = action_state.pressed(PlayerActions::FastFall);

    // Jump
    if action_state.just_pressed(PlayerActions::Jump) {
        character.movement.wants_to_jump = true;
    }
}

// println!("Move:");
// println!("   distance: {}", axis_pair.length());
// println!("          x: {}", axis_pair.x());
// println!("          y: {}", axis_pair.y());
