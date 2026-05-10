use avian2d::prelude::Position;
use bevy::prelude::*;
use protocol::VfxTag;

use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

#[derive(Component)]
pub struct ProjectileTrailEmitter {
    pub tracked_entity: Entity,
    frame: u32,
}

pub fn on_projectile_trail_tag(
    trigger: On<Add, VfxTag>,
    tags: Query<(&VfxTag, &Position)>,
    mut commands: Commands,
) {
    let Ok((tag, pos)) = tags.get(trigger.entity) else { return };
    if tag.0 != "projectile_trail" { return }

    commands.spawn((
        ProjectileTrailEmitter {
            tracked_entity: trigger.entity,
            frame: 0,
        },
        Transform::from_translation(pos2_to_vec3(pos.x, pos.y, 0.5)),
    ));
}

pub fn tick_trail_emitters(
    mut emitters: Query<(Entity, &mut ProjectileTrailEmitter, &mut Transform)>,
    tracked: Query<&Position>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (emitter_entity, mut emitter, mut transform) in &mut emitters {
        match tracked.get(emitter.tracked_entity) {
            Ok(pos) => {
                // Follow the actual projectile position
                transform.translation = pos2_to_vec3(pos.x, pos.y, 0.5);
            }
            Err(_) => {
                // Projectile despawned — remove the emitter
                commands.entity(emitter_entity).despawn();
                continue;
            }
        }

        emitter.frame += 1;
        if emitter.frame % 3 != 0 {
            continue;
        }

        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.08))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.4, 0.8, 1.0, 0.8),
                emissive: LinearRgba::rgb(0.5, 1.5, 3.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(transform.translation),
            EffectLifetime::new(0.15),
        ));
    }
}
