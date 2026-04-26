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
}

pub fn load_characters(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<CharacterRegistry>,
) {
    for path in [
        "characters://comini.character.ron",
        "characters://kaps.character.ron",
        "characters://dummy.character.ron",
    ] {
        let handle: Handle<CharacterDef> = asset_server.load(path);
        let key = path
            .trim_end_matches(".character.ron")
            .split('/')
            .last()
            .unwrap()
            .to_string();
        registry.handles.insert(key, handle);
    }
}
