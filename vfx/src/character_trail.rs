use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use bevy_hanabi::prelude::*;
use protocol::PlayerId;

use crate::util::pos2_to_vec3;

#[derive(Resource, Default)]
pub struct CharacterTrailState {
    frame: u32,
    foot: bool,
}

#[derive(Resource)]
pub struct FootstepEffect(pub Handle<EffectAsset>);

pub fn setup_footstep_effect(mut commands: Commands, mut effects: ResMut<Assets<EffectAsset>>) {
    let writer = ExprWriter::new();

    let init_pos = SetPositionSphereModifier {
        center: writer.lit(Vec3::ZERO).expr(),
        radius: writer.lit(0.12_f32).expr(),
        dimension: ShapeDimension::Surface,
    };

    let init_vel = SetVelocityTangentModifier {
        origin: writer.lit(Vec3::ZERO).expr(),
        axis: writer.lit(Vec3::Y).expr(),
        speed: writer.lit(0.3_f32).expr(),
    };

    let lifetime = writer.lit(0.5_f32).expr();
    let init_lifetime = SetAttributeModifier::new(Attribute::LIFETIME, lifetime);
    let age = writer.lit(0_f32).expr();
    let init_age = SetAttributeModifier::new(Attribute::AGE, age);

    let mut color = bevy_hanabi::Gradient::new();
    color.add_key(0.0, Vec4::new(0.5, 1.5, 4.0, 1.0));
    color.add_key(0.5, Vec4::new(0.2, 0.8, 2.0, 0.6));
    color.add_key(1.0, Vec4::new(0.1, 0.3, 1.0, 0.0));

    let mut size = bevy_hanabi::Gradient::new();
    size.add_key(0.0, Vec3::splat(0.05));
    size.add_key(1.0, Vec3::splat(0.0));

    let effect = EffectAsset::new(256, SpawnerSettings::once(20.0.into()), writer.finish())
        .with_name("footstep")
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

    commands.insert_resource(FootstepEffect(effects.add(effect)));
}

pub fn spawn_character_trail(
    players: Query<(&Position, &LinearVelocity), With<PlayerId>>,
    mut state: ResMut<CharacterTrailState>,
    effect: Res<FootstepEffect>,
    mut commands: Commands,
) {
    state.frame += 1;
    if state.frame % 16 != 0 {
        return;
    }
    state.foot = !state.foot;

    for (pos, vel) in &players {
        if vel.0.length() < 0.5 {
            continue;
        }

        let dir = vel.0.normalize();
        let behind = -dir * 0.3;
        let side = Vec2::new(-dir.y, dir.x) * if state.foot { 0.2 } else { -0.2 };
        let px = pos.x + behind.x + side.x;
        let py = pos.y + behind.y + side.y;

        commands.spawn((
            ParticleEffect::new(effect.0.clone()),
            Transform::from_translation(pos2_to_vec3(px, py, 0.0)),
        ));
    }
}
