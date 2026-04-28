pub mod animation;
pub mod dummy;
pub mod movement;
pub mod registry;
pub mod types;

pub use animation::CharacterAnimationPlugin;
pub use dummy::{DummyClientPlugin, DummyServerPlugin};
pub use movement::MovementPlugin;
pub use types::{Character, CharacterId};

pub use crate::registry::CharactersReady;
use crate::registry::{
    check_characters_ready, load_characters, register_loaded_characters, CharacterRegistry,
};
use crate::types::{CharacterDef, CharacterDefRaw};
use bevy::asset::io::Reader;
use bevy::asset::{AssetLoader, LoadContext};
use bevy::prelude::*;
use lightyear::prelude::*;

pub struct CharactersPlugin;

impl Plugin for CharactersPlugin {
    fn build(&self, app: &mut App) {
        app.register_component::<CharacterId>();
        app.init_asset::<CharacterDef>()
            .register_asset_loader(RonCharacterLoader)
            .init_resource::<CharacterRegistry>()
            .init_resource::<CharactersReady>()
            .add_systems(Startup, load_characters)
            .add_systems(
                Update,
                (register_loaded_characters, check_characters_ready).chain(),
            )
            .add_plugins((CharacterAnimationPlugin, MovementPlugin))
            .add_observer(on_character_id_added);
    }
}

fn on_character_id_added(
    trigger: On<Add, CharacterId>,
    query: Query<&CharacterId>,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(char_id) = query.get(entity) else { return };
    let Some(handle) = registry.handles.get(&char_id.0).cloned() else { return };
    let Some(def) = char_assets.get(&handle) else { return };

    let mut entity_cmd = commands.entity(entity);
    entity_cmd.insert((Character(handle), def.to_bundle()));
    if let Some(ref scene) = def.visual {
        entity_cmd.insert(SceneRoot(scene.clone()));
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
        load_context: &mut LoadContext<'_>,
    ) -> Result<Self::Asset, Self::Error> {
        let mut bytes = Vec::new();
        reader.read_to_end(&mut bytes).await?;
        let raw: CharacterDefRaw = ron::de::from_bytes(&bytes)?;
        let ability_slots = raw
            .ability_slots
            .iter()
            .map(|path| load_context.load(path))
            .collect();
        let visual = raw.visual.as_deref().map(|path| {
            load_context.load(bevy::gltf::GltfAssetLabel::Scene(0).from_asset(path.to_string()))
        });
        Ok(CharacterDef {
            key: raw.key,
            max_health: raw.max_health,
            move_speed: raw.move_speed,
            ability_slots,
            playable: raw.playable,
            visual,
        })
    }

    fn extensions(&self) -> &[&str] {
        &["character.ron"]
    }
}
