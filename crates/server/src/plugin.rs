use bevy::prelude::*;
use bevy_renet::RenetServer;

use crate::{
    game::systems::{
        broadcast_despawn, broadcast_state, handle_server_events, receive_inputs,
        send_initial_state, tick_game,
    },
    resources::EntityRegistry,
};

pub struct ServerGamePlugin;

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<EntityRegistry>()
            .add_observer(handle_server_events)
            .add_systems(
                FixedUpdate,
                (send_initial_state, broadcast_despawn, receive_inputs, tick_game, broadcast_state)
                    .chain()
                    .run_if(resource_exists::<RenetServer>),
            );
    }
}
