use bevy::prelude::*;
use crate::registry::AbilityRegistry;
use crate::systems::{move_projectiles, spawn_hitbox, tick_cooldowns};
use crate::types::{AbilityCast, AbilityDef, AbilityLoadout};
use crate::AbilitySharedPlugin;

pub struct AbilityClientPlugin;

impl Plugin for AbilityClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbilitySharedPlugin);
        app.add_observer(on_ability_cast);
        app.add_systems(FixedUpdate, (tick_cooldowns, move_projectiles).chain());
    }
}

fn on_ability_cast(
    trigger: On<Replace, AbilityCast>,
    players: Query<(&AbilityCast, &AbilityLoadout)>,
    registry: Res<AbilityRegistry>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    let Ok((cast, loadout)) = players.get(trigger.entity) else { return };
    if cast.cast_id == 0 { return; }

    let Some(slot) = loadout.slots.get(cast.slot) else { return };
    let Some(def) = registry.get(&slot.key, &assets) else { return };

    spawn_hitbox(trigger.entity, cast, def, &mut commands);
}
