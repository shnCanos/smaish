mod player;

use bevy::{prelude::*, time::Stopwatch, utils::HashMap};
use bevy_rapier3d::prelude::*;

use crate::stage::Stage;

pub struct CharacterPlugin;

impl Plugin for CharacterPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(player::PlayerPlugin)
            .add_system(character_touching_stage_check)
            .add_system(character_movement)
            .add_system(character_information_update);
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Wall {
    Left,
    Right,
    Floor,
    Below,
    Unknown, //If this value appears, might as well be an error
}

#[derive(Component, Debug)]
pub struct Character {
    movement_x: f32,
    just_jumped: bool,
    is_fastfalling: bool,
    fastfall_air_time: Stopwatch,
    is_touching_stage: Option<Wall>,
    current_air_jumps: usize,

    // TODO: The following variables are Placeholders!
    speed_air: f32,
    speed_floor: f32,
    max_speed_air: f32,
    fastfall_initial_speed: f32,
    normal_gravity: f32,
    fastfalling_gravity: f32,
    jump_boost: f32,
    air_time_needed_to_fastfall: f32,
    max_air_jumps: usize,
}

impl Character {
    fn is_on_floor(&self) -> bool {
        match &self.is_touching_stage {
            Some(wall) => matches!(wall, Wall::Floor),
            None => false,
        }
    }

    /// Returns false if the player is not touching ANYTHING
    ///
    /// This function also takes into account if, for instance,
    /// the player is bumping its head against the stage, therefore
    /// !is_on_floor is recommended
    fn is_on_air(&self) -> bool {
        self.is_touching_stage.is_none()
    }
}

impl Default for Character {
    fn default() -> Self {
        return Self {
            speed_air: 2.,
            speed_floor: 10.,
            max_speed_air: 10.,
            fastfall_initial_speed: 10.,
            normal_gravity: 2.,
            fastfalling_gravity: 4.,
            jump_boost: 10.,
            air_time_needed_to_fastfall: 0.5,
            max_air_jumps: 1,
            movement_x: Default::default(),
            just_jumped: Default::default(),
            is_fastfalling: Default::default(),
            is_touching_stage: Default::default(),
            fastfall_air_time: Default::default(),
            current_air_jumps: Default::default(),
        };
    }
}

fn character_touching_stage_check(
    mut contact_force_events: EventReader<ContactForceEvent>,
    stage_query: Query<&Stage>,
    mut character_query: Query<&mut Character>,
) {
    character_query.for_each_mut(|mut character| {
        character.is_touching_stage = None; // Reset the value
    });

    'contact_loop: for contact_force_event in contact_force_events.iter() {
        // Not very pretty or preformant, but bug free👍🏻! (Probably)
        // It should be better in terms of performance if I use
        // collision events too, but that would mean I need to think
        // and like, use my brain????? so weird. Not gonna do it lol

        let character_check = [
            [
                character_query.get(contact_force_event.collider1).is_ok(),
                character_query.get(contact_force_event.collider2).is_ok(),
            ],
            [
                stage_query.get(contact_force_event.collider1).is_ok(),
                stage_query.get(contact_force_event.collider2).is_ok(),
            ],
        ];

        for check in character_check {
            if check.iter().filter(|a| **a).count() == 0 {
                continue 'contact_loop;
            }
        }

        let mut character = if character_query.get(contact_force_event.collider1).is_ok() {
            character_query.get_mut(contact_force_event.collider1)
        } else {
            character_query.get_mut(contact_force_event.collider2)
        }
        .unwrap();

        // Yandere dev time! Why use anything else, when you can use ELSE IF
        character.is_touching_stage = Some(if contact_force_event.total_force.y < 0. {
            Wall::Floor
        } else if contact_force_event.total_force.y > 0. {
            Wall::Floor
        } else if contact_force_event.total_force.y < 0. {
            Wall::Below
        } else if contact_force_event.total_force.x > 0. {
            Wall::Right
        } else if contact_force_event.total_force.x < 0. {
            Wall::Left
        } else {
            Wall::Unknown
        });

        if let Wall::Floor = character.is_touching_stage.unwrap() {
            character.is_fastfalling = false;
        }
    }
}

/// Applies the movement to the character.
///
/// NOTE: This function does not check whether the movement
/// should be applied, it simply applies. There are some exceptions though
/// specifically with fast falling.
///
/// Just do
/// ```rust
/// character.fastfalling = true;
/// ```
/// and you should be good to go
fn character_movement(
    mut was_fastfalling_last_frame_hashmap: Local<HashMap<Entity, bool>>,
    mut character_query: Query<(Entity, &mut Character, &mut Velocity, &mut GravityScale)>,
) {
    for (entity_character, mut character, mut vel, mut gravity) in character_query.iter_mut() {
        // Horizontal Movement
        if character.is_on_floor() {
            vel.linvel.x = character.movement_x * character.speed_floor;
        } else {
            // Using the same thing as in 2 lines above makes the movement feel very awkward
            vel.linvel.x = (vel.linvel.x + character.movement_x * character.speed_air)
                .clamp(-character.max_speed_air, character.max_speed_air);
        }

        // FastFall
        let was_fastfalling_last_frame =
            match was_fastfalling_last_frame_hashmap.get(&entity_character) {
                Some(was_fastfalling_last_frame) => was_fastfalling_last_frame,
                None => {
                    was_fastfalling_last_frame_hashmap
                        .insert(entity_character, character.is_fastfalling);
                    &false
                }
            };

        if !was_fastfalling_last_frame && character.is_fastfalling {
            if character.fastfall_air_time.elapsed_secs() <= character.air_time_needed_to_fastfall {
                character.is_fastfalling = false;
            } else {
                vel.linvel.y = -character.fastfall_initial_speed;
                gravity.0 = character.fastfalling_gravity;
            }
        }

        // ~~Encompasses jump~~
        if *was_fastfalling_last_frame && !character.is_fastfalling {
            gravity.0 = character.normal_gravity;
            character.fastfall_air_time.reset();
        }

        was_fastfalling_last_frame_hashmap.insert(entity_character, character.is_fastfalling);

        // Jump
        if character.just_jumped {
            vel.linvel.y = character.jump_boost;
            character.just_jumped = false;
            character.is_fastfalling = false;
        }
    }
}

fn character_information_update(mut character_query: Query<&mut Character>, time: Res<Time>) {
    for mut character in character_query.iter_mut() {
        character.fastfall_air_time.tick(time.delta());

        if character.is_on_floor() {
            character.current_air_jumps = character.max_air_jumps;
        }
    }
}
