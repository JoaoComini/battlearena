use abilities::types::{AbilityInstance, Active};
use avian2d::prelude::Position;
use bevy::prelude::*;

use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

pub fn on_active_added(
    trigger: On<Add, Active>,
    instances: Query<&AbilityInstance>,
    positions: Query<&Position>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(instance) = instances.get(trigger.entity) else {
        return;
    };
    let Ok(pos) = positions.get(instance.caster) else {
        return;
    };

    // Outer burst ring
    commands.spawn((
        Mesh3d(meshes.add(Annulus::new(0.3, 0.9))),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: Color::srgba(1.0, 0.8, 0.2, 0.9),
            emissive: LinearRgba::rgb(4.0, 2.0, 0.2),
            alpha_mode: AlphaMode::Blend,
            unlit: true,
            ..default()
        })),
        Transform::from_translation(pos2_to_vec3(pos.x, pos.y, 0.03))
            .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        EffectLifetime::new(0.15),
    ));
}
