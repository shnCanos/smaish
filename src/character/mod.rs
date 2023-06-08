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
            .add_system(character_attack)
            .add_system(attack_system);
    }
}

#[derive(Bundle, Default, Clone)]
pub struct CharacterBundle {
    pub typ: Character,
    pub movement: CharacterMovement,
    pub vel: Velocity,
    pub grav: GravityScale,
    pub damping: Damping,
    pub kincharcont: KinematicCharacterController,
    pub attacks: CharacterAttackController,
}

#[derive(Component, Debug, Clone, Default)]
pub struct Character {
    pub percentage: f32,
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
    pub can_walljump: bool,

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

    /// Helper to track the last walljump direction
    walljump_direction: f32,
}

#[derive(Component, Debug, Clone, Default)]
pub struct CharacterAttack {
    damage: f32,
    has_attacked: Vec<Entity>,
}

#[derive(Component, Debug, Clone, Default)]
pub struct CharacterAttackController {
    attack_timer: Option<Timer>,
    wants_to_forward_air: bool,
    velocity_from_knockback: Vec2,
}

impl CharacterMovement {
    fn is_on_stage(&self) -> bool {
        self.stage_touch_force.y > 0.
    }

    /// The difference between the function and
    /// `!is_on_floor()` is that this one will return
    /// `false` if the character is on a wall
    fn is_not_touching_stage(&self) -> bool {
        !self.is_on_stage() && !self.is_on_stage_wall()
    }

    fn is_on_stage_wall(&self) -> bool {
        self.stage_touch_force.x != 0.
            // For slopes
            && self.stage_touch_force.y == 0.
    }

    fn fastfall(&mut self) {
        self.wants_to_fastfall = true;
    }

    fn jump(&mut self) {
        self.wants_to_jump = true;
    }
}

impl CharacterAttackController {
    pub fn forward_air(&mut self) {
        self.wants_to_forward_air = true;
    }
    fn is_attacking(&self) -> bool {
        self.attack_timer.is_some()
    }
}

impl Default for CharacterMovement {
    fn default() -> Self {
        return Self {
            speed_air: 20.,
            speed_floor: 500.,
            max_speed_air: 650.,
            fastfall_initial_speed: 0.,
            normal_gravity: 20.,
            fastfalling_gravity: 175.,
            jump_boost: 1000.,
            max_air_jumps: 1,
            can_walljump: true,
            x: default(),
            wants_to_jump: default(),
            is_fastfalling: default(),
            current_air_jumps: default(),
            was_fastfalling_last_frame: default(),
            wants_to_fastfall: default(),
            stage_touch_force: default(),
            walljump_direction: default(),
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
        /*
        Not very pretty or preformant, but bug free👍🏻! (Probably)
        It should be better in terms of performance if I use
        collision events too, but that would mean I need to think
        and like, use my brain????? so weird. Not gonna do it lol
        */

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
fn character_movement(
    mut character_query: Query<(
        &mut CharacterAttackController,
        &mut CharacterMovement,
        &mut Velocity,
        &mut GravityScale,
    )>,
) {
    for (mut attack_controller, mut movement, mut vel, mut gravity) in character_query.iter_mut() {
        // Horizontal Movement
        if movement.is_on_stage() {
            vel.linvel.x = movement.x * movement.speed_floor;
        } else {
            // Using the same thing as in 2 lines above makes the movement feel very awkward
            vel.linvel.x = (vel.linvel.x + movement.x * movement.speed_air)
                .clamp(-movement.max_speed_air, movement.max_speed_air);
        }

        // Knockback
        vel.linvel += attack_controller.velocity_from_knockback;
        attack_controller.velocity_from_knockback = Vec2::ZERO;

        // FastFall
        let just_started_fastfalling = !movement.was_fastfalling_last_frame
            && movement.wants_to_fastfall
            && vel.linvel.y < 0.
            && !movement.is_on_stage();

        // Apply fastfall
        if just_started_fastfalling {
            vel.linvel.y -= movement.fastfall_initial_speed;
            gravity.0 = movement.fastfalling_gravity;
            movement.is_fastfalling = true;
        }

        // Remove fasfall
        if movement.is_on_stage() {
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
        let gonna_inevitably_walljump = movement.stage_touch_force.x * movement.x > 0.
            && movement.walljump_direction * movement.x <= 0.
            && movement.can_walljump;

        // This is done in order to track direction of the wall that was jumped in order
        // to replicate the behavior in smash
        // (walls with the same direction cannot be jumped twice in a row)
        if gonna_inevitably_walljump {
            movement.walljump_direction = movement.x;
        }

        if (movement.wants_to_jump && movement.current_air_jumps > 0) || gonna_inevitably_walljump {
            if movement.is_not_touching_stage() {
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

        // reset the variables
        movement.wants_to_jump = false;
        if movement.is_on_stage() {
            movement.current_air_jumps = movement.max_air_jumps;
            movement.walljump_direction = 0.;
        }
    }
}

fn attack_system(
    mut attack: Query<&mut CharacterAttack>,
    mut attacked: Query<(Entity, &mut Character, &mut CharacterAttackController)>,
    mut collision_event: EventReader<CollisionEvent>,
) {
    for collision in collision_event.iter() {
        if let CollisionEvent::Started(col1, col2, _) = collision {
            let (attacked_entity, mut attacked_character, mut attacked_controller) =
                match attacked.get_mut(*col2) {
                    Ok(uwu) => uwu,
                    Err(_) => {
                        continue;
                    }
                };
            let mut attack = attack.get_mut(*col1).unwrap();

            if attack.has_attacked.contains(&attacked_entity) {
                continue;
            }
            dbg!(collision);

            // Change code if stupid
            attacked_character.percentage += attack.damage;

            attacked_controller.velocity_from_knockback += Vec2::new(0., 1000.);

            attack.has_attacked.push(attacked_entity);
        }
    }
}

fn character_attack(
    mut character: Query<(Entity, &mut CharacterAttackController)>,
    children: Query<(Entity, &Parent), With<CharacterAttack>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut character) in character.iter_mut() {
        let mut just_finished_attack = false;

        if let Some(timer) = &mut character.attack_timer {
            just_finished_attack = timer.finished();
            timer.tick(time.delta());
        }

        if just_finished_attack {
            character.attack_timer = None;

            for (child, parent) in children.iter() {
                if entity == parent.get() {
                    commands.entity(child).despawn_recursive();
                    break;
                }
            }
            break;
        }

        if character.is_attacking() {
            character.wants_to_forward_air = false;
            continue;
        }

        if character.wants_to_forward_air {
            character.attack_timer = Some(Timer::from_seconds(1., TimerMode::Once));

            let attack_entity = commands
                .spawn((
                    Collider::cuboid(100., 100.),
                    Sensor,
                    CharacterAttack {
                        damage: 20.,
                        ..default()
                    },
                    ActiveEvents::COLLISION_EVENTS,
                    CollisionGroups::new(
                        Group::from_bits(0b100).unwrap(),
                        Group::from_bits(0b10).unwrap(),
                    ),
                ))
                .id();

            commands.entity(entity).add_child(attack_entity);
        }
    }
}
