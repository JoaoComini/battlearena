use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use protocol::PlayerId;

use crate::lifetime::{EffectLifetime, OutroAnim};
use crate::sprite_anim::SpriteAnim;
use crate::util::pos2_to_vec3;

#[derive(Resource, Default)]
pub struct CharacterTrailState {
    frame: u32,
    foot: bool, // false = left, true = right
}

#[derive(Resource)]
pub struct FootstepAssets {
    pub anim: SpriteAnim,
    pub outro: SpriteAnim,
}

pub fn load_footstep_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // VFX5: 512×128 sheet, 4 cols × 1 row, each frame 128×128
    let vfx5 = asset_server.load("vfx/VFX5/Sprite-sheet/Sprite-sheet.png");
    // VFX3: 640×256 sheet, 5 cols × 2 rows, each frame 128×128 (5 frames total, last row is empty)
    let vfx3 = asset_server.load("vfx/VFX3/Sprite-sheet/Sprite-sheet.png");

    commands.insert_resource(FootstepAssets {
        anim: SpriteAnim::sheet(vfx5, 4, 1, 4, 0.08),
        outro: SpriteAnim::sheet(vfx3, 5, 1, 5, 0.07),
    });
}

pub fn spawn_character_trail(
    players: Query<(&Position, &LinearVelocity), With<PlayerId>>,
    mut state: ResMut<CharacterTrailState>,
    assets: Res<FootstepAssets>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    state.frame += 1;
    if state.frame % 16 != 0 {
        return;
    }
    state.foot = !state.foot;

    for (pos, vel) in &players {
        if vel.0.length() < 0.5 {
            continue;
        }

        let dir = vel.0.normalize();
        let behind = -dir * 0.3;
        let side = Vec2::new(-dir.y, dir.x) * if state.foot { 0.2 } else { -0.2 };
        let px = pos.x + behind.x + side.x;
        let py = pos.y + behind.y + side.y;

        let mut anim = assets.anim.clone();
        anim.looping = true;
        let mat = materials.add(anim.initial_material());

        commands.spawn((
            Mesh3d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial3d(mat),
            Transform::from_translation(pos2_to_vec3(px, py, 0.01))
                .with_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2))
                .with_scale(Vec3::splat(0.5)),
            anim,
            EffectLifetime::new(0.6),
            OutroAnim {
                anim: assets.outro.clone(),
                y_offset: 1.3,
                rotation: Some(Quat::from_rotation_x(-50_f32.to_radians())),
                // VFX3 frames are 128×256 (1:2 ratio), scale Y accordingly
                scale: Some(Vec3::new(1.0, 2.0, 1.0)),
            },
        ));
    }
}
