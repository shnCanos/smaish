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

#[derive(Debug, Clone, Copy)]
pub enum Wall {
    Left,
    Right,
    Floor,
    Below,
    Unknown, //If this value appears, might as well be an error
}

#[derive(Component, Debug, Clone)]
pub struct Character {
    // Variables
    pub air_time_needed_to_fastfall: f32,
    pub speed_air: f32,
    pub speed_floor: f32,
    pub max_speed_air: f32,
    pub fastfall_initial_speed: f32,
    pub normal_gravity: f32,
    pub fastfalling_gravity: f32,
    pub jump_boost: f32,
    pub max_air_jumps: usize,

    // Helpers
    /// The movement in the x axys the character should do
    movement_x: f32,

    /// Whether the character wants to jump
    /// This variable is not the same as is_jumping
    /// That one should be `!self.is_on_floor()`
    wants_to_jump: bool,

    /// Whether the character is fastfalling
    is_fastfalling: bool,

    /// This stopwatch counts the time from when the character
    /// last jumped (doesn't count if the character is on the floor).
    /// It is used to prevent the character from fastfalling right after jumping
    fastfall_air_timer: Stopwatch,

    /// Whether the character is touching the stage
    is_touching_stage: Option<Wall>,

    /// How many air jumps the player has currently
    current_air_jumps: usize,

    /// A helper to fastfall
    was_fastfalling_last_frame: bool,

    /// A helper to fastfall
    wants_to_fastfall: bool,

    /// final y position  - beginning y position
    y_position_delta: f32,
}

impl Character {
    pub fn is_on_floor(&self) -> bool {
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
    pub fn is_on_air(&self) -> bool {
        self.is_touching_stage.is_none()
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
        }
    }
}

impl Default for Character {
    fn default() -> Self {
        return Self {
            speed_air: 20.,
            speed_floor: 500.,
            max_speed_air: 500.,
            fastfall_initial_speed: 750.,
            normal_gravity: 20.,
            fastfalling_gravity: 100.,
            jump_boost: 1000.,
            air_time_needed_to_fastfall: 0.35,
            max_air_jumps: 1,
            movement_x: Default::default(),
            wants_to_jump: Default::default(),
            is_fastfalling: Default::default(),
            is_touching_stage: Default::default(),
            fastfall_air_timer: Default::default(),
            current_air_jumps: Default::default(),
            was_fastfalling_last_frame: default(),
            wants_to_fastfall: default(),
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
/// ~~TODO: Change this terrible system, what the hell was I thinking when I wrote this~~
fn character_movement(
    mut character_query: Query<(&mut Character, &mut Velocity, &mut GravityScale)>,
) {
    for (mut character, mut vel, mut gravity) in character_query.iter_mut() {
        // Horizontal Movement
        if character.is_on_floor() {
            vel.linvel.x = character.movement_x * character.speed_floor;
        } else {
            // Using the same thing as in 2 lines above makes the movement feel very awkward
            vel.linvel.x = (vel.linvel.x + character.movement_x * character.speed_air)
                .clamp(-character.max_speed_air, character.max_speed_air);
        }

        // FastFall
        let mut should_fastfall = false;
        if !character.was_fastfalling_last_frame
            && character.wants_to_fastfall
            && character.fastfall_air_timer.elapsed_secs() >= character.air_time_needed_to_fastfall
        {
            should_fastfall = true;
        }

        // Apply fastfall
        if should_fastfall {
            if vel.linvel.y < 0. {
                vel.linvel.y -= character.fastfall_initial_speed;
            } else {
                vel.linvel.y = -character.fastfall_initial_speed;
            }
            gravity.0 = character.fastfalling_gravity;
            character.is_fastfalling = true;
        }

        // Remove fasfall
        let just_stopped_fasfalling =
            character.was_fastfalling_last_frame && !character.is_fastfalling;

        if just_stopped_fasfalling {
            gravity.0 = character.normal_gravity;
            character.fastfall_air_timer.reset();
            character.is_fastfalling = false;
        }

        // Reset the variable
        character.wants_to_fastfall = false;
        character.was_fastfalling_last_frame = character.is_fastfalling;

        // Jump
        if character.wants_to_jump && character.current_air_jumps > 0 {
            if !character.is_on_floor() {
                character.current_air_jumps -= 1;
            }
            vel.linvel.y = character.jump_boost;
            character.wants_to_jump = false;
            character.is_fastfalling = false;
            character.fastfall_air_timer.reset();
        }
    }
}

fn character_information_update(mut character_query: Query<&mut Character>, time: Res<Time>) {
    for mut character in character_query.iter_mut() {
        if !character.is_on_floor() {
            character.fastfall_air_timer.tick(time.delta());
        }

        if character.is_on_floor() {
            character.current_air_jumps = character.max_air_jumps;
        }
    }
}
