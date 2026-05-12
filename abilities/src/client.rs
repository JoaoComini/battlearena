use crate::systems::tick_cooldowns;
use crate::types::{AbilityInstance, Active};
use crate::AbilitySharedPlugin;
use bevy::prelude::*;

pub struct AbilityClientPlugin;

impl Plugin for AbilityClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(AbilitySharedPlugin);
        app.add_systems(FixedUpdate, tick_cooldowns);
        app.add_observer(on_ability_instance_added);
        app.add_observer(on_active_added);
    }
}

fn on_ability_instance_added(trigger: On<Add, AbilityInstance>, query: Query<&AbilityInstance>) {
    if let Ok(instance) = query.get(trigger.entity) {
        info!(
            "AbilityInstance added: {:?} caster={:?} slot={}",
            trigger.entity, instance.caster, instance.slot,
        );
    }
}


fn on_active_added(trigger: On<Add, Active>) {
    info!("Active added: {:?}", trigger.entity);
}
