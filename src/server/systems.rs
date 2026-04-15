use crate::protocol::*;
use crate::shared::SEND_INTERVAL;
use bevy::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use lightyear::prelude::MessageReceiver;

pub struct BattleArenaServerPlugin;

impl Plugin for BattleArenaServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
        app.add_systems(Update, spawn_player_on_selection);
    }
}

pub(crate) fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from("Client"),
    ));
}

#[derive(Component)]
pub(crate) struct PendingPlayer;

pub(crate) fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    info!("Client {:?} connected, awaiting character selection", client_id.0);
    commands.entity(trigger.entity).insert(PendingPlayer);
}

pub(crate) fn spawn_player_on_selection(
    mut client_query: Query<(Entity, &RemoteId, &mut MessageReceiver<SelectCharacter>), With<PendingPlayer>>,
    mut commands: Commands,
) {
    for (client_entity, remote_id, mut receiver) in &mut client_query {
        for msg in receiver.receive() {
            let SelectCharacter(kind) = msg;
            let sender_peer = remote_id.0;

            let stats = match kind.load_stats() {
                Ok(s) => s,
                Err(e) => {
                    error!("Failed to load stats for {:?}: {}", kind, e);
                    continue;
                }
            };

            let entity = commands
                .spawn((
                    PlayerBundle::new(sender_peer, kind, &stats),
                    Replicate::to_clients(NetworkTarget::All),
                    PredictionTarget::to_clients(NetworkTarget::Single(sender_peer)),
                    InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(sender_peer)),
                    ControlledBy {
                        owner: client_entity,
                        lifetime: Default::default(),
                    },
                    DisableReplicateHierarchy,
                ))
                .id();

            commands.entity(client_entity).remove::<PendingPlayer>();
            info!(
                "Spawned {:?} player entity {:?} for client {:?}",
                kind, entity, sender_peer
            );
        }
    }
}
