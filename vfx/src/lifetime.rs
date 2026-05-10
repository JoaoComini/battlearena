use bevy::prelude::*;

#[derive(Component)]
pub struct EffectLifetime(pub Timer);

impl EffectLifetime {
    pub fn new(secs: f32) -> Self {
        Self(Timer::from_seconds(secs, TimerMode::Once))
    }
}

pub fn tick_effect_lifetimes(
    mut commands: Commands,
    mut effects: Query<(Entity, &mut EffectLifetime)>,
    time: Res<Time>,
) {
    for (entity, mut lifetime) in &mut effects {
        lifetime.0.tick(time.delta());
        if lifetime.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
