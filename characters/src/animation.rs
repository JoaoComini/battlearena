use avian2d::prelude::{LinearVelocity, Rotation};
use bevy::scene::SceneInstanceReady;
use bevy::{asset::AssetPath, prelude::*};
use protocol::PlayerId;
use std::collections::HashMap;

use crate::types::{Character, CharacterDef};

pub struct CharacterAnimationPlugin;

impl Plugin for CharacterAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_scene_ready);
        app.add_systems(Update, tick_animation_graph);
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
enum AnimationName {
    Idle,
    RunForward,
    RunBack,
    RunLeft,
    RunRight,
    RunForwardLeft,
    RunForwardRight,
    RunBackLeft,
    RunBackRight,
}

#[derive(Component)]
pub struct CharacterAnimationState {
    anims: HashMap<AnimationName, AnimationNodeIndex>,
}

impl CharacterAnimationState {
    fn get(&self, name: AnimationName) -> AnimationNodeIndex {
        *self.anims.get(&name).unwrap()
    }
}

// ── Setup ─────────────────────────────────────────────────────────────────────

struct Paths {
    idle: AssetPath<'static>,
    run_f: AssetPath<'static>,
    run_b: AssetPath<'static>,
    run_l: AssetPath<'static>,
    run_r: AssetPath<'static>,
    run_fl: AssetPath<'static>,
    run_fr: AssetPath<'static>,
    run_bl: AssetPath<'static>,
    run_br: AssetPath<'static>,
}

impl Paths {
    fn new(glb: &str) -> Self {
        // 0:Idle 1:Running_B 2:Running_BL 3:Running_BR
        // 4:Running_F 5:Running_FL 6:Running_FR 7:Running_L 8:Running_R
        let a = |i: usize| GltfAssetLabel::Animation(i).from_asset(glb.to_string());
        Self {
            idle: a(0),
            run_f: a(4),
            run_b: a(1),
            run_l: a(7),
            run_r: a(8),
            run_fl: a(5),
            run_fr: a(6),
            run_bl: a(2),
            run_br: a(3),
        }
    }
}

fn on_scene_ready(
    trigger: On<SceneInstanceReady>,
    characters: Query<&Character>,
    char_assets: Res<Assets<CharacterDef>>,
    asset_server: Res<AssetServer>,
    mut commands: Commands,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    children: Query<&Children>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
) {
    let root = trigger.entity;
    let Ok(character) = characters.get(root) else { return };
    let Some(def) = char_assets.get(&character.0) else { return };
    let Some(ref visual_handle) = def.visual else { return };
    let Some(asset_path) = asset_server.get_path(visual_handle) else { return };
    let glb = asset_path.path().to_string_lossy().into_owned();

    let Some(player_entity) = find_descendant(root, &children, &animation_players) else {
        warn!("No AnimationPlayer found in character scene");
        return;
    };

    let p = Paths::new(&glb);
    let mut graph = AnimationGraph::new();
    let r = graph.root;
    let mut map = HashMap::default();

    let mut add = |name, path: &AssetPath<'static>| {
        map.insert(
            name,
            graph.add_clip(asset_server.load(path.clone()), 1.0, r),
        );
    };

    add(AnimationName::Idle, &p.idle);
    add(AnimationName::RunForward, &p.run_f);
    add(AnimationName::RunBack, &p.run_b);
    add(AnimationName::RunLeft, &p.run_l);
    add(AnimationName::RunRight, &p.run_r);
    add(AnimationName::RunForwardLeft, &p.run_fl);
    add(AnimationName::RunForwardRight, &p.run_fr);
    add(AnimationName::RunBackLeft, &p.run_bl);
    add(AnimationName::RunBackRight, &p.run_br);

    commands.entity(player_entity).insert((
        AnimationGraphHandle(graphs.add(graph)),
        CharacterAnimationState { anims: map },
    ));
}

// ── Tick ──────────────────────────────────────────────────────────────────────

// (forward, right) directions for each animation. Idle is at the origin (0, 0).
const DIR_ANIMS: [(f32, f32, AnimationName); 9] = [
    (0.0, 0.0, AnimationName::Idle),
    (1.0, 0.0, AnimationName::RunForward),
    (-1.0, 0.0, AnimationName::RunBack),
    (0.0, -1.0, AnimationName::RunLeft),
    (0.0, 1.0, AnimationName::RunRight),
    (1.0, -1.0, AnimationName::RunForwardLeft),
    (1.0, 1.0, AnimationName::RunForwardRight),
    (-1.0, -1.0, AnimationName::RunBackLeft),
    (-1.0, 1.0, AnimationName::RunBackRight),
];

fn tick_animation_graph(
    mut query: Query<(Entity, &mut AnimationPlayer, &CharacterAnimationState)>,
    parents: Query<&ChildOf>,
    player_query: Query<(&LinearVelocity, &Rotation), With<PlayerId>>,
) {
    for (entity, mut player, state) in &mut query {
        let Some((velocity, rotation)) = find_ancestor(entity, &parents, &player_query) else {
            continue;
        };

        let speed = velocity.0.length();

        // Project velocity into player-local forward/right axes.
        let facing = rotation.as_radians();
        let fwd = Vec2::new(-facing.sin(), facing.cos());
        let rgt = Vec2::new(fwd.y, -fwd.x);
        let vel = velocity.0;
        let local = Vec2::new(vel.dot(fwd), vel.dot(rgt));

        // Idle gets weight inversely proportional to speed; directional clips
        // get dot-product weights. All are computed uniformly then normalized.
        let mut weights = [0.0f32; 9];
        for (i, &(fwd_d, rgt_d, _)) in DIR_ANIMS.iter().enumerate() {
            let dir = Vec2::new(fwd_d, rgt_d);
            weights[i] = if dir == Vec2::ZERO {
                // Idle: full weight when still, fades as speed increases.
                (1.0 - speed / 5.0).max(0.0)
            } else {
                local.dot(dir.normalize()).max(0.0)
            };
        }
        let sum: f32 = weights.iter().sum();
        if sum > 0.0 {
            for w in &mut weights {
                *w /= sum;
            }
        }

        for (i, &(_, _, name)) in DIR_ANIMS.iter().enumerate() {
            player.play(state.get(name)).set_weight(weights[i]).repeat();
        }
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────────

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

fn find_ancestor<'w, T: Component>(
    entity: Entity,
    parents: &Query<&ChildOf>,
    target_query: &'w Query<(&LinearVelocity, &Rotation), With<T>>,
) -> Option<(&'w LinearVelocity, &'w Rotation)> {
    let mut current = entity;
    loop {
        if let Ok(result) = target_query.get(current) {
            return Some(result);
        }
        match parents.get(current) {
            Ok(ChildOf(parent)) => current = *parent,
            Err(_) => return None,
        }
    }
}
