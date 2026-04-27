use avian2d::prelude::{LinearVelocity, Rotation};
use bevy::scene::SceneInstanceReady;
use bevy::{animation::RepeatAnimation, asset::AssetPath, prelude::*};
use protocol::PlayerId;
use std::collections::HashMap;

use crate::CharacterVisual;

pub struct CharacterAnimationPlugin;

impl Plugin for CharacterAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_scene_ready);
        app.add_systems(Update, drive_animations);
    }
}

// ── Animation name enum ───────────────────────────────────────────────────────

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum AnimationName {
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

impl AnimationName {
    pub fn repeat(&self) -> RepeatAnimation {
        RepeatAnimation::Forever
    }
}

// ── Animation path table ──────────────────────────────────────────────────────

struct CharacterAnimationPaths {
    idle: AssetPath<'static>,
    run_forward: AssetPath<'static>,
    run_back: AssetPath<'static>,
    run_left: AssetPath<'static>,
    run_right: AssetPath<'static>,
    run_forward_left: AssetPath<'static>,
    run_forward_right: AssetPath<'static>,
    run_back_left: AssetPath<'static>,
    run_back_right: AssetPath<'static>,
}

impl CharacterAnimationPaths {
    fn new(glb: &str) -> Self {
        let anim = |i: usize| GltfAssetLabel::Animation(i).from_asset(glb.to_string());
        Self {
            idle: anim(0),
            run_forward: anim(4),
            run_back: anim(1),
            run_left: anim(7),
            run_right: anim(8),
            run_forward_left: anim(5),
            run_forward_right: anim(6),
            run_back_left: anim(2),
            run_back_right: anim(3),
        }
    }
}

// ── Animation map ─────────────────────────────────────────────────────────────

#[derive(Component)]
pub struct CharacterAnimations {
    nodes: HashMap<AnimationName, AnimationNodeIndex>,
}

impl CharacterAnimations {
    pub fn get(&self, name: AnimationName) -> AnimationNodeIndex {
        *self.nodes.get(&name).unwrap()
    }
}

// ── Setup observer ────────────────────────────────────────────────────────────

fn on_scene_ready(
    trigger: On<SceneInstanceReady>,
    visuals: Query<&CharacterVisual>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    children: Query<&Children>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
) {
    let root = trigger.entity;

    let Ok(visual) = visuals.get(root) else {
        return;
    };
    let glb = &visual.0;

    let Some(player_entity) = find_descendant(root, &children, &animation_players) else {
        warn!("No AnimationPlayer found in character scene");
        return;
    };

    let paths = CharacterAnimationPaths::new(glb);
    let mut graph = AnimationGraph::new();

    let graph_root = graph.root;
    let run_blend = graph.add_blend(1.0, graph_root);

    let mut nodes = HashMap::default();

    let mut add = |name: AnimationName, path: &AssetPath<'static>, parent| {
        let clip = asset_server.load(path.clone());
        let ix = graph.add_clip(clip, 1.0, parent);
        nodes.insert(name, ix);
    };

    add(AnimationName::Idle, &paths.idle, graph_root);
    add(AnimationName::RunForward, &paths.run_forward, run_blend);
    add(AnimationName::RunBack, &paths.run_back, run_blend);
    add(AnimationName::RunLeft, &paths.run_left, run_blend);
    add(AnimationName::RunRight, &paths.run_right, run_blend);
    add(
        AnimationName::RunForwardLeft,
        &paths.run_forward_left,
        run_blend,
    );
    add(
        AnimationName::RunForwardRight,
        &paths.run_forward_right,
        run_blend,
    );
    add(AnimationName::RunBackLeft, &paths.run_back_left, run_blend);
    add(
        AnimationName::RunBackRight,
        &paths.run_back_right,
        run_blend,
    );

    let graph_handle = graphs.add(graph);

    commands.entity(player_entity).insert((
        AnimationGraphHandle(graph_handle),
        CharacterAnimations { nodes },
    ));
}

// ── Drive system ──────────────────────────────────────────────────────────────

fn drive_animations(
    mut controllers: Query<(Entity, &mut AnimationPlayer, &CharacterAnimations)>,
    parents: Query<&ChildOf>,
    player_query: Query<(&LinearVelocity, &Rotation), With<PlayerId>>,
) {
    for (ctrl_entity, mut player, anims) in &mut controllers {
        let Some((velocity, rotation)) = find_ancestor(ctrl_entity, &parents, &player_query) else {
            continue;
        };

        let speed = velocity.0.length();

        if speed < 0.1 {
            play_idle(&mut player, anims);
        } else {
            play_directional(&mut player, anims, velocity, rotation);
        }
    }
}

fn play_idle(player: &mut AnimationPlayer, anims: &CharacterAnimations) {
    player
        .play(anims.get(AnimationName::Idle))
        .set_weight(1.0)
        .repeat();

    for name in directional_names() {
        player.play(anims.get(name)).set_weight(0.0).repeat();
    }
}

fn play_directional(
    player: &mut AnimationPlayer,
    anims: &CharacterAnimations,
    velocity: &LinearVelocity,
    rotation: &Rotation,
) {
    player
        .play(anims.get(AnimationName::Idle))
        .set_weight(0.0)
        .repeat();

    let facing = rotation.as_radians();
    let forward = Vec2::new(-facing.sin(), facing.cos());
    let right = Vec2::new(forward.y, -forward.x);

    let vel_dir = velocity.0.normalize_or_zero();
    let cos_a = vel_dir.dot(forward);
    let sin_a = vel_dir.dot(right);

    let k = std::f32::consts::FRAC_1_SQRT_2;
    let dirs: [(f32, f32); 8] = [
        (1.0, 0.0),
        (-1.0, 0.0),
        (0.0, -1.0),
        (0.0, 1.0),
        (k, -k),
        (k, k),
        (-k, -k),
        (-k, k),
    ];

    let mut weights = [0.0f32; 8];
    for (i, (dc, ds)) in dirs.iter().enumerate() {
        weights[i] = (cos_a * dc + sin_a * ds).max(0.0);
    }
    let sum: f32 = weights.iter().sum();
    if sum > 0.0 {
        for w in &mut weights {
            *w /= sum;
        }
    }

    for (name, &w) in directional_names().iter().zip(weights.iter()) {
        player.play(anims.get(*name)).set_weight(w).repeat();
    }
}

fn directional_names() -> [AnimationName; 8] {
    [
        AnimationName::RunForward,
        AnimationName::RunBack,
        AnimationName::RunLeft,
        AnimationName::RunRight,
        AnimationName::RunForwardLeft,
        AnimationName::RunForwardRight,
        AnimationName::RunBackLeft,
        AnimationName::RunBackRight,
    ]
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
