pub mod registry;
pub mod server;
pub mod types;

use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use crate::registry::{load_characters, CharacterRegistry};
use crate::types::CharacterDef;

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.init_asset::<CharacterDef>()
            .register_asset_loader(RonCharacterLoader)
            .init_resource::<CharacterRegistry>()
            .add_systems(Startup, load_characters);
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
