use crate::protocol::*;
use crate::shared;
use bevy::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::input::native::*;
use lightyear::prelude::server::*;
use lightyear::prelude::*;

pub struct ExampleServerPlugin;

impl Plugin for ExampleServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, movement);
        app.add_observer(handle_new_client);
        app.add_observer(handle_connected);
    }
}

/// When a new client link entity is created, attach a ReplicationSender so the
/// server knows to replicate entities to that client.
pub fn handle_new_client(trigger: On<Add, LinkOf>, mut commands: Commands) {
    commands.entity(trigger.entity).insert((
        ReplicationSender::new(
            shared::SEND_INTERVAL,
            SendUpdatesMode::SinceLastAck,
            false,
        ),
        Name::from("Client"),
    ));
}

/// Once a client is confirmed connected, spawn its player entity and configure
/// replication: everyone sees it, only the owner predicts it, everyone else
/// interpolates it. ControlledBy auto-despawns the entity on disconnect.
pub fn handle_connected(
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
        ))
        .id();

    info!("Spawned player {:?} for client {:?}", entity, client_id);
}

/// Read client inputs and advance player positions authoritatively.
/// The `Without<Predicted>` guard prevents double-applying inputs in
/// host-server mode (where the local client's Predicted entity also exists).
fn movement(
    mut position_query: Query<
        (&mut PlayerPosition, &ActionState<Inputs>),
        Without<Predicted>,
    >,
) {
    for (position, inputs) in position_query.iter_mut() {
        shared::shared_movement_behaviour(position, inputs);
    }
}
