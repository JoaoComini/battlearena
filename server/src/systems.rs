use protocol::*;
use shared::SEND_INTERVAL;
use bevy::prelude::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

pub struct BattleArenaServerPlugin;

impl Plugin for BattleArenaServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_new_client);
        app.add_systems(FixedUpdate, receive_character_selections);
    }
}

pub(crate) fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from("Client"),
    ));
}

pub(crate) fn receive_character_selections(
    mut receivers: Query<(Entity, &RemoteId, &mut MessageReceiver<SelectCharacter>)>,
    mut commands: Commands,
) {
    for (conn_entity, remote_id, mut receiver) in &mut receivers {
        for msg in receiver.receive() {
            let client_id = remote_id.0;

            let entity = commands
                .spawn((
                    PlayerBundle::new(client_id, Vec2::ZERO),
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
                    ControlledBy {
                        owner: conn_entity,
                        lifetime: Default::default(),
                    },
                    DisableReplicateHierarchy,
                    CharacterType(msg.key),
                ))
                .id();

            info!("Spawned player {:?} for client {:?}", entity, client_id);
        }
    }
}
