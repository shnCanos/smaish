use bevy::prelude::*;
use bevy_rapier2d::prelude::*;
use leafwing_input_manager::prelude::*;

use crate::camera::CameraFollows;

use super::{CharacterBundle, CharacterMovement};

const FASTFALL_THRESHOLD: f32 = 0.5;
// How fast you need to move the stick to fastfall
const STICK_MOVEMENT_NEEDED_TO_FASTFALL: f32 = 0.1;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_startup_system(setup_dummy)
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
    let character = CharacterBundle {
        grav: GravityScale(20.),
        ..default()
    };

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
        LockedAxes::ROTATION_LOCKED,
        character.clone(),
        CameraFollows { padding: 250 },
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

fn setup_dummy(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                custom_size: Some(Vec2::new(50., 50.)),
                ..default()
            },
            texture: asset_server.load("bandanadee.png"),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::cuboid(25., 25.),
        LockedAxes::ROTATION_LOCKED,
        CharacterBundle::default(),
        CameraFollows { padding: 250 },
    ));
}

fn player_movement(
    action_state_query: Query<&ActionState<PlayerActions>, With<Player>>,
    mut player_query: Query<&mut CharacterMovement, With<Player>>,
    mut last_stick_position: Local<f32>,
    time: Res<Time>,
) {
    // Controller Movement
    let action_state = action_state_query.single();
    let axis_pair = action_state
        .clamped_axis_pair(PlayerActions::MoveStick)
        .unwrap();
    let mut movement = player_query.single_mut();

    if action_state.pressed(PlayerActions::MoveStick) {
        // Sides
        movement.x = axis_pair.x().clamp(-1., 1.);

        // Fast Fall
        if !movement.is_fastfalling
            && axis_pair.y() < -FASTFALL_THRESHOLD
            // Honestly I have no clue
            && axis_pair.y() - *last_stick_position
                < -STICK_MOVEMENT_NEEDED_TO_FASTFALL * time.delta_seconds()
        {
            movement.fastfall();
        }
    } else {
        movement.x = 0.;
    }
    *last_stick_position = axis_pair.y();

    // Keyboard Movement
    let mut direction = 0.;
    if action_state.pressed(PlayerActions::MoveLeft) {
        direction += -1.;
    }
    if action_state.pressed(PlayerActions::MoveRight) {
        direction += 1.;
    }
    if direction != 0. {
        movement.x = direction;
        *last_stick_position = action_state.pressed(PlayerActions::FastFall).into();
    }
    if action_state.pressed(PlayerActions::FastFall) && *last_stick_position >= 0. {
        movement.fastfall();
    }

    // Jump
    if action_state.just_pressed(PlayerActions::Jump) {
        movement.jump()
    }
}
