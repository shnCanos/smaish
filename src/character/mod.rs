mod player;

use bevy::{prelude::*, time::Stopwatch};
use bevy_rapier2d::prelude::*;

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

#[derive(Component, Debug, Clone)]
pub struct Character {
    movement: CharacterMovement,
}

#[derive(Component, Debug, Clone, Default)]
struct CharacterMovement {
    // Constants
    pub air_time_needed_to_fastfall: f32,
    pub speed_air: f32,
    pub speed_floor: f32,
    pub max_speed_air: f32,
    pub fastfall_initial_speed: f32,
    pub normal_gravity: f32,
    pub fastfalling_gravity: f32,
    pub jump_boost: f32,
    pub max_air_jumps: usize,

    /// The movement in the x axys the character should do
    x: f32,

    /// Whether the character wants to jump
    /// This variable is not the same as is_jumping
    /// That one should be `!self.is_on_floor()`
    wants_to_jump: bool,

    /// Whether the character wants to fastfall
    wants_to_fastfall: bool,

    /// Whether the character is fastfalling
    is_fastfalling: bool,

    /// This stopwatch counts the time from when the character
    /// last jumped (doesn't count if the character is on the floor).
    /// It is used to prevent the character from fastfalling right after jumping
    fastfall_air_timer: Stopwatch,

    /// The force the character is exerting in the stage
    /// it is equal to (0., 0.) if the character is not touching it
    stage_touch_force: Vec2,

    /// How many air jumps the player has currently
    current_air_jumps: usize,

    /// A helper to fastfall
    was_fastfalling_last_frame: bool,
}

impl Character {
    pub fn is_on_floor(&self) -> bool {
        self.movement.stage_touch_force.y > 0.
    }

    /// Returns false if the player is not touching ANYTHING
    ///
    /// This function also takes into account if, for instance,
    /// the player is bumping its head against the stage, therefore
    /// !is_on_floor is recommended
    pub fn is_on_air(&self) -> bool {
        self.movement.stage_touch_force == Vec2::ZERO
    }

    pub fn is_on_wall(&self) -> bool {
        self.movement.stage_touch_force.x != 0.
    }

    pub fn new(
        air_time_needed_to_fastfall: f32,
        speed_air: f32,
        speed_floor: f32,
        max_speed_air: f32,
        fastfall_initial_speed: f32,
        normal_gravity: f32,
        fastfalling_gravity: f32,
        jump_boost: f32,
        max_air_jumps: usize,
    ) -> Self {
        Self {
            movement: CharacterMovement {
                air_time_needed_to_fastfall,
                speed_air,
                speed_floor,
                max_speed_air,
                fastfall_initial_speed,
                normal_gravity,
                fastfalling_gravity,
                jump_boost,
                max_air_jumps,
                ..default()
            },
            ..default()
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        return Self {
            movement: CharacterMovement {
                speed_air: 20.,
                speed_floor: 500.,
                max_speed_air: 500.,
                fastfall_initial_speed: 750.,
                normal_gravity: 20.,
                fastfalling_gravity: 100.,
                jump_boost: 1000.,
                air_time_needed_to_fastfall: 0.35,
                max_air_jumps: 1,
                x: default(),
                wants_to_jump: default(),
                is_fastfalling: default(),
                fastfall_air_timer: default(),
                current_air_jumps: default(),
                was_fastfalling_last_frame: default(),
                wants_to_fastfall: default(),
                stage_touch_force: default(),
            },
        };
    }
}

fn character_touching_stage_check(
    mut contact_force_events: EventReader<ContactForceEvent>,
    stage_query: Query<&Stage>,
    mut character_query: Query<&mut Character>,
) {
    character_query.for_each_mut(|mut character| {
        character.movement.stage_touch_force *= 0.; // Reset the variable
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
        character.movement.stage_touch_force = contact_force_event.total_force;
    }
}

/// Applies the movement to the character.
/// ~~TODO: Change this terrible system, what the hell was I thinking when I wrote this~~
fn character_movement(
    mut character_query: Query<(&mut Character, &mut Velocity, &mut GravityScale)>,
) {
    for (mut character, mut vel, mut gravity) in character_query.iter_mut() {
        // Horizontal Movement
        if character.is_on_floor() {
            vel.linvel.x = character.movement.x * character.movement.speed_floor;
        } else {
            // Using the same thing as in 2 lines above makes the movement feel very awkward
            vel.linvel.x = (vel.linvel.x + character.movement.x * character.movement.speed_air)
                .clamp(
                    -character.movement.max_speed_air,
                    character.movement.max_speed_air,
                );
        }

        // FastFall
        let mut should_fastfall = false;
        if !character.movement.was_fastfalling_last_frame
            && character.movement.wants_to_fastfall
            && character.movement.fastfall_air_timer.elapsed_secs()
                >= character.movement.air_time_needed_to_fastfall
        {
            should_fastfall = true;
        }

        // Apply fastfall
        if should_fastfall {
            if vel.linvel.y < 0. {
                vel.linvel.y -= character.movement.fastfall_initial_speed;
            } else {
                vel.linvel.y = -character.movement.fastfall_initial_speed;
            }
            gravity.0 = character.movement.fastfalling_gravity;
            character.movement.is_fastfalling = true;
        }

        // Remove fasfall
        let just_stopped_fasfalling =
            character.movement.was_fastfalling_last_frame && !character.movement.is_fastfalling;

        if just_stopped_fasfalling {
            gravity.0 = character.movement.normal_gravity;
            character.movement.fastfall_air_timer.reset();
            character.movement.is_fastfalling = false;
        }

        // Reset the variable
        character.movement.wants_to_fastfall = false;
        character.movement.was_fastfalling_last_frame = character.movement.is_fastfalling;

        // Jump
        if character.movement.wants_to_jump && character.movement.current_air_jumps > 0 {
            if !character.is_on_floor() {
                character.movement.current_air_jumps -= 1;
            }
            vel.linvel.y = character.movement.jump_boost;
            character.movement.wants_to_jump = false;
            character.movement.is_fastfalling = false;
            character.movement.fastfall_air_timer.reset();
        }
    }
}

fn character_information_update(mut character_query: Query<&mut Character>, time: Res<Time>) {
    for mut character in character_query.iter_mut() {
        if !character.is_on_floor() {
            character.movement.fastfall_air_timer.tick(time.delta());
        }

        if character.is_on_floor() {
            character.movement.current_air_jumps = character.movement.max_air_jumps;
        }
    }
}
