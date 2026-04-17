use avian2d::prelude::{ColliderConstructor, RigidBody};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use scene::{save_scene, GltfPrimitiveRef};

use crate::selection::SelectedEntity;
use crate::spawn::{asset_fs_path, Editable, OpenGltf};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(EguiPrimaryContextPass, inspector_panel);
    }
}

fn inspector_panel(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    node_query: Query<(Option<&GltfPrimitiveRef>, Option<&Name>)>,
    mesh_query: Query<(&Mesh3d, Option<&MeshMaterial3d<StandardMaterial>>)>,
    mut collider_query: Query<Option<&mut ColliderConstructor>>,
    rigid_body_query: Query<Option<&RigidBody>>,
    mut commands: Commands,
) -> Result {
    egui::SidePanel::right("inspector")
        .min_width(240.0)
        .show(contexts.ctx_mut()?, |ui| {
            ui.heading("Inspector");
            ui.separator();

            let Some(entity) = selected.0 else {
                ui.label("Click a mesh to select a node.");
                ui.separator();
                if ui.button("Save Scene").clicked() {
                    commands.run_system_cached(save_scene_system);
                }
                return;
            };

            // Entity info.
            if let Ok((prim_ref, name)) = node_query.get(entity) {
                if let Some(n) = name {
                    ui.label(format!("Name:  {}", n));
                }
                if let Some(r) = prim_ref {
                    ui.label(format!("Mesh:      {}", r.mesh_index));
                    ui.label(format!("Primitive: {}", r.primitive_index));
                    ui.label(format!("File:      {}", r.path));
                }
            }
            ui.separator();

            // Mesh info (only shown when a primitive entity is selected).
            if let Ok((mesh, material)) = mesh_query.get(entity) {
                egui::CollapsingHeader::new("Mesh")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("Mesh:     {:?}", mesh.id()));
                        ui.label(format!("Material: {}", if material.is_some() { "present" } else { "none" }));
                    });
                ui.separator();
            }

            // ColliderConstructor component editor.
            if let Ok(Some(mut collider)) = collider_query.get_mut(entity) {
                let mut remove = false;

                egui::CollapsingHeader::new("ColliderConstructor")
                    .default_open(true)
                    .show(ui, |ui| {
                        collider_editor(ui, &mut collider);
                        if ui.small_button("Remove").clicked() {
                            remove = true;
                        }
                    });

                if remove {
                    commands.entity(entity).remove::<ColliderConstructor>();
                }

                ui.separator();
            }

            // RigidBody component editor.
            if let Ok(Some(rigid_body)) = rigid_body_query.get(entity) {
                let mut current = *rigid_body;
                let mut remove = false;

                egui::CollapsingHeader::new("RigidBody")
                    .default_open(true)
                    .show(ui, |ui| {
                        rigid_body_editor(ui, &mut current);
                        if ui.small_button("Remove").clicked() {
                            remove = true;
                        }
                    });

                if remove {
                    commands.entity(entity).remove::<RigidBody>();
                } else if current != *rigid_body {
                    commands.entity(entity).insert(current);
                }

                ui.separator();
            }

            // Add Component button.
            let has_collider = collider_query
                .get(entity)
                .map(|c| c.is_some())
                .unwrap_or(false);
            let has_rigid_body = rigid_body_query
                .get(entity)
                .map(|r| r.is_some())
                .unwrap_or(false);

            ui.menu_button("+ Add Component", |ui| {
                if !has_collider {
                    if ui.button("ColliderConstructor").clicked() {
                        commands
                            .entity(entity)
                            .insert(ColliderConstructor::Circle { radius: 1.0 });
                        ui.close();
                    }
                }
                if !has_rigid_body {
                    if ui.button("RigidBody").clicked() {
                        commands.entity(entity).insert(RigidBody::Static);
                        ui.close();
                    }
                }
            });

            ui.separator();
            if ui.button("Save Scene").clicked() {
                commands.run_system_cached(save_scene_system);
            }
        });
    Ok(())
}

/// Edits a `ColliderConstructor` in-place, bound directly to the component.
fn collider_editor(ui: &mut egui::Ui, collider: &mut ColliderConstructor) {
    // Variant selector.
    let mut selected = match collider {
        ColliderConstructor::Circle { .. } => 0_usize,
        ColliderConstructor::Rectangle { .. } => 1,
        _ => 0,
    };

    egui::ComboBox::from_label("Type")
        .selected_text(match selected {
            0 => "Circle",
            _ => "Rectangle",
        })
        .show_ui(ui, |ui| {
            if ui.selectable_value(&mut selected, 0, "Circle").clicked()
                && !matches!(collider, ColliderConstructor::Circle { .. })
            {
                *collider = ColliderConstructor::Circle { radius: 1.0 };
            }
            if ui.selectable_value(&mut selected, 1, "Rectangle").clicked()
                && !matches!(collider, ColliderConstructor::Rectangle { .. })
            {
                *collider = ColliderConstructor::Rectangle {
                    x_length: 1.0,
                    y_length: 1.0,
                };
            }
        });

    // Fields bound directly to the component.
    match collider {
        ColliderConstructor::Circle { radius } => {
            ui.horizontal(|ui| {
                ui.label("Radius");
                ui.add(egui::DragValue::new(radius).speed(0.1).range(0.01..=f32::MAX));
            });
        }
        ColliderConstructor::Rectangle { x_length, y_length } => {
            ui.horizontal(|ui| {
                ui.label("X length");
                ui.add(
                    egui::DragValue::new(x_length)
                        .speed(0.1)
                        .range(0.01..=f32::MAX),
                );
            });
            ui.horizontal(|ui| {
                ui.label("Y length");
                ui.add(
                    egui::DragValue::new(y_length)
                        .speed(0.1)
                        .range(0.01..=f32::MAX),
                );
            });
        }
        _ => {}
    }
}

fn rigid_body_editor(ui: &mut egui::Ui, rigid_body: &mut RigidBody) {
    egui::ComboBox::from_label("Type")
        .selected_text(match rigid_body {
            RigidBody::Dynamic => "Dynamic",
            RigidBody::Static => "Static",
            RigidBody::Kinematic => "Kinematic",
        })
        .show_ui(ui, |ui| {
            ui.selectable_value(rigid_body, RigidBody::Dynamic, "Dynamic");
            ui.selectable_value(rigid_body, RigidBody::Static, "Static");
            ui.selectable_value(rigid_body, RigidBody::Kinematic, "Kinematic");
        });
}

fn save_scene_system(world: &mut World) {
    let scene_path = world
        .get_resource::<OpenGltf>()
        .map(|o| asset_fs_path(&o.scene_path))
        .unwrap_or_else(|| asset_fs_path("scene.scn.ron"));

    if let Err(e) = save_scene::<With<Editable>>(world, &scene_path) {
        error!("Failed to save scene: {e}");
    } else {
        info!("Scene saved to {}", scene_path.display());
    }
}
