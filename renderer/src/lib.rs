#[cfg(feature = "client")]
pub mod client;
#[cfg(feature = "server")]
pub mod server;

use bevy::{color::palettes::css::BLUE, prelude::*};
use protocol::*;

pub struct BattleArenaRendererPlugin;

impl Plugin for BattleArenaRendererPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(physics::debug::PhysicsDebugRenderPlugin);
        app.add_plugins(abilities::debug::AbilityDebugPlugin);
        app.add_systems(Startup, init);
        app.add_systems(Update, draw_player_foward);
        app.add_systems(PostUpdate, follow_local_player);
    }
}

fn init(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 12.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        Projection::Perspective(PerspectiveProjection {
            fov: 60_f32.to_radians(),
            ..default()
        }),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(100.0, 600.0, 300.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn follow_local_player(
    local_player: Query<&Transform, (With<LocalPlayer>, Without<Camera3d>)>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Ok(transform) = local_player.single() else {
        return;
    };
    let Ok(mut camera_transform) = camera.single_mut() else {
        return;
    };

    let target = transform.translation;
    camera_transform.translation = Vec3::new(target.x, target.y + 12.0, target.z + 10.0);
    camera_transform.look_at(target, Vec3::Y);
}

fn draw_player_foward(player: Query<&Transform, With<LocalPlayer>>, mut gizmos: Gizmos) {
    let Ok(transform) = player.single() else {
        return;
    };

    gizmos.arrow(
        transform.translation + Vec3::Y,
        transform.translation + (Vec3::Y + *transform.forward()),
        BLUE,
    );
}
