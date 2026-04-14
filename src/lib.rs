// Check feature combo sanity, to make it easier to cfg out code by just checking one feature.
// For example, don't allow "steam" to be set on wasm builds, because it can't work in wasm anyway.

#[cfg(all(feature = "steam", target_family = "wasm"))]
compile_error!("steam feature is not supported in wasm");

#[cfg(all(feature = "server", target_family = "wasm"))]
compile_error!("server feature is not supported in wasm");

pub mod protocol;

#[cfg(feature = "client")]
pub mod client;

#[cfg(feature = "server")]
pub mod server;

#[cfg(feature = "gui")]
pub mod renderer;

pub mod shared;

// Harness modules moved from common crate
pub mod cli;

#[cfg(feature = "client")]
pub mod client_setup;

#[cfg(feature = "server")]
pub mod server_setup;

#[cfg(all(any(feature = "gui2d", feature = "gui3d"), feature = "client"))]
pub mod client_renderer;

#[cfg(all(any(feature = "gui2d", feature = "gui3d"), feature = "server"))]
pub mod server_renderer;
