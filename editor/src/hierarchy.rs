use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use crate::selection::SelectedEntity;
use crate::spawn::Editable;

pub struct HierarchyPlugin;

impl Plugin for HierarchyPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, hierarchy_panel);
    }
}

fn hierarchy_panel(
    mut contexts: EguiContexts,
    entities: Query<(Entity, Option<&Name>, Option<&Children>), With<Editable>>,
    parents: Query<&ChildOf>,
    mut selected: ResMut<SelectedEntity>,
    mut commands: Commands,
) -> Result {
    egui::SidePanel::left("hierarchy")
        .min_width(200.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("Hierarchy");
            ui.separator();

            let mut roots: Vec<Entity> = entities
                .iter()
                .filter(|(e, _, _)| !parents.contains(*e))
                .map(|(e, _, _)| e)
                .collect();
            roots.sort_by_key(|e| e.index());

            let mut spawn_child_of: Option<Entity> = None;
            let mut spawn_root = false;

            for root in roots {
                entity_tree(root, &entities, ui, &mut selected, &mut spawn_child_of);
            }

            // Right-click on empty space below the list → add root entity.
            let remaining = ui.allocate_response(ui.available_size(), egui::Sense::click());
            remaining.context_menu(|ui| {
                if ui.button("Add Entity").clicked() {
                    spawn_root = true;
                    ui.close();
                }
            });

            if let Some(parent) = spawn_child_of {
                let child = commands
                    .spawn((Name::new("Entity"), Editable, Transform::default()))
                    .id();
                commands.entity(parent).add_child(child);
                selected.0 = Some(child);
            } else if spawn_root {
                let e = commands
                    .spawn((Name::new("Entity"), Editable, Transform::default()))
                    .id();
                selected.0 = Some(e);
            }
        });
    Ok(())
}

fn entity_tree(
    entity: Entity,
    entities: &Query<(Entity, Option<&Name>, Option<&Children>), With<Editable>>,
    ui: &mut egui::Ui,
    selected: &mut ResMut<SelectedEntity>,
    spawn_child_of: &mut Option<Entity>,
) {
    let Ok((_, name, children)) = entities.get(entity) else {
        return;
    };

    let label = name
        .map(|n| n.to_string())
        .unwrap_or_else(|| format!("Entity {}", entity.index()));

    let is_selected = selected.0 == Some(entity);

    let editable_children: Vec<Entity> = children
        .map(|c| c.iter().filter(|e| entities.contains(*e)).collect())
        .unwrap_or_default();

    if !editable_children.is_empty() {
        let id = ui.make_persistent_id(entity);
        egui::collapsing_header::CollapsingState::load_with_default_open(ui.ctx(), id, true)
            .show_header(ui, |ui: &mut egui::Ui| {
                let response = ui.selectable_label(is_selected, &label);
                if response.clicked() {
                    selected.0 = Some(entity);
                }
                context_menu(&response, entity, spawn_child_of);
            })
            .body(|ui| {
                for child in &editable_children {
                    entity_tree(*child, entities, ui, selected, spawn_child_of);
                }
            });
    } else {
        let response = ui.selectable_label(is_selected, &label);
        if response.clicked() {
            selected.0 = Some(entity);
        }
        context_menu(&response, entity, spawn_child_of);
    }
}

fn context_menu(response: &egui::Response, entity: Entity, spawn_child_of: &mut Option<Entity>) {
    response.context_menu(|ui| {
        if ui.button("Add Child Entity").clicked() {
            *spawn_child_of = Some(entity);
            ui.close();
        }
    });
}
