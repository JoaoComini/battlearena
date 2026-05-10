use abilities::types::AbilityInstance;
use bevy::prelude::*;
use protocol::VfxTag;

use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

pub fn on_ability_burst_tag(
    trigger: On<Add, VfxTag>,
    tags: Query<&VfxTag>,
    instances: Query<&AbilityInstance>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let Ok(tag) = tags.get(trigger.entity) else { return };
    let Ok(instance) = instances.get(trigger.entity) else { return };
    let origin = pos2_to_vec3(instance.origin.x, instance.origin.y, 0.03);

    match tag.0.as_str() {
        "activate_melee" => {
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
        "activate_projectile" => {
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
        _ => {}
    }
}
