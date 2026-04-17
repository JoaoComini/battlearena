pub mod registry;
pub mod systems;
pub mod types;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use lightyear::prelude::*;
use crate::registry::{load_abilities, AbilityRegistry};
use crate::systems::{activate_abilities, apply_hitbox_damage, tick_cooldowns};
use crate::types::{AbilityDef, AbilityLoadout};

pub struct AbilityPlugin;

impl Plugin for AbilityPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AbilityDef>()
            .register_asset_loader(RonAbilityLoader)
            .init_resource::<AbilityRegistry>()
            .add_systems(Startup, load_abilities)
            .add_systems(FixedUpdate, (tick_cooldowns, activate_abilities, apply_hitbox_damage).chain());

        app.register_component::<AbilityLoadout>()
            .add_prediction()
            .add_should_rollback(|a: &AbilityLoadout, b: &AbilityLoadout| a != b);
    }
}

#[derive(Default, TypePath)]
struct RonAbilityLoader;

impl AssetLoader for RonAbilityLoader {
    type Asset = AbilityDef;
    type Settings = ();
    type Error = Box<dyn std::error::Error + Send + Sync>;

    async fn load(
        &self,
        reader: &mut dyn Reader,
        _settings: &Self::Settings,
        _load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let def: AbilityDef = ron::de::from_bytes(&bytes)?;
        Ok(def)
    }

    fn extensions(&self) -> &[&str] {
        &["ability.ron"]
    }
}
