use avian2d::prelude::Position;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::event::VfxContext;
use crate::util::pos2_to_vec3;

#[derive(Resource)]
pub struct ProjectileTrailEffect(pub Handle<EffectAsset>);

#[derive(Component)]
pub struct ProjectileTrailEmitter {
    pub tracked_entity: Entity,
}

pub fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.05_f32).expr(),
        dimension: ShapeDimension::Volume,
    };

    let lifetime = writer.lit(0.2_f32).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let age = writer.lit(0_f32).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(0.5, 2.0, 4.0, 1.0));
    color.add_key(1.0, Vec4::new(0.1, 0.5, 1.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.06));
    size.add_key(1.0, Vec3::splat(0.0));

    let effect = EffectAsset::new(512, SpawnerSettings::rate(60.0.into()), writer.finish())
        .with_name("projectile_trail")
        .init(init_pos)
        .init(init_lifetime)
        .init(init_age)
        .render(ColorOverLifetimeModifier {
            gradient: color,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size,
            screen_space_size: false,
        });

    commands.insert_resource(ProjectileTrailEffect(effects.add(effect)));
}

pub fn spawn_emitter(
    ctx: &VfxContext,
    effect: &ProjectileTrailEffect,
    commands: &mut Commands,
) {
    commands.spawn((
        ParticleEffect::new(effect.0.clone()),
        Transform::from_translation(pos2_to_vec3(ctx.position.x, ctx.position.y, 0.05)),
        ProjectileTrailEmitter {
            tracked_entity: ctx.entity,
        },
    ));
}

pub fn tick_trail_emitters(
    mut emitters: Query<(Entity, &ProjectileTrailEmitter, &mut Transform)>,
    tracked: Query<&Position>,
    mut commands: Commands,
) {
    for (emitter_entity, emitter, mut transform) in &mut emitters {
        match tracked.get(emitter.tracked_entity) {
            Ok(pos) => {
                transform.translation = pos2_to_vec3(pos.x, pos.y, 0.05);
            }
            Err(_) => {
                commands.entity(emitter_entity).despawn();
            }
        }
    }
}
