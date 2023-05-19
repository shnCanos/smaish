use bevy::prelude::*;
use bevy_rapier2d::prelude::*;

pub struct StagePlugin;

impl Plugin for StagePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_stage);
    }
}

#[derive(Component)]
pub struct Stage;

fn setup_stage(mut commands: Commands) {
    commands.spawn((
        SpriteBundle {
            transform: Transform {
                translation: Vec2::new(0., -1000.).extend(0.),
                ..default()
            },
            ..default()
        },
        RigidBody::Fixed,
        // GravityScale(0.),
        Velocity::default(),
        Collider::cuboid(500., 500.),
        Stage,
        ActiveEvents::CONTACT_FORCE_EVENTS,
    ));
}
