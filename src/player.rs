use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use leafwing_input_manager::prelude::*;

// Most of these constants will change according to the character.
// They're placeholders
const PLAYER_JUMP: f32 = 10.;
const PLAYER_FAST_FALL: f32 = 5.;
const PLAYER_SPEED: f32 = 10.;

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_player)
            .add_plugin(InputManagerPlugin::<PlayerActions>::default())
            .add_system(jump)
            .add_system(fast_fall)
            .add_system(move_player)
            .insert_resource(PlayerMovementInfo::default());
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

#[derive(Resource, Default)]
struct PlayerMovementInfo {
    is_fastfalling: bool,
    is_on_floor: bool,
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

fn jump(
    jump_query: Query<&ActionState<PlayerActions>, With<Player>>,
    mut player_query: Query<&mut Velocity, With<Player>>,
    mut info: ResMut<PlayerMovementInfo>,
) {
    let action_state = jump_query.single();
    if !action_state.just_pressed(PlayerActions::Jump) {
        return;
    }

    let mut playervl = player_query.single_mut();

    playervl.linvel.y = PLAYER_JUMP;

    info.is_on_floor = false; // TODO
}

fn fast_fall(
    fastfall_query: Query<&ActionState<PlayerActions>, With<Player>>,
    mut player_query: Query<&mut Velocity, With<Player>>,
    mut info: ResMut<PlayerMovementInfo>,
) {
    let action = fastfall_query.single();

    if !action.just_pressed(PlayerActions::FastFall) {
        return;
    }

    let mut playervl = player_query.single_mut();

    playervl.linvel.y = -PLAYER_FAST_FALL;

    info.is_fastfalling = true;
}

fn move_player(
    move_query: Query<&ActionState<PlayerActions>, With<Player>>,
    mut player_query: Query<&mut Velocity, With<Player>>,
) {
    let action_state = move_query.single();

    if !action_state.pressed(PlayerActions::Move) {
        return;
    }

    let axis_pair = action_state.clamped_axis_pair(PlayerActions::Move).unwrap();
    // println!("Move:");
    // println!("   distance: {}", axis_pair.length());
    // println!("          x: {}", axis_pair.x());
    // println!("          y: {}", axis_pair.y());

    let mut playervl = player_query.single_mut();

    playervl.linvel.x = axis_pair.x() * PLAYER_SPEED;
}
