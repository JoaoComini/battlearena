use bevy::prelude::*;

use crate::event::VfxContext;
use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

pub fn spawn(
    ctx: &VfxContext,
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let duration = ctx.cast_duration.unwrap_or(0.3);
    commands.spawn((
        Mesh3d(meshes.add(Circle::new(1.2))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(0.3, 0.6, 1.0, 0.5),
            emissive: LinearRgba::rgb(0.5, 1.0, 2.0),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(pos2_to_vec3(ctx.position.x, ctx.position.y, 0.02))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        EffectLifetime::new(duration),
    ));
}
