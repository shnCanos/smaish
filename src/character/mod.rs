mod player;

use bevy::prelude::*;
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

#[derive(Bundle, Default, Clone)]
pub struct CharacterBundle {
    name: Character,
    pub movement: CharacterMovement,
    pub vel: Velocity,
    pub grav: GravityScale,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Character {
    // Controls the movement of the character
    // movement: CharacterMovement,
}

#[derive(Component, Debug, Clone)]
pub struct CharacterMovement {
    // Constants
    pub speed_air: f32,
    pub speed_floor: f32,
    pub max_speed_air: f32,
    pub fastfall_initial_speed: f32,
    pub normal_gravity: f32,
    pub fastfalling_gravity: f32,
    pub jump_boost: f32,
    pub max_air_jumps: usize,

    /// The movement in the x axys the character should do
    /// Must be between -1 and 1
    x: f32,

    /// Whether the character wants to jump
    /// This variable is not the same as is_jumping
    /// That one should be `!self.is_on_floor()`
    wants_to_jump: bool,

    /// Whether the character wants to fastfall
    wants_to_fastfall: bool,

    /// Whether the character is fastfalling
    is_fastfalling: bool,

    /// The force the character is exerting in the stage
    /// it is equal to (0., 0.) if the character is not touching it
    stage_touch_force: Vec2,

    /// How many air jumps the player has currently
    current_air_jumps: usize,

    /// A helper to fastfall
    was_fastfalling_last_frame: bool,
}

impl CharacterMovement {
    fn is_on_floor(&self) -> bool {
        self.stage_touch_force.y > 0.
    }

    /// The difference between the function and
    /// `!is_on_floor()` is that this one will return
    /// `false` if the character is on a wall
    fn is_on_air(&self) -> bool {
        !self.is_on_floor() && !self.is_on_wall()
    }

    fn is_on_wall(&self) -> bool {
        self.stage_touch_force.x != 0.
    }

    fn fastfall(&mut self) {
        self.wants_to_fastfall = true;
    }

    fn jump(&mut self) {
        self.wants_to_jump = true;
    }
}

impl Default for CharacterMovement {
    fn default() -> Self {
        return Self {
            speed_air: 20.,
            speed_floor: 500.,
            max_speed_air: 500.,
            fastfall_initial_speed: 0.,
            normal_gravity: 20.,
            fastfalling_gravity: 175.,
            jump_boost: 1000.,
            max_air_jumps: 1,
            x: default(),
            wants_to_jump: default(),
            is_fastfalling: default(),
            current_air_jumps: default(),
            was_fastfalling_last_frame: default(),
            wants_to_fastfall: default(),
            stage_touch_force: default(),
        };
    }
}

fn character_touching_stage_check(
    mut contact_force_events: EventReader<ContactForceEvent>,
    stage_query: Query<&Stage>,
    mut character_query: Query<&mut CharacterMovement>,
) {
    character_query.for_each_mut(|mut character| {
        character.stage_touch_force *= 0.; // Reset the variable
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

        character.stage_touch_force = contact_force_event.total_force;
    }
}

/// Applies the movement to the character.
/// TODO Add walljump limit to wall on the same side
fn character_movement(
    mut character_query: Query<(&mut CharacterMovement, &mut Velocity, &mut GravityScale)>,
) {
    for (mut movement, mut vel, mut gravity) in character_query.iter_mut() {
        // Horizontal Movement
        if movement.is_on_floor() {
            vel.linvel.x = movement.x * movement.speed_floor;
        } else {
            // Using the same thing as in 2 lines above makes the feel very awkward
            vel.linvel.x = (vel.linvel.x + movement.x * movement.speed_air)
                .clamp(-movement.max_speed_air, movement.max_speed_air);
        }

        // FastFall
        let just_started_fastfalling = !movement.was_fastfalling_last_frame
            && movement.wants_to_fastfall
            && vel.linvel.y < 0.
            && !movement.is_on_floor();

        // Apply fastfall
        if just_started_fastfalling {
            vel.linvel.y -= movement.fastfall_initial_speed;
            gravity.0 = movement.fastfalling_gravity;
            movement.is_fastfalling = true;
        }

        // Remove fasfall
        if movement.is_on_floor() {
            movement.is_fastfalling = false;
        }

        let just_stopped_fasfalling =
            movement.was_fastfalling_last_frame && !movement.is_fastfalling;

        if just_stopped_fasfalling {
            gravity.0 = movement.normal_gravity;
            movement.is_fastfalling = false;
        }

        // Reset the variables
        movement.wants_to_fastfall = false;
        movement.was_fastfalling_last_frame = movement.is_fastfalling;

        // Jump
        let gonna_inevitably_walljump =
            movement.stage_touch_force.x * movement.x > 0. && movement.is_on_wall();

        if movement.wants_to_jump && movement.current_air_jumps > 0 || gonna_inevitably_walljump {
            if movement.is_on_air() {
                movement.current_air_jumps -= 1;
            }
            vel.linvel.y = movement.jump_boost;
            movement.is_fastfalling = false;

            // In smash, when you jump, for some reason
            // you temporarily get a speed boost or
            // something. It seems like the game
            // thinks you are on floor, therefore
            // I am gonna use the same as
            // the one I use when the character is on floor
            vel.linvel.x = movement.x * movement.speed_floor;
        }

        // reset the variable
        movement.wants_to_jump = false;
    }
}

fn character_information_update(mut character_query: Query<&mut CharacterMovement>) {
    for mut character in character_query.iter_mut() {
        if character.is_on_floor() {
            character.current_air_jumps = character.max_air_jumps;
        }
    }
}
