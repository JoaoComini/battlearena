#![allow(unused_imports)]
#![allow(unused_variables)]
use core::net::{Ipv4Addr, SocketAddr};

use bevy::prelude::*;
use bevy::ecs::lifecycle::HookContext;
use bevy::ecs::world::DeferredWorld;
use lightyear::netcode::{NetcodeServer, NetcodeConfig, PRIVATE_KEY_BYTES};
use lightyear::prelude::server::*;
use lightyear::prelude::*;

use crate::common::shared::SharedSettings;

#[derive(Clone, Debug)]
pub struct ServerTransports {
    pub local_port: u16,
}

#[derive(Component, Debug)]
#[component(on_add = ExampleServer::on_add)]
pub struct ExampleServer {
    pub conditioner: Option<RecvLinkConditioner>,
    pub transport: ServerTransports,
    pub shared: SharedSettings,
}

impl ExampleServer {
    fn on_add(mut world: DeferredWorld, context: HookContext) {
        let entity = context.entity;
        world.commands().queue(move |world: &mut World| -> Result {
            let mut entity_mut = world.entity_mut(entity);
            let settings = entity_mut.take::<ExampleServer>().unwrap();
            entity_mut.insert(Name::from("Server"));

            let private_key = settings.shared.private_key;
            entity_mut.insert(NetcodeServer::new(NetcodeConfig {
                protocol_id: settings.shared.protocol_id,
                private_key,
                ..Default::default()
            }));

            let server_addr = SocketAddr::new(
                Ipv4Addr::UNSPECIFIED.into(),
                settings.transport.local_port,
            );
            entity_mut.insert((LocalAddr(server_addr), ServerUdpIo::default()));

            Ok(())
        });
    }
}

pub(crate) fn start(mut commands: Commands, server: Single<Entity, With<Server>>) {
    commands.trigger(Start {
        entity: server.into_inner(),
    });
}
