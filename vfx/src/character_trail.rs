use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use protocol::PlayerId;

use crate::lifetime::EffectLifetime;
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
}

pub fn load_footstep_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // VFX5: 512×128 sheet, 4 columns × 1 row, each frame 128×128
    let image = asset_server.load("vfx/VFX5/Sprite-sheet/Sprite-sheet.png");
    commands.insert_resource(FootstepAssets {
        anim: SpriteAnim::sheet(image, 4, 1, 4, 0.08),
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
        // perpendicular to movement direction, alternating left/right
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
        ));
    }
}
