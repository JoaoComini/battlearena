use bevy::prelude::*;

use crate::sprite_anim::SpriteAnim;

#[derive(Component)]
pub struct EffectLifetime(pub Timer);

impl EffectLifetime {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

/// When the `EffectLifetime` expires, spawns a companion entity playing `anim`
/// at the same position (offset by `y_offset`) while the original entity plays
/// one final non-looping cycle of its current anim before despawning.
#[derive(Component)]
pub struct OutroAnim {
    pub anim: SpriteAnim,
    pub y_offset: f32,
    pub rotation: Option<Quat>,
    pub scale: Option<Vec3>,
}

pub fn tick_effect_lifetimes(
    mut commands: Commands,
    mut effects: Query<(
        Entity,
        &mut EffectLifetime,
        Option<&OutroAnim>,
        Option<&MeshMaterial3d<StandardMaterial>>,
        Option<&mut SpriteAnim>,
        Option<&Transform>,
    )>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    time: Res<Time>,
) {
    for (entity, mut lifetime, outro, mat_handle, anim, transform) in &mut effects {
        lifetime.0.tick(time.delta());
        if !lifetime.0.just_finished() {
            continue;
        }

        if let Some(outro) = outro {
            // Play one final non-looping cycle on the original entity then despawn
            if let Some(mut a) = anim {
                a.looping = false;
                a.current_frame = 0;
            }
            // Give it just enough lifetime to finish one cycle
            commands.entity(entity)
                .remove::<OutroAnim>()
                .insert(EffectLifetime::new(outro.anim.frame_count as f32 * outro.anim.timer.duration().as_secs_f32()));

            // Spawn companion VFX3 entity at the same position
            if let Some(t) = transform {
                let mut companion_pos = t.translation;
                companion_pos.y += outro.y_offset;

                let companion_rot = outro.rotation.unwrap_or(Quat::IDENTITY);
                let companion_scale = outro.scale.unwrap_or(Vec3::splat(t.scale.x));

                let mat = materials.add(outro.anim.initial_material());
                let mut companion_anim = outro.anim.clone();
                companion_anim.looping = false;

                commands.spawn((
                    Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
                    MeshMaterial3d(mat),
                    Transform::from_translation(companion_pos)
                        .with_rotation(companion_rot)
                        .with_scale(companion_scale),
                    companion_anim,
                ));
            }
        } else {
            commands.entity(entity).despawn();
        }
    }
}
