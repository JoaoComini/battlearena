pub mod client;
pub mod debug;
pub mod server;
pub mod systems;
pub mod types;

use crate::types::{AbilityCooldowns, AbilityDef, AbilityInstance, Active, Casting};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct AbilitySharedPlugin;

impl Plugin for AbilitySharedPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<AbilityDef>()
            .register_asset_loader(RonAbilityLoader);

        app.register_component::<AbilityCooldowns>();
        app.register_component::<AbilityInstance>();
        app.register_component::<Casting>();
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
