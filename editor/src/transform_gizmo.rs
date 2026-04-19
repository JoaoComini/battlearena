use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_egui::input::EguiWantsInput;

use crate::free_camera::FreeCamera;
use crate::selection::SelectedEntity;

const GIZMO_SCALE: f32 = 0.6;
const HIT_RADIUS: f32 = 0.15;

#[derive(Resource, Default, PartialEq, Clone, Copy)]
pub enum GizmoMode {
    #[default]
    Translate,
    Rotate,
}

#[derive(Resource, Default)]
pub struct GizmoDragState {
    axis: Option<Vec3>,
    last_cursor: Option<Vec2>,
}

pub struct TransformGizmoPlugin;

impl Plugin for TransformGizmoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GizmoMode>();
        app.init_resource::<GizmoDragState>();
        app.add_systems(Update, (switch_mode, draw_gizmo, handle_drag).chain());
    }
}

fn switch_mode(keyboard: Res<ButtonInput<KeyCode>>, mut mode: ResMut<GizmoMode>) {
    if keyboard.just_pressed(KeyCode::KeyT) {
        *mode = GizmoMode::Translate;
    }
    if keyboard.just_pressed(KeyCode::KeyR) {
        *mode = GizmoMode::Rotate;
    }
}

struct GizmoAxis {
    dir: Vec3,
    color: Color,
}

impl GizmoAxis {
    const fn new(dir: Vec3, color: Color) -> Self {
        Self { dir, color }
    }

    fn display_color(&self, active: Option<Vec3>) -> Color {
        if active == Some(self.dir) {
            Color::srgb(1.0, 1.0, 0.0)
        } else {
            self.color
        }
    }
}

const AXES: [GizmoAxis; 3] = [
    GizmoAxis::new(Vec3::X, Color::srgb(1.0, 0.2, 0.2)),
    GizmoAxis::new(Vec3::Y, Color::srgb(0.2, 1.0, 0.2)),
    GizmoAxis::new(Vec3::Z, Color::srgb(0.2, 0.4, 1.0)),
];

fn draw_gizmo(
    selected: Res<SelectedEntity>,
    transform_query: Query<&GlobalTransform>,
    mode: Res<GizmoMode>,
    drag_state: Res<GizmoDragState>,
    camera_query: Query<(&Camera, &GlobalTransform), With<FreeCamera>>,
    mut gizmos: Gizmos,
) {
    let Some(entity) = selected.0 else { return };
    let Ok(gt) = transform_query.get(entity) else { return };
    let Ok((_camera, cam_gt)) = camera_query.single() else { return };

    let origin = gt.translation();
    let dist = (origin - cam_gt.translation()).length().max(0.1);
    let scale = dist * GIZMO_SCALE * 0.1;

    match *mode {
        GizmoMode::Translate => {
            for ax in &AXES {
                let color = ax.display_color(drag_state.axis);
                gizmos.arrow(origin, origin + ax.dir * scale, color);
            }
        }
        GizmoMode::Rotate => {
            for ax in &AXES {
                let color = ax.display_color(drag_state.axis);
                let iso = Isometry3d::new(origin, Quat::from_rotation_arc(Vec3::Z, ax.dir));
                gizmos.circle(iso, scale, color);
            }
        }
    }
}

