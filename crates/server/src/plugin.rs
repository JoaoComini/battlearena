use avian2d::prelude::*;
use bevy::prelude::*;
use shared::{
    map::MAP_OBSTACLES,
    physics::PlayerMovementPlugin,
    states::AppState,
};

use crate::{
    game::{net::NetPlugin, simulation::SimulationPlugin},
    resources::EntityRegistry,
};

fn spawn_obstacles(mut commands: Commands) {
    for (i, obs) in MAP_OBSTACLES.iter().enumerate() {
        commands.spawn((
            Name::new(format!("Obstacle_{i}")),
            RigidBody::Static,
            Position(Vec2::new(obs.center_x, obs.center_y)),
            Collider::rectangle(obs.half_x * 2.0, obs.half_y * 2.0),
        ));
    }
}

fn enter_in_game(mut next: ResMut<NextState<AppState>>) {
    next.set(AppState::InGame);
}

pub struct ServerGamePlugin;

impl Plugin for ServerGamePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PlayerMovementPlugin)
            .init_resource::<EntityRegistry>()
            .add_systems(Startup, (spawn_obstacles, enter_in_game).chain())
            .add_plugins((NetPlugin, SimulationPlugin));
    }
}
