use abilities::types::{AbilityInstance, Casting};
use avian2d::prelude::Position;
use bevy::prelude::*;
use protocol::VfxTag;

use crate::effects::{ability_burst, cast_ring, projectile_trail};
use crate::event::VfxContext;

pub fn on_vfx_tag(
    trigger: On<Add, VfxTag>,
    tags: Query<&VfxTag>,
    instances: Query<&AbilityInstance>,
    positions: Query<&Position>,
    castings: Query<&Casting>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(tag) = tags.get(trigger.entity) else { return };

    let position = instances
        .get(trigger.entity)
        .map(|i| i.origin)
        .or_else(|_| positions.get(trigger.entity).map(|p| p.0))
        .unwrap_or(Vec2::ZERO);

    let cast_duration = castings.get(trigger.entity).ok().map(|c| c.remaining_secs);

    let ctx = VfxContext {
        position,
        entity: trigger.entity,
        cast_duration,
    };

    match tag.0.as_str() {
        "cast_ring"           => cast_ring::spawn(&ctx, &mut commands, &mut meshes, &mut materials),
        "activate_melee"      => ability_burst::spawn_melee(&ctx, &mut commands, &mut meshes, &mut materials),
        "activate_projectile" => ability_burst::spawn_projectile(&ctx, &mut commands, &mut meshes, &mut materials),
        "projectile_trail"    => projectile_trail::spawn_emitter(&ctx, &mut commands),
        _                     => {}
    }
}
