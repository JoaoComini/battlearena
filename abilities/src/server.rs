use avian2d::prelude::{Position, Rotation};
use bevy::prelude::*;
use inputs::{Inputs, PlayerInput};
use lightyear::prelude::input::native::ActionState;
use crate::registry::AbilityRegistry;
use crate::systems::{
    apply_hitbox_damage, apply_projectile_damage, move_projectiles, spawn_hitbox, tick_cooldowns,
};
use crate::types::{AbilityCast, AbilityDef, AbilityLoadout};
use crate::AbilitySharedPlugin;

pub struct AbilityServerPlugin;

impl Plugin for AbilityServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbilitySharedPlugin);
        app.add_systems(
            FixedUpdate,
            (
                tick_cooldowns,
                process_ability_inputs,
                move_projectiles,
                apply_hitbox_damage,
                apply_projectile_damage,
            )
                .chain(),
        );
    }
}

fn process_ability_inputs(
    mut players: Query<(Entity, &ActionState<Inputs>, &Position, &Rotation, &mut AbilityLoadout, &AbilityCast)>,
    registry: Res<AbilityRegistry>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    for (player_entity, action_state, position, rotation, mut loadout, prev_cast) in &mut players {
        let Inputs::PlayerInput(PlayerInput { abilities, .. }) = &action_state.0;

        let pressed = [abilities.slot1, abilities.slot2];
        let origin = position.0;
        let facing_rad = rotation.as_radians();

        for (slot_idx, &is_pressed) in pressed.iter().enumerate() {
            if !is_pressed { continue; }

            let Some(slot) = loadout.slots.get_mut(slot_idx) else { continue };
            if !slot.is_ready() { continue; }

            let Some(def) = registry.get(&slot.key, &assets) else { continue };
            slot.cooldown_remaining = def.cooldown_secs;

            let new_cast = AbilityCast {
                slot: slot_idx,
                origin,
                facing_rad,
                cast_id: prev_cast.cast_id + 1,
            };

            // Spawn server-side authoritative hitbox for damage
            spawn_hitbox(player_entity, &new_cast, def, &mut commands);

            // Replace the component so replication fires On<Replace, AbilityCast> on all clients
            commands.entity(player_entity).insert(new_cast);
        }
    }
}
