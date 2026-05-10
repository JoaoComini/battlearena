use bevy::prelude::*;

use crate::sprite_anim::SpriteAnim;

#[derive(Component)]
pub struct EffectLifetime(pub Timer);

impl EffectLifetime {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

/// When present on an entity with `EffectLifetime`, plays this anim once
/// instead of despawning when the lifetime expires.
#[derive(Component)]
pub struct OutroAnim(pub SpriteAnim);

pub fn tick_effect_lifetimes(
    mut commands: Commands,
    mut effects: Query<(
        Entity,
        &mut EffectLifetime,
        Option<&OutroAnim>,
        Option<&MeshMaterial3d<StandardMaterial>>,
    )>,
    mut anims: Query<&mut SpriteAnim>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
) {
    for (entity, mut lifetime, outro, mat_handle) in &mut effects {
        lifetime.0.tick(time.delta());
        if !lifetime.0.just_finished() {
            continue;
        }

        if let Some(OutroAnim(outro_anim)) = outro {
            // Swap the SpriteAnim to the outro
            if let Ok(mut anim) = anims.get_mut(entity) {
                *anim = outro_anim.clone();
                anim.looping = false;
                anim.current_frame = 0;
            }
            // Update the material to the outro's first frame
            if let Some(handle) = mat_handle {
                if let Some(mat) = materials.get_mut(handle) {
                    *mat = outro_anim.initial_material();
                }
            }
            commands.entity(entity).remove::<EffectLifetime>().remove::<OutroAnim>();
        } else {
            commands.entity(entity).despawn();
        }
    }
}
