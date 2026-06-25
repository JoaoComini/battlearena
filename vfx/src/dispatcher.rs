use abilities::types::{AbilityInstance, Casting};
use avian2d::prelude::Position;
use bevy::prelude::*;
use protocol::VfxTag;

use crate::effects::{
    ability_burst::{spawn_melee, spawn_projectile, MeleeBurstEffect, ProjectileBurstEffect},
    cast_ring::{spawn as spawn_cast_ring, CastRingEffect},
    projectile_trail::{spawn_emitter, ProjectileTrailEffect},
};
use crate::event::VfxContext;

pub fn on_vfx_tag(
    trigger: On<Add, VfxTag>,
    tags: Query<&VfxTag>,
    instances: Query<&AbilityInstance>,
    positions: Query<&Position>,
    castings: Query<&Casting>,
    cast_ring: Res<CastRingEffect>,
    melee: Res<MeleeBurstEffect>,
    projectile_burst: Res<ProjectileBurstEffect>,
    trail: Res<ProjectileTrailEffect>,
    mut commands: Commands,
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
        "cast_ring"           => spawn_cast_ring(&ctx, &cast_ring, &mut commands),
        "activate_melee"      => spawn_melee(&ctx, &melee, &mut commands),
        "activate_projectile" => spawn_projectile(&ctx, &projectile_burst, &mut commands),
        "projectile_trail"    => spawn_emitter(&ctx, &trail, &mut commands),
        _                     => {}
    }
}
