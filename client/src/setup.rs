#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};
use std::time::Duration;

use bevy::prelude::*;

use shared::SharedSettings;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use lightyear::netcode::client_plugin::NetcodeConfig;
use lightyear::netcode::NetcodeClient;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ClientTransports {
    Udp,
    #[cfg(feature = "steam")]
    Steam,
}

/// Event that examples can trigger to spawn a client.
#[derive(Component, Clone, Debug)]
#[component(on_add = BattleArenaClient::on_add)]
pub struct BattleArenaClient {
    pub client_id: u64,
    /// The client port to listen on
    pub client_port: u16,
    /// The socket address of the server
    pub server_addr: SocketAddr,
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ClientTransports,
    pub shared: SharedSettings,
}

impl BattleArenaClient {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<BattleArenaClient>().unwrap();
            let client_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), settings.client_port);
            entity_mut.insert((
                Client::default(),
                Link::new(settings.conditioner.clone()),
                LocalAddr(client_addr),
                PeerAddr(settings.server_addr),
                ReplicationReceiver::default(),
                PredictionManager::default(),
                Name::from("Client"),
                lightyear::interpolation::timeline::InterpolationConfig {
                    min_delay: Duration::from_millis(100),
                    send_interval_ratio: 0.0,
                    ..default()
                },
            ));

            let add_netcode = |entity_mut: &mut EntityWorldMut| -> Result {
                // use dummy zeroed key explicitly here.
                let auth = Authentication::Manual {
                    server_addr: settings.server_addr,
                    client_id: settings.client_id,
                    private_key: settings.shared.private_key,
                    protocol_id: settings.shared.protocol_id,
                };
                let netcode_config = NetcodeConfig {
                    // Make sure that the server times out clients when their connection is closed
                    client_timeout_secs: 3,
                    token_expire_secs: -1,
                    ..default()
                };
                entity_mut.insert(NetcodeClient::new(auth, netcode_config)?);
                Ok(())
            };

            match settings.transport {
                #[cfg(feature = "udp")]
                ClientTransports::Udp => {
                    add_netcode(&mut entity_mut)?;
                    entity_mut.insert(UdpIo::default());
                }
                #[cfg(feature = "steam")]
                ClientTransports::Steam => {
                    entity_mut.insert(SteamClientIo {
                        target: ConnectTarget::Addr(settings.server_addr),
                        config: Default::default(),
                    });
                }
            };
            Ok(())
        });
    }
}

pub fn connect(mut commands: Commands, client: Single<Entity, With<Client>>) {
    commands.trigger(Connect {
        entity: client.into_inner(),
    });
}
