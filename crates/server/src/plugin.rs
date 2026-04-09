use bevy::prelude::*;
use bevy_renet::RenetServer;

use crate::{
    game::systems::{
        advance_tick, broadcast_despawn, broadcast_state, handle_server_events, receive_inputs,
        send_initial_state, tick_game,
    },
    resources::{CurrentTick, PlayerRegistry},
};

pub struct ServerGamePlugin;

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CurrentTick>()
            .init_resource::<PlayerRegistry>()
            .add_observer(handle_server_events)
            .add_systems(
                FixedUpdate,
                (send_initial_state, broadcast_despawn, receive_inputs, tick_game, broadcast_state, advance_tick)
                    .chain()
                    .run_if(resource_exists::<RenetServer>),
            );
    }
}
