use bevy::prelude::*;
use crate::spawn::ActiveSceneRoot;
use bevy_egui::{egui, EguiContexts};

use crate::selection::SelectedEntity;

pub(crate) fn hierarchy_panel(
    mut contexts: EguiContexts,
    root_query: Query<(Entity, &Children), With<ActiveSceneRoot>>,
    entities: Query<(Entity, Option<&Name>, Option<&Children>)>,
    mut selected: ResMut<SelectedEntity>,
    mut commands: Commands,
) -> Result {
    egui::SidePanel::left("hierarchy")
        .min_width(200.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("Hierarchy");
            ui.separator();

            let Ok((root, root_children)) = root_query.single() else {
                return;
            };

            let mut roots: Vec<Entity> = root_children.iter().collect::<Vec<_>>();
            roots.sort_by_key(|e| e.index());

            let mut spawn_child_of: Option<Entity> = None;
            let mut spawn_root = false;

            for entity in roots {
                entity_tree(entity, &entities, ui, &mut selected, &mut spawn_child_of);
            }

            let remaining = ui.allocate_response(ui.available_size(), egui::Sense::click());
            remaining.context_menu(|ui| {
                if ui.button("Add Entity").clicked() {
                    spawn_root = true;
                    ui.close();
                }
            });

            if let Some(parent) = spawn_child_of {
                let child = commands
                    .spawn((Name::new("Entity"), Transform::default()))
                    .id();
                commands.entity(parent).add_child(child);
                selected.0 = Some(child);
            } else if spawn_root {
                let child = commands
                    .spawn((Name::new("Entity"), Transform::default()))
                    .id();
                commands.entity(root).add_child(child);
                selected.0 = Some(child);
            }
        });
    Ok(())
}

fn entity_tree(
    entity: Entity,
    entities: &Query<(Entity, Option<&Name>, Option<&Children>)>,
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

    let child_list: Vec<Entity> = children
        .map(|c| c.iter().collect())
        .unwrap_or_default();

    if !child_list.is_empty() {
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
                for child in &child_list {
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
