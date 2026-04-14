//! This module introduces a settings struct that can be used to configure the server and client.
#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use core::time::Duration;
use ron;

use crate::shared::SharedSettings;
#[cfg(not(target_family = "wasm"))]
use async_compat::Compat;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
#[cfg(not(target_family = "wasm"))]
use bevy::tasks::IoTaskPool;
use lightyear::netcode::{NetcodeServer, PRIVATE_KEY_BYTES};
use lightyear::prelude::server::*;
use lightyear::prelude::*;
use serde::{Deserialize, Serialize};
use tracing::warn;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[non_exhaustive]
pub enum ServerTransports {
    #[cfg(feature = "udp")]
    Udp {
        local_port: u16,
    },
    WebSocket {
        local_port: u16,
    },
    #[cfg(feature = "steam")]
    Steam {
        local_port: u16,
    },
}

#[derive(Component, Debug)]
#[component(on_add = ExampleServer::on_add)]
pub struct ExampleServer {
    /// Possibly add a conditioner to simulate network conditions
    pub conditioner: Option<RecvLinkConditioner>,
    /// Which transport to use
    pub transport: ServerTransports,
    pub shared: SharedSettings,
}

impl ExampleServer {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ExampleServer>().unwrap();
            entity_mut.insert((Name::from("Server"),));

            let add_netcode = |entity_mut: &mut EntityWorldMut| {
                // Use private key from environment variable, if set. Otherwise from settings file.
                let private_key = if let Some(key) = parse_private_key_from_env() {
                    info!("Using private key from LIGHTYEAR_PRIVATE_KEY env var");
                    key
                } else {
                    settings.shared.private_key
                };
                entity_mut.insert(NetcodeServer::new(NetcodeConfig {
                    protocol_id: settings.shared.protocol_id,
                    private_key,
                    ..Default::default()
                }));
            };
            match settings.transport {
                #[cfg(feature = "udp")]
                ServerTransports::Udp { local_port } => {
                    add_netcode(&mut entity_mut);
                    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                    entity_mut.insert((LocalAddr(server_addr), ServerUdpIo::default()));
                }
                ServerTransports::WebSocket { local_port } => {
                    add_netcode(&mut entity_mut);
                    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                    let sans = vec![
                        "localhost".to_string(),
                        "127.0.0.1".to_string(),
                        "::1".to_string(),
                    ];
                    let config = ServerConfig::builder()
                        .with_bind_address(server_addr)
                        .with_identity(
                            lightyear::websocket::server::Identity::self_signed(sans).unwrap(),
                        );
                    entity_mut.insert((LocalAddr(server_addr), WebSocketServerIo { config }));
                }
                #[cfg(feature = "steam")]
                ServerTransports::Steam { local_port } => {
                    let server_addr = SocketAddr::new(Ipv4Addr::UNSPECIFIED.into(), local_port);
                    entity_mut.insert(SteamServerIo {
                        target: ListenTarget::Addr(server_addr),
                        config: SessionConfig::default(),
                    });
                }
            };
            Ok(())
        });
    }
}

pub(crate) fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger(Start {
        entity: server.into_inner(),
    });
}

/// Reads and parses the LIGHTYEAR_PRIVATE_KEY environment variable into a private key.
pub fn parse_private_key_from_env() -> Option<[u8; PRIVATE_KEY_BYTES]> {
    let Ok(key_str) = std::env::var("LIGHTYEAR_PRIVATE_KEY") else {
        return None;
    };
    let private_key: Vec<u8> = key_str
        .chars()
        .filter(|c| c.is_ascii_digit() || *c == ',')
        .collect::<String>()
        .split(',')
        .map(|s| {
            s.parse::<u8>()
                .expect("Failed to parse number in private key")
        })
        .collect();

    if private_key.len() != PRIVATE_KEY_BYTES {
        panic!("Private key must contain exactly {PRIVATE_KEY_BYTES} numbers",);
    }

    let mut bytes = [0u8; PRIVATE_KEY_BYTES];
    bytes.copy_from_slice(&private_key);
    Some(bytes)
}
