use avian2d::prelude::{ColliderConstructor, RigidBody};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use scene::MeshPath;
use scene::save;

use crate::hierarchy::hierarchy_panel;
use crate::selection::SelectedEntity;
use crate::spawn::{asset_fs_path, ActiveSceneRoot, OpenScene, ScenePath};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OpenSceneDialog>();
        app.add_systems(
            EguiPrimaryContextPass,
            (toolbar, hierarchy_panel, inspector_panel).chain(),
        );
    }
}

#[derive(Resource, Default)]
struct OpenSceneDialog {
    open: bool,
    path_input: String,
}

fn toolbar(
    mut contexts: EguiContexts,
    mut dialog: ResMut<OpenSceneDialog>,
    mut commands: Commands,
    scene_root: Query<Entity, With<ActiveSceneRoot>>,
) -> Result {
    egui::TopBottomPanel::top("toolbar").show(contexts.ctx_mut()?, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Scene").clicked() {
                    dialog.open = true;
                    ui.close();
                }
            });
        });
    });

    if dialog.open {
        egui::Window::new("Open Scene")
            .collapsible(false)
            .resizable(false)
            .show(contexts.ctx_mut()?, |ui| {
                ui.label("Scene path:");
                let response = ui.text_edit_singleline(&mut dialog.path_input);
                if response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    dialog.open = false;
                    dialog.path_input.clear();
                }
                ui.horizontal(|ui| {
                    let can_open = !dialog.path_input.is_empty();
                    if ui
                        .add_enabled(can_open, egui::Button::new("Open"))
                        .clicked()
                        || (response.lost_focus()
                            && ui.input(|i| i.key_pressed(egui::Key::Enter))
                            && can_open)
                    {
                        let path = dialog.path_input.trim().to_string();
                        if let Ok(entity) = scene_root.single() {
                            commands
                                .entity(entity)
                                .despawn_related::<Children>()
                                .insert(OpenScene(path));
                        }
                        dialog.open = false;
                        dialog.path_input.clear();
                    }
                    if ui.button("Cancel").clicked() {
                        dialog.open = false;
                        dialog.path_input.clear();
                    }
                });
            });
    }

    Ok(())
}

fn inspector_panel(
    mut contexts: EguiContexts,
    selected: Res<SelectedEntity>,
    node_query: Query<(Option<&MeshPath>, Option<&Name>)>,
    mesh_query: Query<&Mesh3d>,
    mut transform_query: Query<&mut Transform>,
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
            if let Ok((mesh_path, name)) = node_query.get(entity) {
                if let Some(n) = name {
                    ui.label(format!("Name: {}", n));
                }
                if let Some(p) = mesh_path {
                    ui.label(format!("Mesh: {}", p.0));
                }
            }
            ui.separator();

            if let Ok(mut transform) = transform_query.get_mut(entity) {
                egui::CollapsingHeader::new("Transform")
                    .default_open(true)
                    .show(ui, |ui| {
                        let mut translation = transform.translation;
                        let (yaw, pitch, roll) = transform.rotation.to_euler(EulerRot::YXZ);
                        let mut euler = Vec3::new(yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees());
                        let mut scale = transform.scale;

                        egui::Grid::new("transform_grid").show(ui, |ui| {
                            ui.label("Position");
                            ui.add(egui::DragValue::new(&mut translation.x).prefix("X: ").speed(0.1));
                            ui.add(egui::DragValue::new(&mut translation.y).prefix("Y: ").speed(0.1));
                            ui.add(egui::DragValue::new(&mut translation.z).prefix("Z: ").speed(0.1));
                            ui.end_row();

                            ui.label("Rotation");
                            ui.add(egui::DragValue::new(&mut euler.x).prefix("Y: ").speed(1.0).suffix("°"));
                            ui.add(egui::DragValue::new(&mut euler.y).prefix("X: ").speed(1.0).suffix("°"));
                            ui.add(egui::DragValue::new(&mut euler.z).prefix("Z: ").speed(1.0).suffix("°"));
                            ui.end_row();

                            ui.label("Scale");
                            ui.add(egui::DragValue::new(&mut scale.x).prefix("X: ").speed(0.01));
                            ui.add(egui::DragValue::new(&mut scale.y).prefix("Y: ").speed(0.01));
                            ui.add(egui::DragValue::new(&mut scale.z).prefix("Z: ").speed(0.01));
                            ui.end_row();
                        });

                        transform.translation = translation;
                        transform.rotation = Quat::from_euler(EulerRot::YXZ, euler.x.to_radians(), euler.y.to_radians(), euler.z.to_radians());
                        transform.scale = scale;
                    });
                ui.separator();
            }

            if let Ok(mesh) = mesh_query.get(entity) {
                egui::CollapsingHeader::new("Mesh")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.label(format!("Mesh:     {:?}", mesh.id()));
                    });
                ui.separator();
            }

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
                ui.add(
                    egui::DragValue::new(radius)
                        .speed(0.1)
                        ,
                );
            });
        }
        ColliderConstructor::Rectangle { x_length, y_length } => {
            ui.horizontal(|ui| {
                ui.label("X length");
                ui.add(
                    egui::DragValue::new(x_length)
                        .speed(0.1)
                        ,
                );
            });
            ui.horizontal(|ui| {
                ui.label("Y length");
                ui.add(
                    egui::DragValue::new(y_length)
                        .speed(0.1)
                        ,
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
    let root = world
        .query_filtered::<Entity, With<ActiveSceneRoot>>()
        .single(world);

    let Ok(root) = root else {
        error!("No ActiveSceneRoot found, cannot save");
        return;
    };

    let loaded_path = world.get::<ScenePath>(root).map(|s| s.0.clone());

    let Some(loaded_path) = loaded_path else {
        error!("No scene loaded, cannot save");
        return;
    };

    let scene_path = asset_fs_path(&loaded_path).with_extension("scn");

    match save(root, world, &scene_path) {
        Ok(()) => info!("Scene saved to {}", scene_path.display()),
        Err(e) => error!("Failed to save scene: {e}"),
    }
}
