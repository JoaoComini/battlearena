use avian2d::prelude::{LinearVelocity, Rotation};
use bevy::prelude::*;
use bevy::scene::SceneInstanceReady;
use protocol::PlayerId;
use scene::SceneSourcePath;

pub struct CharacterAnimationPlugin;

impl Plugin for CharacterAnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_observer(on_scene_ready);
        app.add_systems(Update, drive_animations);
    }
}

#[derive(Component)]
pub struct CharacterAnimationController {
    idle_node: AnimationNodeIndex,
    forward_node: AnimationNodeIndex,
    back_node: AnimationNodeIndex,
    left_node: AnimationNodeIndex,
    right_node: AnimationNodeIndex,
    forward_left_node: AnimationNodeIndex,
    forward_right_node: AnimationNodeIndex,
    back_left_node: AnimationNodeIndex,
    back_right_node: AnimationNodeIndex,
}

fn on_scene_ready(
    trigger: On<SceneInstanceReady>,
    scene_roots: Query<&SceneSourcePath>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    children: Query<&Children>,
    animation_players: Query<Entity, With<AnimationPlayer>>,
) {
    let root = trigger.entity;

    let Ok(source) = scene_roots.get(root) else {
        return;
    };
    if !source.0.contains("character") {
        return;
    }

    let Some(player_entity) = find_descendant(root, &children, &animation_players) else {
        warn!("No AnimationPlayer found in character scene");
        return;
    };

    let glb = "assets/models/character.glb";

    let mut graph = AnimationGraph::new();

    let idle_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation0")), // Idle
        1.0,
        graph.root,
    );

    // All 8 directional clips share one blend parent so their weights are
    // automatically normalized relative to each other.
    let run_blend_node = graph.add_blend(0.0, graph.root);

    let forward_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation4")), // Running_F
        1.0,
        run_blend_node,
    );
    let back_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation1")), // Running_B
        1.0,
        run_blend_node,
    );
    let left_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation7")), // Running_L
        1.0,
        run_blend_node,
    );
    let right_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation8")), // Running_R
        1.0,
        run_blend_node,
    );
    let forward_left_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation5")), // Running_FL
        1.0,
        run_blend_node,
    );
    let forward_right_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation6")), // Running_FR
        1.0,
        run_blend_node,
    );
    let back_left_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation2")), // Running_BL
        1.0,
        run_blend_node,
    );
    let back_right_node = graph.add_clip(
        asset_server.load(format!("{glb}#Animation3")), // Running_BR
        1.0,
        run_blend_node,
    );

    let graph_handle = graphs.add(graph);

    commands.entity(player_entity).insert((
        AnimationGraphHandle(graph_handle),
        CharacterAnimationController {
            idle_node,
            forward_node,
            back_node,
            left_node,
            right_node,
            forward_left_node,
            forward_right_node,
            back_left_node,
            back_right_node,
        },
    ));
}

fn drive_animations(
    mut controllers: Query<(Entity, &mut AnimationPlayer, &CharacterAnimationController)>,
    parents: Query<&ChildOf>,
    player_query: Query<(&LinearVelocity, &Rotation), With<PlayerId>>,
) {
    for (ctrl_entity, mut player, ctrl) in &mut controllers {
        let Some((velocity, rotation)) = find_ancestor(ctrl_entity, &parents, &player_query) else {
            continue;
        };

        let speed = velocity.0.length();

        if speed < 0.1 {
            play_idle(&mut player, ctrl);
        } else {
            play_directional(&mut player, ctrl, velocity, rotation);
        }
    }
}

fn play_idle(player: &mut AnimationPlayer, ctrl: &CharacterAnimationController) {
    player.play(ctrl.idle_node).set_weight(1.0).repeat();

    for &node in directional_nodes(ctrl).iter() {
        player.play(node).set_weight(0.0).repeat();
    }
}

fn play_directional(
    player: &mut AnimationPlayer,
    ctrl: &CharacterAnimationController,
    velocity: &LinearVelocity,
    rotation: &Rotation,
) {
    player.play(ctrl.idle_node).set_weight(0.0).repeat();

    // The facing angle has - π/2 baked in (mouse above player → angle 0,
    // model faces screen-up = 2D +Y). Reconstruct the facing unit vector
    // directly from the angle to avoid reference-frame mismatches.
    // In 2D physics space: +X = right, +Y = up/forward on screen.
    // from_rotation_y(angle) rotates in XZ, mapping 2D +Y → 3D -Z.
    // The 2D facing direction corresponding to a given rotation angle is:
    //   forward_2d = (sin(angle), cos(angle))  [because angle=0 → facing +Y]
    let facing = rotation.as_radians();
    let forward = Vec2::new(-facing.sin(), facing.cos()); // points toward mouse
    let right = Vec2::new(forward.y, -forward.x); // 90° CW from forward

    let vel_dir = velocity.0.normalize_or_zero();
    let cos_a = vel_dir.dot(forward); // > 0 = moving toward mouse
    let sin_a = vel_dir.dot(right); // > 0 = moving right relative to facing
                                    // cos_a > 0 = forward, sin_a > 0 = right (in standard math convention)

    // Unit vectors for each of the 8 directions in (cos, sin) = (fwd, right) space
    let k = std::f32::consts::FRAC_1_SQRT_2;
    let dirs: [(f32, f32); 8] = [
        (1.0, 0.0),  // Forward
        (-1.0, 0.0), // Back
        (0.0, -1.0), // Left
        (0.0, 1.0),  // Right
        (k, -k),     // Forward-Left
        (k, k),      // Forward-Right
        (-k, -k),    // Back-Left
        (-k, k),     // Back-Right
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

    for (&node, &w) in directional_nodes(ctrl).iter().zip(weights.iter()) {
        player.play(node).set_weight(w).repeat();
    }
}

fn directional_nodes(ctrl: &CharacterAnimationController) -> [AnimationNodeIndex; 8] {
    [
        ctrl.forward_node,
        ctrl.back_node,
        ctrl.left_node,
        ctrl.right_node,
        ctrl.forward_left_node,
        ctrl.forward_right_node,
        ctrl.back_left_node,
        ctrl.back_right_node,
    ]
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
