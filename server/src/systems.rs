use abilities::types::{AbilityCast, AbilityLoadout, AbilitySlot};
use characters::registry::CharacterRegistry;
use characters::types::CharacterDef;
use physics::MovementSpeed;
use protocol::*;
use shared::SEND_INTERVAL;
use bevy::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

#[derive(Resource, Default)]
pub(crate) struct PlayerCounter(u32);

pub struct BattleArenaServerPlugin;

impl Plugin for BattleArenaServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCounter>();
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
    }
}

pub(crate) fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from("Client"),
    ));
}

pub(crate) fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut counter: ResMut<PlayerCounter>,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let client_id = client_id.0;

    let char_key = if counter.0 % 2 == 0 { "comini" } else { "kaps" };
    counter.0 += 1;

    let (max_health, move_speed, ability_slots) =
        if let Some(def) = registry.get(char_key, &char_assets) {
            (def.max_health, def.move_speed, def.ability_slots.clone())
        } else {
            warn!(
                "CharacterDef '{}' not yet loaded, using defaults",
                char_key
            );
            (100.0, 200.0, vec!["melee".to_string()])
        };

    let loadout = AbilityLoadout {
        slots: ability_slots.iter().map(|k| AbilitySlot::new(k)).collect(),
    };

    let entity = commands
        .spawn((
            PlayerBundle::new(client_id, Vec2::ZERO),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner: trigger.entity,
                lifetime: Default::default(),
            },
            DisableReplicateHierarchy,
            Health { current: max_health, max: max_health },
            MovementSpeed(move_speed),
            CharacterType(char_key.to_string()),
            loadout,
            AbilityCast::default(),
        ))
        .id();

    info!(
        "Spawned player {:?} as '{}' for client {:?}",
        entity, char_key, client_id
    );
}
