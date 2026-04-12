use avian2d::prelude::Position;
use bevy::prelude::*;
use bevy_renet::{renet::DefaultChannel, RenetServer};
use shared::{
    components::NetworkId,
    physics::PhysicsInput,
    protocol::S2C,
    types::Pos2,
};

use crate::resources::{
    InputQueue, LastSimulatedTick, PendingVerification, PlayerPosition,
    CORRECTION_EPSILON_SQ,
};

/// Drains one input per player from the queue and writes PhysicsInput for the
/// shared movement system to consume. Records PendingVerification when needed.
pub fn prepare_physics_inputs(
    mut commands: Commands,
    mut players: Query<(Entity, &NetworkId, &mut InputQueue, &mut LastSimulatedTick)>,
) {
    for (entity, net_id, mut queue, mut last_simulated) in &mut players {
        let Some((mv, verify)) = queue.0.pop_front() else { continue };

        last_simulated.0 = mv.tick;
        commands.entity(entity).insert(PhysicsInput(mv.input));

        if verify {
            commands.entity(entity).insert(PendingVerification {
                tick: mv.tick,
                reported_pos: mv.pos,
                client_id: net_id.0,
            });
        }
    }
}

/// After the shared movement system has flushed Position, compare the authoritative
/// position against what the client reported and send Ack or Correction.
pub fn verify_and_respond(
    mut commands: Commands,
    mut server: ResMut<RenetServer>,
    mut players: Query<(Entity, &Position, &PendingVerification, &mut PlayerPosition)>,
) {
    for (entity, position, verification, mut player_pos) in &mut players {
        let server_pos = Pos2 { x: position.0.x, y: position.0.y };
        player_pos.0 = server_pos;

        let dx = server_pos.x - verification.reported_pos.x;
        let dy = server_pos.y - verification.reported_pos.y;
        let msg = if dx * dx + dy * dy > CORRECTION_EPSILON_SQ {
            S2C::Correction { tick: verification.tick, pos: server_pos }
        } else {
            S2C::Ack { tick: verification.tick }
        };

        if let Ok(bytes) = postcard::to_allocvec(&msg) {
            server.send_message(verification.client_id, DefaultChannel::ReliableOrdered, bytes);
        }

        commands.entity(entity).remove::<PendingVerification>();
    }
}
