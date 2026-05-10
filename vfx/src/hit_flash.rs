use abilities::Health;
use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

pub fn spawn_hit_flash(
    changed: Query<&Position, Changed<Health>>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for pos in &changed {
        commands.spawn((
            Mesh3d(meshes.add(Annulus::new(0.5, 0.8))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(1.0, 0.4, 0.1, 1.0),
                emissive: LinearRgba::rgb(3.0, 1.0, 0.1),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(pos2_to_vec3(pos.x, pos.y, 0.05))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            EffectLifetime::new(0.3),
        ));
    }
}
