use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::event::VfxContext;
use crate::util::pos2_to_vec3;

#[derive(Resource)]
pub struct MeleeBurstEffect(pub Handle<EffectAsset>);

#[derive(Resource)]
pub struct ProjectileBurstEffect(pub Handle<EffectAsset>);

pub fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    // ── Melee burst ───────────────────────────────────────────────────────────
    {
        let writer = ExprWriter::new();

        let init_pos = SetPositionCircleModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            axis: writer.lit(Vec3::Y).expr(),
            radius: writer.lit(0.6_f32).expr(),
            dimension: ShapeDimension::Surface,
        };

        let init_vel = SetVelocityCircleModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            axis: writer.lit(Vec3::Y).expr(),
            speed: writer.lit(3.0_f32).expr(),
        };

        let lifetime = writer.lit(0.2_f32).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
        let age = writer.lit(0_f32).expr();
        let init_age = SetAttributeModifier::new(Attribute::AGE, age);

        let mut color = bevy_hanabi::Gradient::new();
        color.add_key(0.0, Vec4::new(4.0, 1.5, 0.1, 1.0));
        color.add_key(0.5, Vec4::new(2.0, 0.5, 0.1, 0.8));
        color.add_key(1.0, Vec4::new(1.0, 0.2, 0.0, 0.0));

        let mut size = bevy_hanabi::Gradient::new();
        size.add_key(0.0, Vec3::splat(0.07));
        size.add_key(1.0, Vec3::splat(0.01));

        let effect = EffectAsset::new(256, SpawnerSettings::once(60.0.into()), writer.finish())
            .with_name("melee_burst")
            .init(init_pos)
            .init(init_vel)
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

        commands.insert_resource(MeleeBurstEffect(effects.add(effect)));
    }

    // ── Projectile burst ──────────────────────────────────────────────────────
    {
        let writer = ExprWriter::new();

        let init_pos = SetPositionCircleModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            axis: writer.lit(Vec3::Y).expr(),
            radius: writer.lit(0.25_f32).expr(),
            dimension: ShapeDimension::Surface,
        };

        let init_vel = SetVelocityCircleModifier {
            center: writer.lit(Vec3::ZERO).expr(),
            axis: writer.lit(Vec3::Y).expr(),
            speed: writer.lit(2.0_f32).expr(),
        };

        let lifetime = writer.lit(0.15_f32).expr();
        let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
        let age = writer.lit(0_f32).expr();
        let init_age = SetAttributeModifier::new(Attribute::AGE, age);

        let mut color = bevy_hanabi::Gradient::new();
        color.add_key(0.0, Vec4::new(0.5, 2.0, 4.0, 1.0));
        color.add_key(1.0, Vec4::new(0.2, 0.8, 2.0, 0.0));

        let mut size = bevy_hanabi::Gradient::new();
        size.add_key(0.0, Vec3::splat(0.05));
        size.add_key(1.0, Vec3::splat(0.01));

        let effect = EffectAsset::new(128, SpawnerSettings::once(40.0.into()), writer.finish())
            .with_name("projectile_burst")
            .init(init_pos)
            .init(init_vel)
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

        commands.insert_resource(ProjectileBurstEffect(effects.add(effect)));
    }
}

pub fn spawn_melee(ctx: &VfxContext, effect: &MeleeBurstEffect, commands: &mut Commands) {
    commands.spawn((
        ParticleEffect::new(effect.0.clone()),
        Transform::from_translation(pos2_to_vec3(ctx.position.x, ctx.position.y, 0.03)),
    ));
}

pub fn spawn_projectile(
    ctx: &VfxContext,
    effect: &ProjectileBurstEffect,
    commands: &mut Commands,
) {
    commands.spawn((
        ParticleEffect::new(effect.0.clone()),
        Transform::from_translation(pos2_to_vec3(ctx.position.x, ctx.position.y, 0.03)),
    ));
}
