use avian2d::prelude::{LinearVelocity, Position};
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use protocol::PlayerId;

use crate::util::pos2_to_vec3;

#[derive(Resource, Default)]
pub struct CharacterTrailState {
    frame: u32,
    foot: bool, // false = left, true = right
}

#[derive(Resource)]
pub struct FootstepAssets {
    pub scene: Handle<Scene>,
    pub graph: Handle<AnimationGraph>,
    pub index: AnimationNodeIndex,
}

/// Marks a spawned footprint scene so we can find its AnimationPlayer
/// and despawn it when the animation finishes.
#[derive(Component)]
pub struct FootprintScene;

/// Placed on the AnimationPlayer entity inside the footprint scene.
#[derive(Component)]
pub struct FootprintPlayer {
    pub root: Entity,
    pub index: AnimationNodeIndex,
}

pub fn load_footstep_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
) {
    let clip: Handle<AnimationClip> =
        asset_server.load(GltfAssetLabel::Animation(0).from_asset("vfx/lightning_footprint.glb"));
    let mut graph = AnimationGraph::new();
    let index = graph.add_clip(clip, 1.0, graph.root);
    commands.insert_resource(FootstepAssets {
        scene: asset_server.load(GltfAssetLabel::Scene(0).from_asset("vfx/lightning_footprint.glb")),
        graph: graphs.add(graph),
        index,
    });
}

pub fn spawn_character_trail(
    players: Query<(&Position, &LinearVelocity), With<PlayerId>>,
    mut state: ResMut<CharacterTrailState>,
    assets: Res<FootstepAssets>,
    mut commands: Commands,
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

        commands.spawn((
            SceneRoot(assets.scene.clone()),
            Transform::from_translation(pos2_to_vec3(px, py, 0.0)),
            FootprintScene,
        ));
    }
}

pub fn on_footprint_ready(
    trigger: On<SceneInstanceReady>,
    footprints: Query<&FootprintScene>,
    children: Query<&Children>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
    assets: Res<FootstepAssets>,
    mut commands: Commands,
) {
    let root = trigger.entity;
    if footprints.get(root).is_err() {
        return;
    }

    let Some(player_entity) = find_descendant(root, &children, &animation_players) else {
        return;
    };

    commands.entity(player_entity).insert((
        AnimationGraphHandle(assets.graph.clone()),
        FootprintPlayer { root, index: assets.index },
    ));
}

pub fn tick_footprint_players(
    mut query: Query<(Entity, &mut AnimationPlayer, &FootprintPlayer)>,
    mut commands: Commands,
) {
    for (_entity, mut player, fp) in &mut query {
        let active = player.play(fp.index);
        if active.is_finished() {
            commands.entity(fp.root).despawn();
        }
    }
}

fn find_descendant<T: Component>(
    root: Entity,
    children_query: &Query<&Children>,
    target_query: &Query<Entity, With<T>>,
) -> Option<Entity> {
    if target_query.get(root).is_ok() {
        return Some(root);
    }
    let Ok(children) = children_query.get(root) else {
        return None;
    };
    for child in children.iter() {
        if let Some(found) = find_descendant(child, children_query, target_query) {
            return Some(found);
        }
    }
    None
}
