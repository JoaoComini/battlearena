use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::event::VfxContext;
use crate::util::pos2_to_vec3;

#[derive(Resource)]
pub struct CastRingEffect(pub Handle<EffectAsset>);

pub fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let writer = ExprWriter::new();

    let init_pos = SetPositionCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        radius: writer.lit(1.2_f32).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityCircleModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed: writer.lit(1.5_f32).expr(),
    };

    let lifetime = writer.lit(0.4_f32).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);

    let age = writer.lit(0_f32).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let mut color_gradient = bevy_hanabi::Gradient::new();
    color_gradient.add_key(0.0, Vec4::new(0.5, 1.0, 2.0, 1.0));
    color_gradient.add_key(1.0, Vec4::new(0.2, 0.5, 1.0, 0.0));

    let mut size_gradient = bevy_hanabi::Gradient::new();
    size_gradient.add_key(0.0, Vec3::splat(0.06));
    size_gradient.add_key(1.0, Vec3::splat(0.01));

    let effect = EffectAsset::new(512, SpawnerSettings::once(80.0.into()), writer.finish())
        .with_name("cast_ring")
        .init(init_pos)
        .init(init_vel)
        .init(init_lifetime)
        .init(init_age)
        .render(ColorOverLifetimeModifier {
            gradient: color_gradient,
            blend: ColorBlendMode::Overwrite,
            mask: ColorBlendMask::RGBA,
        })
        .render(SizeOverLifetimeModifier {
            gradient: size_gradient,
            screen_space_size: false,
        });

    commands.insert_resource(CastRingEffect(effects.add(effect)));
}

pub fn spawn(ctx: &VfxContext, effect: &CastRingEffect, commands: &mut Commands) {
    commands.spawn((
        ParticleEffect::new(effect.0.clone()),
        Transform::from_translation(pos2_to_vec3(ctx.position.x, ctx.position.y, 0.02)),
    ));
}
