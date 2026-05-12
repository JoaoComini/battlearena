use crate::attributes::{EffectEvent, Energy, Health};
use crate::systems::{
    apply_ability_effects, move_projectiles, process_melee_hit_tasks, tick_casting_tasks,
    tick_cooldowns, tick_projectile_collision,
};
use crate::types::{AbilityCooldowns, AbilityDef, AbilityInstance, AbilityLoadout};
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
        app.add_message::<EffectEvent<Health>>();
        app.add_message::<EffectEvent<Energy>>();
        app.add_systems(
            FixedUpdate,
            (
                tick_cooldowns,
                tick_casting_tasks,
                process_ability_inputs,
                process_melee_hit_tasks,
                move_projectiles,
                tick_projectile_collision,
                apply_ability_effects::<Health>,
                apply_ability_effects::<Energy>,
            )
                .chain(),
        );
    }
}

fn process_ability_inputs(
    players: Query<(
        Entity,
        &ActionState<Inputs>,
        &Position,
        &Rotation,
        &AbilityLoadout,
        &AbilityCooldowns,
    )>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    for (player_entity, action_state, position, rotation, loadout, cooldowns) in &players {
        let Inputs::PlayerInput(PlayerInput { abilities, .. }) = &action_state.0;
        let pressed = [abilities.slot1, abilities.slot2];
        let origin = position.0;
        let facing_rad = rotation.as_radians();

        for (slot_idx, &is_pressed) in pressed.iter().enumerate() {
            if !is_pressed {
                continue;
            }
            if !cooldowns.is_ready(slot_idx) {
                continue;
            }
            let Some(slot) = loadout.slots.get(slot_idx) else {
                continue;
            };
            let Some(def) = assets.get(&slot.handle) else {
                continue;
            };
            commands
                .entity(player_entity)
                .queue(move |mut e: EntityWorldMut| {
                    if let Some(mut cd) = e.get_mut::<AbilityCooldowns>() {
                        if let Some(r) = cd.remaining.get_mut(slot_idx) {
                            *r = f32::MAX;
                        }
                    }
                });

            let inst = AbilityInstance {
                caster: player_entity,
                slot: slot_idx,
                origin,
                facing_rad,
                cooldown_secs: def.cooldown_secs,
            };
            let instance_entity = commands
                .spawn((inst, Replicate::to_clients(NetworkTarget::All)))
                .id();

            def.ability.cast(instance_entity, &inst, &mut commands);
        }
    }
}
