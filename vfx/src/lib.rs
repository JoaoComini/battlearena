mod ability_burst;
mod cast_ring;
mod character_trail;
mod hit_flash;
mod lifetime;
mod projectile_trail;
mod util;

use ability_burst::on_active_added;
use cast_ring::on_casting_added;
use character_trail::{spawn_character_trail, CharacterTrailState};
use hit_flash::spawn_hit_flash;
use lifetime::tick_effect_lifetimes;
use projectile_trail::{on_projectile_spawned, tick_trail_emitters};

use bevy::prelude::*;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterTrailState>();

        app.add_systems(
            Update,
            (
                tick_effect_lifetimes,
                spawn_hit_flash,
                tick_trail_emitters,
                spawn_character_trail,
            ),
        );

        app.add_observer(on_casting_added);
        app.add_observer(on_active_added);
        app.add_observer(on_projectile_spawned);
    }
}
