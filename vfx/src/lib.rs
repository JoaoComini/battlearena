mod character_trail;
mod dispatcher;
mod effects;
mod event;
mod hit_flash;
mod util;

use bevy::prelude::*;
use bevy_hanabi::prelude::*;

use character_trail::{setup_footstep_effect, spawn_character_trail, CharacterTrailState};
use dispatcher::on_vfx_tag;
use effects::projectile_trail::tick_trail_emitters;
use hit_flash::spawn_hit_flash;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(HanabiPlugin);

        app.init_resource::<CharacterTrailState>();

        app.add_observer(on_vfx_tag);

        app.add_systems(Startup, (
            setup_footstep_effect,
            effects::cast_ring::setup,
            effects::ability_burst::setup,
            effects::projectile_trail::setup,
            hit_flash::setup,
        ));

        app.add_systems(Update, (
            spawn_character_trail,
            tick_trail_emitters,
            spawn_hit_flash,
        ));
    }
}
