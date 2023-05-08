use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_stage);
    }
}

#[derive(Component)]
struct Stage;

fn setup_stage(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1000.,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(3., 2., 3.),
        ..default()
    });

    commands.spawn((
        PbrBundle {
            mesh: meshes.add(shape::Plane::from_size(200.).into()),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            ..Default::default()
        },
        RigidBody::Dynamic, // For moving platforms
        GravityScale(0.),
        Velocity::default(),
        Collider::cuboid(100., 0.05, 100.),
        Stage,
    ));
}