fn handle_drag(
    selected: Res<SelectedEntity>,
    transform_query: Query<&GlobalTransform>,
    mut local_transform_query: Query<&mut Transform>,
    mode: Res<GizmoMode>,
    mut drag_state: ResMut<GizmoDragState>,
    mouse_button: Res<ButtonInput<MouseButton>>,
    egui_wants_input: Res<EguiWantsInput>,
    camera_query: Query<(&Camera, &GlobalTransform), With<FreeCamera>>,
    windows: Query<&Window, With<PrimaryWindow>>,
) {
    if egui_wants_input.wants_any_pointer_input() {
        drag_state.axis = None;
        drag_state.last_cursor = None;
        return;
    }

    let Some(entity) = selected.0 else { return };
    let Ok(gt) = transform_query.get(entity) else { return };
    let Ok(mut transform) = local_transform_query.get_mut(entity) else { return };
    let Ok((camera, cam_gt)) = camera_query.single() else { return };
    let Ok(window) = windows.single() else { return };
    let Some(cursor) = window.cursor_position() else { return };

    let origin = gt.translation();
    let dist = (origin - cam_gt.translation()).length().max(0.1);
    let scale = dist * GIZMO_SCALE * 0.1;

    if mouse_button.just_pressed(MouseButton::Left) {
        let ray = camera.viewport_to_world(cam_gt, cursor).unwrap_or(Ray3d { origin: Vec3::ZERO, direction: Dir3::Z });
        drag_state.axis = pick_axis(ray, origin, scale, &mode);
        drag_state.last_cursor = drag_state.axis.map(|_| cursor);
        return;
    }

    if mouse_button.just_released(MouseButton::Left) {
        drag_state.axis = None;
        drag_state.last_cursor = None;
        return;
    }

    if !mouse_button.pressed(MouseButton::Left) {
        return;
    }

    let Some(axis) = drag_state.axis else { return };
    let Some(last) = drag_state.last_cursor else { return };

    let delta_px = cursor - last;
    drag_state.last_cursor = Some(cursor);

    match *mode {
        GizmoMode::Translate => apply_translation(&mut transform, axis, delta_px, origin, camera, cam_gt, scale),
        GizmoMode::Rotate => apply_rotation(&mut transform, axis, delta_px, cam_gt),
    }
}

fn pick_axis(ray: Ray3d, origin: Vec3, scale: f32, mode: &GizmoMode) -> Option<Vec3> {
    match mode {
        GizmoMode::Translate => {
            AXES.iter().filter_map(|ax| {
                let d = ray_segment_dist(ray, origin, origin + ax.dir * scale);
                (d < HIT_RADIUS * scale).then_some((ax.dir, d))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(dir, _)| dir)
        }
        GizmoMode::Rotate => {
            AXES.iter().filter_map(|ax| {
                let pt = ray.plane_intersection_point(origin, InfinitePlane3d::new(ax.dir))?;
                let diff = ((pt - origin).length() - scale).abs();
                (diff < HIT_RADIUS * scale).then_some((ax.dir, diff))
            })
            .min_by(|a, b| a.1.partial_cmp(&b.1).unwrap())
            .map(|(dir, _)| dir)
        }
    }
}

fn apply_translation(
    transform: &mut Transform,
    axis: Vec3,
    delta_px: Vec2,
    origin: Vec3,
    camera: &Camera,
    cam_gt: &GlobalTransform,
    scale: f32,
) {
    let sp0 = camera.world_to_viewport(cam_gt, origin);
    let sp1 = camera.world_to_viewport(cam_gt, origin + axis * scale);

    let (Ok(sp0), Ok(sp1)) = (sp0, sp1) else { return };

    let screen_axis = sp1 - sp0;
    let screen_len = screen_axis.length();
    if screen_len < 0.001 { return; }

    let dot = delta_px.dot(screen_axis / screen_len);
    transform.translation += axis * dot * (scale / screen_len);
}

fn apply_rotation(transform: &mut Transform, axis: Vec3, delta_px: Vec2, cam_gt: &GlobalTransform) {
    let sign = if cam_gt.forward().dot(axis) > 0.0 { -1.0 } else { 1.0 };
    let angle = (delta_px.x + delta_px.y) * 0.01 * sign;
    transform.rotation = Quat::from_axis_angle(axis, angle) * transform.rotation;
}

fn ray_segment_dist(ray: Ray3d, a: Vec3, b: Vec3) -> f32 {
    let ray_dir: Vec3 = *ray.direction;
    let seg_dir = b - a;
    let w0 = ray.origin - a;

    let b_coef = ray_dir.dot(seg_dir);
    let c_coef = seg_dir.dot(seg_dir);
    let d_coef = ray_dir.dot(w0);
    let e_coef = seg_dir.dot(w0);

    let denom = c_coef - b_coef * b_coef;

    let (t, s) = if denom.abs() < 1e-6 {
        (d_coef, 0.0_f32)
    } else {
        let t = (b_coef * e_coef - c_coef * d_coef) / denom;
        let s = ((e_coef - b_coef * d_coef) / denom).clamp(0.0, 1.0);
        (t, s)
    };

    (ray.get_point(t.max(0.0)) - (a + seg_dir * s)).length()
}
