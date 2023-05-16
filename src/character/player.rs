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
    Move,
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
                (KeyCode::S, PlayerActions::FastFall),
            ])
            .insert(DualAxis::left_stick(), PlayerActions::Move)
            .insert(GamepadButtonType::West, PlayerActions::Jump)
            .insert(GamepadButtonType::North, PlayerActions::Jump)
            .build(),
        })
        .insert(Player);
}

fn player_movement(
    fastfall_query: Query<&ActionState<PlayerActions>, With<Player>>,
    mut player_query: Query<&mut Character, With<Player>>,
) {
    let action_state = fastfall_query.single();
    let axis_pair = action_state.clamped_axis_pair(PlayerActions::Move).unwrap();
    let mut character = player_query.single_mut();

    if action_state.pressed(PlayerActions::Move) {
        // Sides
        character.movement.x = axis_pair.x().clamp(-1., 1.);

        // Fast Fall
        if !character.movement.is_fastfalling
            && axis_pair.y() < -FASTFALL_THRESHOLD
            && character.is_on_air()
        {
            character.movement.wants_to_fastfall = true;
        }
    } else {
        character.movement.x = 0.;
    }

    // Jump
    if action_state.just_pressed(PlayerActions::Jump) {
        character.movement.wants_to_jump = true;
    }
}

// println!("Move:");
// println!("   distance: {}", axis_pair.length());
// println!("          x: {}", axis_pair.x());
// println!("          y: {}", axis_pair.y());
