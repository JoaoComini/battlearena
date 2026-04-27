use bevy::prelude::*;
use crate::registry::AbilityRegistry;
use crate::systems::{move_projectiles, spawn_hitbox, tick_cooldowns};
use crate::types::{AbilityCast, AbilityDef, AbilityLoadout};
use crate::AbilitySharedPlugin;

pub struct AbilityClientPlugin;

impl Plugin for AbilityClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbilitySharedPlugin);
        app.add_systems(FixedUpdate, (tick_cooldowns, on_ability_cast, move_projectiles).chain());
    }
}

fn on_ability_cast(
    players: Query<(Entity, &AbilityCast, &AbilityLoadout), Changed<AbilityCast>>,
    registry: Res<AbilityRegistry>,
    assets: Res<Assets<AbilityDef>>,
    mut commands: Commands,
) {
    for (entity, cast, loadout) in &players {
        if cast.cast_id == 0 { continue; }

        let Some(slot) = loadout.slots.get(cast.slot) else { continue };
        let Some(def) = registry.get(&slot.key, &assets) else { continue };

        spawn_hitbox(entity, cast, def, &mut commands);
    }
}
