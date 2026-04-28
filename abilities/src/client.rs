use bevy::prelude::*;
use crate::systems::tick_cooldowns;
use crate::types::{AbilityInstance, Active, Casting};
use crate::AbilitySharedPlugin;

pub struct AbilityClientPlugin;

impl Plugin for AbilityClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbilitySharedPlugin);
        app.add_systems(FixedUpdate, tick_cooldowns);
        app.add_observer(on_ability_instance_added);
        app.add_observer(on_casting_added);
        app.add_observer(on_active_added);
    }
}

fn on_ability_instance_added(trigger: On<Add, AbilityInstance>, query: Query<&AbilityInstance>) {
    if let Ok(instance) = query.get(trigger.entity) {
        info!(
            "AbilityInstance added: {:?} caster={:?} slot={} cursor={}",
            trigger.entity,
            instance.caster,
            instance.slot,
            instance.cursor,
        );
    }
}

fn on_casting_added(trigger: On<Add, Casting>, query: Query<&Casting>) {
    if let Ok(waiting) = query.get(trigger.entity) {
        info!("Casting added: {:?} remaining={:.2}s", trigger.entity, waiting.remaining_secs);
    }
}

fn on_active_added(trigger: On<Add, Active>) {
    info!("Active added: {:?}", trigger.entity);
}
