use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
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

fn setup_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(0.0, 0.5, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Player,
        Collider::cuboid(0.5, 0.5, 0.5),
        // GravityScale(10.),
        Velocity {
            linvel: Vec3::new(0., 0., 0.),
            ..default()
        },
        LockedAxes::ROTATION_LOCKED,
        Character::default(),
        GravityScale(0.5),
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
        character.movement_x = axis_pair.x().clamp(-1., 1.);

        // Fast Fall
        if !character.is_fastfalling && axis_pair.y() < -FASTFALL_THRESHOLD && character.is_on_air()
        {
            character.is_fastfalling = true;
        }
    } else {
        character.movement_x = 0.;
    }

    // Jump
    if action_state.just_pressed(PlayerActions::Jump) && character.current_air_jumps > 0 {
        if !character.is_on_floor() {
            character.current_air_jumps -= 1;
        }
        character.just_jumped = true;
    }
}

// println!("Move:");
// println!("   distance: {}", axis_pair.length());
// println!("          x: {}", axis_pair.x());
// println!("          y: {}", axis_pair.y());
