pub mod dummy;
pub mod registry;
pub mod types;

pub use dummy::DummyPlugin;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use crate::registry::{
    check_characters_ready, load_characters, register_loaded_characters, CharacterRegistry,
};
pub use crate::registry::CharactersReady;
use crate::types::CharacterDef;

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<CharacterDef>()
            .register_asset_loader(RonCharacterLoader)
            .init_resource::<CharacterRegistry>()
            .init_resource::<CharactersReady>()
            .add_systems(Startup, load_characters)
            .add_systems(Update, (register_loaded_characters, check_characters_ready).chain());
    }
}

#[derive(Default, TypePath)]
struct RonCharacterLoader;

impl AssetLoader for RonCharacterLoader {
    type Asset = CharacterDef;
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
        let def: CharacterDef = ron::de::from_bytes(&bytes)?;
        Ok(def)
    }

    fn extensions(&self) -> &[&str] {
        &["character.ron"]
    }
}
