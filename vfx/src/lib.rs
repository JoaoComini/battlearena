mod character_trail;
mod dispatcher;
mod effects;
mod event;
mod hit_flash;
mod lifetime;
mod sprite_anim;
mod util;

use character_trail::{
    load_footstep_assets, on_footprint_ready, spawn_character_trail, tick_footprint_players,
    CharacterTrailState,
};
use dispatcher::on_vfx_tag;
use effects::projectile_trail::tick_trail_emitters;
use hit_flash::spawn_hit_flash;
use lifetime::tick_effect_lifetimes;
use sprite_anim::tick_sprite_anims;

use bevy::prelude::*;

pub struct VfxPlugin;

impl Plugin for VfxPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CharacterTrailState>();

        app.add_observer(on_vfx_tag);
        app.add_observer(on_footprint_ready);

        app.add_systems(Startup, load_footstep_assets);

        app.add_systems(
            Update,
            (
                tick_sprite_anims,
                tick_effect_lifetimes,
                spawn_hit_flash,
                tick_trail_emitters,
                spawn_character_trail,
                tick_footprint_players,
            ),
        );
    }
}
