use bevy::asset::LoadedFolder;
use bevy::prelude::*;
use std::collections::HashMap;
use crate::types::CharacterDef;

#[derive(Resource, Default)]
pub struct CharacterRegistry {
    pub handles: HashMap<String, Handle<CharacterDef>>,
}

impl CharacterRegistry {
    pub fn get<'a>(
        &self,
        key: &str,
        assets: &'a Assets<CharacterDef>,
    ) -> Option<&'a CharacterDef> {
        self.handles.get(key).and_then(|h| assets.get(h))
    }

    pub fn all_loaded(&self, assets: &Assets<CharacterDef>) -> bool {
        !self.handles.is_empty() && self.handles.values().all(|h| assets.get(h).is_some())
    }
}

/// Set to true once every handle in `CharacterRegistry` has a loaded `CharacterDef`.
#[derive(Resource, Default)]
pub struct CharactersReady(pub bool);

/// Holds the folder handle so the asset server keeps loading the directory.
#[derive(Resource)]
pub struct CharacterFolderHandle(pub Handle<LoadedFolder>);

pub fn load_characters(asset_server: Res<AssetServer>, mut commands: Commands) {
    let handle = asset_server.load_folder("characters/");
    commands.insert_resource(CharacterFolderHandle(handle));
}

pub fn check_characters_ready(
    registry: Res<CharacterRegistry>,
    assets: Res<Assets<CharacterDef>>,
    mut ready: ResMut<CharactersReady>,
) {
    if !ready.0 && registry.all_loaded(&assets) {
        ready.0 = true;
        info!("All character definitions loaded ({} characters)", registry.handles.len());
    }
}

pub fn register_loaded_characters(
    folder_handle: Option<Res<CharacterFolderHandle>>,
    folders: Res<Assets<LoadedFolder>>,
    mut registry: ResMut<CharacterRegistry>,
) {
    let Some(folder_handle) = folder_handle else { return };
    let Some(folder) = folders.get(&folder_handle.0) else { return };

    for untyped in &folder.handles {
        let Some(path) = untyped.path() else { continue };
        let file_name = path.path().file_name().unwrap_or_default().to_string_lossy();
        if !file_name.ends_with(".character.ron") {
            continue;
        }
        let key = file_name.trim_end_matches(".character.ron").to_string();
        if registry.handles.contains_key(&key) {
            continue;
        }
        let handle: Handle<CharacterDef> = untyped.clone().typed();
        info!("Registered character '{}'", key);
        registry.handles.insert(key, handle);
    }
}
