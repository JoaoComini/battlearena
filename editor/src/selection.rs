use bevy::pbr::wireframe::{Wireframe, WireframeColor, WireframePlugin};
use bevy::prelude::*;

use crate::spawn::Editable;

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

/// Selects the clicked entity if it is `Editable`, otherwise walks up to the
/// closest `Editable` ancestor.
fn on_click_select(
    trigger: On<Pointer<Click>>,
    editable_query: Query<(), With<Editable>>,
    parent_query: Query<&ChildOf>,
    mut selected: ResMut<SelectedEntity>,
) {
    let mut current = trigger.entity;
    loop {
        if editable_query.contains(current) {
            selected.0 = Some(current);
            return;
        }
        match parent_query.get(current) {
            Ok(child_of) => current = child_of.parent(),
            Err(_) => break,
        }
    }
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
            commands.entity(entity).remove::<(Wireframe, WireframeColor)>();
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
