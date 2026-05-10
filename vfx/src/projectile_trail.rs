use abilities::types::ProjectileHitbox;
use bevy::prelude::*;

use crate::lifetime::EffectLifetime;

#[derive(Component, Default)]
pub struct TrailEmitter {
    frame: u32,
}

pub fn on_projectile_spawned(
    trigger: On<Add, ProjectileHitbox>,
    mut commands: Commands,
) {
    let emitter = commands.spawn(TrailEmitter::default()).id();
    commands.entity(trigger.entity).add_child(emitter);
}

pub fn tick_trail_emitters(
    mut emitters: Query<(&GlobalTransform, &mut TrailEmitter)>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for (transform, mut emitter) in &mut emitters {
        emitter.frame += 1;
        if emitter.frame % 3 != 0 {
            continue;
        }

        let pos = transform.translation();
        commands.spawn((
            Mesh3d(meshes.add(Sphere::new(0.08))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.4, 0.8, 1.0, 0.8),
                emissive: LinearRgba::rgb(0.5, 1.5, 3.0),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(pos),
            EffectLifetime::new(0.15),
        ));
    }
}
