use bevy::prelude::*;

const ZOOMING_IN_CAMERA_LERP_SPEED: f32 = 0.01;
const ZOOMING_OUT_CAMERA_LERP_SPEED: f32 = 0.8;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_camera)
            .add_system(camera_follows);
    }
}

#[derive(Component)]
struct MainGameCamera;

#[derive(Component, Default)]
pub struct CameraFollows {
    /// The camera will make sure the center of the body is visible + padding pixels.
    /// Since the window isn't a square (I hope),
    /// it only adds `padding` pixels to the smallest side and adds whatever
    /// necessary to maintain ration in the other
    pub padding: usize,
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera2dBundle {
            ..Default::default()
        },
        MainGameCamera,
    ));
}

fn camera_follows(
    mut camera_query: Query<(&mut Transform, &mut OrthographicProjection), With<MainGameCamera>>,
    character_query: Query<(&Transform, &CameraFollows), Without<MainGameCamera>>,
    window: Query<&Window>,
) {
    let (mut camera_tf, mut camera_projection) = camera_query.single_mut();
    let window = window.single();

    let mut character_translation_sum = Vec3::ZERO;

    for (character_tf, _) in character_query.iter() {
        character_translation_sum += character_tf.translation;
    }

    let character_translation_average =
        character_translation_sum / character_query.iter().count() as f32;

    // .0 = max distance
    // .1 = padding
    let max_distance_from_camera = character_query
        .iter()
        .map(|(tf, padding)| {
            (
                tf.translation - character_translation_average,
                padding.padding as f32,
            )
        })
        .max_by(|(a, padding_a), (b, padding_b)| {
            (a.abs().max_element() + padding_a)
                .partial_cmp(&(b.abs().max_element() + padding_b))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    camera_tf.translation = character_translation_average;

    let window_height = window.height();
    let window_width = window.width();
    let new_camera_scale = (max_distance_from_camera.0.abs().x * 2. / window_width)
        .max(max_distance_from_camera.0.abs().y * 2. / window_height)
        + max_distance_from_camera.1 / window_height.min(window_width);

    camera_projection.scale = camera_projection.scale
        + (new_camera_scale - camera_projection.scale)
            * if new_camera_scale - camera_projection.scale < 0. {
                ZOOMING_IN_CAMERA_LERP_SPEED
            } else {
                ZOOMING_OUT_CAMERA_LERP_SPEED
            };
}
