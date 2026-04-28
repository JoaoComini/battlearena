use bevy::prelude::*;
use characters::dummy::Dummy;
use characters::registry::CharacterRegistry;
use characters::types::CharacterDef;
use characters::CharactersReady;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use protocol::*;
use shared::SEND_INTERVAL;

#[derive(Resource, Default)]
pub(crate) struct PlayerCounter(u32);

/// Client owner entities that connected before character assets were ready.
#[derive(Resource, Default)]
struct PendingConnections(Vec<Entity>);

/// Pending respawn for a player that died.
#[derive(Component)]
struct RespawnTicket {
    owner: Entity,
    client_id: PeerId,
    char_key: String,
    timer: Timer,
}

pub struct BattleArenaServerPlugin;

impl Plugin for BattleArenaServerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerCounter>();
        app.init_resource::<PendingConnections>();
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
        app.add_systems(
            Update,
            (flush_pending_connections, check_player_death, tick_respawns).chain(),
        );
    }
}

fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(SEND_INTERVAL, SendUpdatesMode::SinceLastAck, false),
        Name::from("Client"),
    ));
}

fn handle_connected(
    trigger: On<Add, Connected>,
    query: Query<&RemoteId, With<ClientOf>>,
    ready: Res<CharactersReady>,
    mut counter: ResMut<PlayerCounter>,
    mut pending: ResMut<PendingConnections>,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };

    if !ready.0 {
        pending.0.push(trigger.entity);
        return;
    }

    let char_key = if counter.0 % 2 == 0 { "comini" } else { "kaps" };
    counter.0 += 1;

    spawn_player(
        trigger.entity,
        client_id.0,
        char_key,
        &registry,
        &char_assets,
        &mut commands,
    );
}

fn flush_pending_connections(
    ready: Res<CharactersReady>,
    mut pending: ResMut<PendingConnections>,
    query: Query<&RemoteId, With<ClientOf>>,
    mut counter: ResMut<PlayerCounter>,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
    mut commands: Commands,
) {
    if !ready.0 || pending.0.is_empty() {
        return;
    }

    for owner in pending.0.drain(..) {
        let Ok(remote_id) = query.get(owner) else {
            continue;
        };
        let char_key = if counter.0 % 2 == 0 { "comini" } else { "kaps" };
        counter.0 += 1;
        spawn_player(
            owner,
            remote_id.0,
            char_key,
            &registry,
            &char_assets,
            &mut commands,
        );
    }
}

fn spawn_player(
    owner: Entity,
    client_id: PeerId,
    char_key: &str,
    registry: &CharacterRegistry,
    char_assets: &Assets<CharacterDef>,
    commands: &mut Commands,
) {
    let Some(def) = registry.get(char_key, char_assets) else {
        warn!("CharacterDef '{}' not found, skipping spawn", char_key);
        return;
    };

    let entity = commands
        .spawn((
            PlayerBundle::new(client_id, Vec2::ZERO),
            Replicate::to_clients(NetworkTarget::All),
            PredictionTarget::to_clients(NetworkTarget::Single(client_id)),
            InterpolationTarget::to_clients(NetworkTarget::AllExceptSingle(client_id)),
            ControlledBy {
                owner,
                lifetime: Default::default(),
            },
            DisableReplicateHierarchy,
            def.to_bundle(),
        ))
        .id();

    info!(
        "Spawned player {:?} as '{}' for client {:?}",
        entity, char_key, client_id
    );
}

fn check_player_death(
    mut commands: Commands,
    players: Query<(Entity, &Health, &CharacterType, &ControlledBy), Without<Dummy>>,
    owners: Query<&RemoteId, With<ClientOf>>,
) {
    for (entity, health, char_type, controlled_by) in &players {
        if health.is_dead() {
            let Ok(remote_id) = owners.get(controlled_by.owner) else {
                continue;
            };
            info!("Player {:?} (client {:?}) died", entity, remote_id.0);
            commands.entity(entity).despawn();
            commands.spawn(RespawnTicket {
                owner: controlled_by.owner,
                client_id: remote_id.0,
                char_key: char_type.0.clone(),
                timer: Timer::from_seconds(3.0, TimerMode::Once),
            });
        }
    }
}

fn tick_respawns(
    mut commands: Commands,
    mut tickets: Query<(Entity, &mut RespawnTicket)>,
    registry: Res<CharacterRegistry>,
    char_assets: Res<Assets<CharacterDef>>,
    time: Res<Time>,
) {
    for (ticket_entity, mut ticket) in &mut tickets {
        ticket.timer.tick(time.delta());
        if ticket.timer.just_finished() {
            spawn_player(
                ticket.owner,
                ticket.client_id,
                &ticket.char_key,
                &registry,
                &char_assets,
                &mut commands,
            );
            commands.entity(ticket_entity).despawn();
        }
    }
}
