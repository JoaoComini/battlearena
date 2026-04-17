use abilities::types::{AbilityLoadout, AbilitySlot};
use avian2d::prelude::*;
use protocol::*;
use shared::SEND_INTERVAL;
use bevy::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

pub struct BattleArenaServerPlugin;

impl Plugin for BattleArenaServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
        // app.add_systems(Startup, spawn_dummy);
        // app.add_systems(FixedUpdate, (check_dummy_health, tick_dummy_respawn).chain());
    }
}

#[derive(Resource)]
struct DummyRespawnTimer(Timer);

fn spawn_dummy(mut commands: Commands) {
    commands.spawn((
        Dummy,
        Health { current: 100.0, max: 100.0 },
        Position::from_xy(150.0, 0.0),
        RigidBody::Static,
        Collider::circle(25.0),
        Replicate::to_clients(NetworkTarget::All),
        InterpolationTarget::to_clients(NetworkTarget::All),
    ));
}

fn check_dummy_health(
    mut commands: Commands,
    query: Query<(Entity, &Health), With<Dummy>>,
    timer: Option<Res<DummyRespawnTimer>>,
) {
    if timer.is_some() {
        return;
    }
    for (entity, health) in &query {
        if health.current <= 0.0 {
            commands.entity(entity).despawn();
            commands.insert_resource(DummyRespawnTimer(Timer::from_seconds(5.0, TimerMode::Once)));
        }
    }
}

fn tick_dummy_respawn(
    mut commands: Commands,
    timer: Option<ResMut<DummyRespawnTimer>>,
    time: Res<Time>,
) {
    let Some(mut timer) = timer else { return };
    timer.0.tick(time.delta());
    if timer.0.just_finished() {
        commands.remove_resource::<DummyRespawnTimer>();
        commands.spawn((
            Dummy,
            Health { current: 100.0, max: 100.0 },
            Position::from_xy(150.0, 0.0),
            RigidBody::Static,
            Collider::circle(25.0),
            Replicate::to_clients(NetworkTarget::All),
            InterpolationTarget::to_clients(NetworkTarget::All),
        ));
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
    mut commands: Commands,
) {
    let Ok(client_id) = query.get(trigger.entity) else {
        return;
    };
    let client_id = client_id.0;
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
            Health { current: 100.0, max: 100.0 },
            AbilityLoadout {
                slots: vec![AbilitySlot::new("melee")],
            },
        ))
        .id();
    info!(
        "Create player entity {:?} for client {:?}",
        entity, client_id
    );
}
