use bevy::prelude::*;
use std::collections::HashMap;
use crate::types::{AbilityDef, AbilityKey};

#[derive(Resource, Default)]
pub struct AbilityRegistry {
    handles: HashMap<String, Handle<AbilityDef>>,
}

impl AbilityRegistry {
    pub fn get<'a>(
        &self,
        key: &AbilityKey,
        assets: &'a Assets<AbilityDef>,
    ) -> Option<&'a AbilityDef> {
        self.handles.get(key.0.as_str()).and_then(|h: &Handle<AbilityDef>| assets.get(h))
    }
}

pub fn load_abilities(
    asset_server: Res<AssetServer>,
    mut registry: ResMut<AbilityRegistry>,
) {
    for path in ["abilities://abilities/melee.ability.ron"] {
        let handle: Handle<AbilityDef> = asset_server.load(path);
        let key = path
            .trim_end_matches(".ability.ron")
            .split('/')
            .last()
            .unwrap()
            .to_string();
        registry.handles.insert(key, handle);
    }
}
