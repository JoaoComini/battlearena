use crate::systems::{
    advance_instance, apply_damage, despawn_ended, move_projectiles, process_melee_hit_request,
    process_projectile_request, tick_casting, tick_cooldowns, tick_projectile_collision,
};
use crate::types::{AbilityCooldowns, AbilityInstance, AbilityLoadout, Advance};
use crate::AbilitySharedPlugin;
use avian2d::prelude::{Position, Rotation};
use bevy::prelude::*;
use inputs::{Inputs, PlayerInput};
use lightyear::prelude::input::native::ActionState;
use lightyear::prelude::*;

pub struct AbilityServerPlugin;

impl Plugin for AbilityServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbilitySharedPlugin);
        app.add_systems(
            FixedUpdate,
            (
                despawn_ended,
                tick_cooldowns,
                tick_casting,
                process_ability_inputs,
                process_melee_hit_request,
                process_projectile_request,
                move_projectiles,
                tick_projectile_collision,
                apply_damage,
            )
                .chain(),
        );
        app.add_observer(advance_instance);
    }
}

fn process_ability_inputs(
    mut players: Query<(
        Entity,
        &ActionState<Inputs>,
        &Position,
        &Rotation,
        &AbilityLoadout,
        &mut AbilityCooldowns,
    )>,
    mut commands: Commands,
) {
    for (player_entity, action_state, position, rotation, loadout, mut cooldowns) in &mut players {
        let Inputs::PlayerInput(PlayerInput { abilities, .. }) = &action_state.0;

        let pressed = [abilities.slot1, abilities.slot2];
        let origin = position.0;
        let facing_rad = rotation.as_radians();

        for (slot_idx, &is_pressed) in pressed.iter().enumerate() {
            if !is_pressed {
                continue;
            }
            if loadout.slots.get(slot_idx).is_none() {
                continue;
            }
            if !cooldowns.is_ready(slot_idx) {
                continue;
            }

            if let Some(remaining) = cooldowns.remaining.get_mut(slot_idx) {
                *remaining = f32::MAX;
            }

            commands.spawn((
                AbilityInstance {
                    caster: player_entity,
                    slot: slot_idx,
                    origin,
                    facing_rad,
                    cursor: 0,
                },
                Advance,
                Replicate::to_clients(NetworkTarget::All),
            ));
        }
    }
}
