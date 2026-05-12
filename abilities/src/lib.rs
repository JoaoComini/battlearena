pub mod abilities;
pub mod attributes;
pub mod client;
pub mod debug;
pub mod server;
pub mod systems;
pub mod types;

pub use attributes::{Attribute, Energy, Health, Modifier, MovementSpeed};
pub use types::CommandsAbilityExt;

use crate::types::{AbilityCooldowns, AbilityDef, AbilityInstance, Active};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct AbilitySharedPlugin;

impl Plugin for AbilitySharedPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AbilityDef>()
            .register_asset_loader(RonAbilityLoader);

        app.register_component::<Health>();
        app.register_component::<Energy>();
        app.register_component::<MovementSpeed>()
            .add_prediction()
            .add_should_rollback(|a: &MovementSpeed, b: &MovementSpeed| (a.0 - b.0).abs() >= 0.001);
        app.register_component::<AbilityCooldowns>();
        app.register_component::<AbilityInstance>();
        app.register_component::<Active>();
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
