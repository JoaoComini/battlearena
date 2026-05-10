use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use protocol::PlayerId;

use crate::lifetime::EffectLifetime;
use crate::util::pos2_to_vec3;

#[derive(Resource, Default)]
pub struct CharacterTrailState {
    frame: u32,
}

pub fn spawn_character_trail(
    players: Query<(&Position, &LinearVelocity), With<PlayerId>>,
    mut state: ResMut<CharacterTrailState>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    state.frame += 1;
    if state.frame % 4 != 0 {
        return;
    }

    for (pos, vel) in &players {
        if vel.0.length() < 0.5 {
            continue;
        }

        // Offset slightly behind the movement direction
        let behind = -vel.0.normalize() * 0.3;
        let px = pos.x + behind.x;
        let py = pos.y + behind.y;

        commands.spawn((
            Mesh3d(meshes.add(Circle::new(0.18))),
            MeshMaterial3d(materials.add(StandardMaterial {
                base_color: Color::srgba(0.6, 0.5, 0.4, 0.5),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            })),
            Transform::from_translation(pos2_to_vec3(px, py, 0.01))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
            EffectLifetime::new(0.25),
        ));
    }
}
