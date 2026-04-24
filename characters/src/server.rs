use bevy::prelude::*;
use physics::MovementSpeed;
use protocol::{CharacterType, Health};

use crate::registry::CharacterRegistry;
use crate::types::CharacterDef;

pub struct CharactersServerPlugin;

impl Plugin for CharactersServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_character_type_added);
    }
}

fn on_character_type_added(
    trigger: On<Add, CharacterType>,
    char_types: Query<&CharacterType>,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(char_type) = char_types.get(entity) else {
        return;
    };
    let char_key = char_type.0.as_str();

    let (max_health, move_speed) = if let Some(def) = registry.get(char_key, &char_assets) {
        (def.max_health, def.move_speed)
    } else {
        warn!("CharacterDef '{}' not yet loaded, using defaults", char_key);
        (100.0, 200.0)
    };

    commands.entity(entity).insert((
        Health {
            current: max_health,
            max: max_health,
        },
        MovementSpeed(move_speed),
    ));
}
