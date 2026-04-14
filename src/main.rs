//! Run with
//! - `cargo run -- server`
//! - `cargo run -- client -c 1`
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(dead_code)]

use crate::cli::{Cli, Mode};
#[cfg(feature = "client")]
use crate::client::BattleArenaClientPlugin;
#[cfg(feature = "server")]
use crate::server::BattleArenaServerPlugin;
use crate::shared::SharedPlugin;
use crate::shared::FIXED_TIMESTEP_HZ;
use bevy::prelude::*;
use core::time::Duration;

mod cli;
mod protocol;
mod shared;

#[cfg(feature = "client")]
mod client;
#[cfg(feature = "gui")]
mod renderer;
#[cfg(feature = "server")]
mod server;

fn main() {
    let cli = Cli::default();

    let mut app = cli.build_app(Duration::from_secs_f64(1.0 / FIXED_TIMESTEP_HZ), true);

    app.add_plugins(SharedPlugin);

    cli.spawn_connections(&mut app);

    match cli.mode {
        #[cfg(feature = "client")]
        Some(Mode::Client { .. }) => {
            app.add_plugins(BattleArenaClientPlugin);
        }
        #[cfg(feature = "server")]
        Some(Mode::Server) => {
            app.add_plugins(BattleArenaServerPlugin);
        }
        _ => {}
    }

    #[cfg(feature = "gui")]
    app.add_plugins(renderer::BattleArenaRendererPlugin);

    app.run();
}
