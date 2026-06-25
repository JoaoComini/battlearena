use abilities::Health;
use avian2d::prelude::Position;
use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use crate::util::pos2_to_vec3;

#[derive(Resource)]
pub struct HitFlashEffect(pub Handle<EffectAsset>);

pub fn setup(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
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
        speed: writer.lit(2.5_f32).expr(),
    };

    let lifetime = writer.lit(0.3_f32).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let age = writer.lit(0_f32).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(3.0, 1.0, 0.1, 1.0));
    color.add_key(0.5, Vec4::new(2.0, 0.4, 0.1, 0.8));
    color.add_key(1.0, Vec4::new(1.0, 0.2, 0.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.08));
    size.add_key(1.0, Vec3::splat(0.0));

    let effect = EffectAsset::new(256, SpawnerSettings::once(80.0.into()), writer.finish())
        .with_name("hit_flash")
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

    commands.insert_resource(HitFlashEffect(effects.add(effect)));
}

pub fn spawn_hit_flash(
    changed: Query<&Position, Changed<Health>>,
    effect: Res<HitFlashEffect>,
    mut commands: Commands,
) {
    for pos in &changed {
        commands.spawn((
            ParticleEffect::new(effect.0.clone()),
            Transform::from_translation(pos2_to_vec3(pos.x, pos.y, 0.05)),
        ));
    }
}
