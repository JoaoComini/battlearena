use bevy::prelude::*;

use crate::event::VfxContext;
use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

pub fn spawn_melee(
    ctx: &VfxContext,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let origin = pos2_to_vec3(ctx.position.x, ctx.position.y, 0.03);
    commands.spawn((
        Mesh3d(meshes.add(Annulus::new(0.3, 0.9))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.3, 0.1, 0.9),
            emissive: LinearRgba::rgb(4.0, 1.0, 0.1),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(origin)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        EffectLifetime::new(0.2),
    ));
}

pub fn spawn_projectile(
    ctx: &VfxContext,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let origin = pos2_to_vec3(ctx.position.x, ctx.position.y, 0.03);
    commands.spawn((
        Mesh3d(meshes.add(Annulus::new(0.1, 0.4))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.4, 0.8, 1.0, 0.9),
            emissive: LinearRgba::rgb(0.5, 2.0, 4.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(origin)
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        EffectLifetime::new(0.12),
    ));
}
