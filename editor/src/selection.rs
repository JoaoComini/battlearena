use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use bevy::prelude::*;
use bevy_egui::input::EguiWantsInput;

/// The currently selected scene entity.
#[derive(Resource, Default)]
pub struct SelectedEntity(pub Option<Entity>);

pub struct SelectionPlugin;

impl Plugin for SelectionPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(WireframePlugin::default());
        app.init_resource::<SelectedEntity>();
        app.add_observer(on_click_select);
        app.add_systems(Update, update_wireframe);
    }
}

fn on_click_select(
    mut trigger: On<Pointer<Click>>,
    egui_wants_input: Res<EguiWantsInput>,
    mesh_query: Query<(), With<Mesh3d>>,
    mut selected: ResMut<SelectedEntity>,
) {
    if egui_wants_input.wants_any_pointer_input() {
        return;
    }

    if !mesh_query.contains(trigger.entity) {
        return;
    }

    trigger.propagate(false);
    selected.0 = Some(trigger.entity);
}

/// Adds/removes wireframe when selection changes.
/// If the selected entity has a `Mesh3d`, wireframe it directly.
/// Otherwise wireframe its mesh children.
fn update_wireframe(
    selected: Res<SelectedEntity>,
    children_query: Query<&Children>,
    mesh_query: Query<(), With<Mesh3d>>,
    mut commands: Commands,
    mut prev_selected: Local<Option<Entity>>,
) {
    if !selected.is_changed() {
        return;
    }

    if let Some(prev) = *prev_selected {
        for entity in mesh_entities(prev, &children_query, &mesh_query) {
            commands
                .entity(entity)
                .remove::<(Wireframe, WireframeColor)>();
        }
    }

    if let Some(entity) = selected.0 {
        for entity in mesh_entities(entity, &children_query, &mesh_query) {
            commands.entity(entity).insert((
                Wireframe,
                WireframeColor {
                    color: Color::srgb(1.0, 0.5, 0.0),
                },
            ));
        }
    }

    *prev_selected = selected.0;
}

fn mesh_entities(
    entity: Entity,
    children_query: &Query<&Children>,
    mesh_query: &Query<(), With<Mesh3d>>,
) -> Vec<Entity> {
    if mesh_query.contains(entity) {
        return vec![entity];
    }
    children_query
        .get(entity)
        .map(|c| c.iter().filter(|&e| mesh_query.contains(e)).collect())
        .unwrap_or_default()
}
