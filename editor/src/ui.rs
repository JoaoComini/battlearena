use avian2d::prelude::{ColliderConstructor, RigidBody};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};

use scene::save;
use scene::MeshPath;

use crate::file_dialog::{open_file_dialog, save_file_dialog, FilePicked, SaveFilePicked};
use crate::hierarchy::hierarchy_panel;
use crate::selection::SelectedEntity;
use crate::spawn::{
    asset_fs_path, fs_to_asset_path, ActiveSceneRoot, OpenScene, SceneDirty, ScenePath,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            EguiPrimaryContextPass,
            (toolbar, hierarchy_panel, inspector_panel).chain(),
        );
        app.add_systems(Update, keyboard_shortcuts);
        app.add_observer(on_file_picked);
        app.add_observer(on_save_file_picked);
    }
}

fn toolbar(
    mut contexts: EguiContexts,
    mut commands: Commands,
    scene_root: Query<Has<SceneDirty>, With<ActiveSceneRoot>>,
) -> Result {
    let dirty = scene_root.single().unwrap_or(false);

    egui::TopBottomPanel::top("toolbar").show(contexts.ctx_mut()?, |ui| {
        egui::MenuBar::new().ui(ui, |ui| {
            ui.menu_button("File", |ui| {
                if ui.button("Open Scene").clicked() {
                    open_file_dialog(&mut commands, &[("Scene files", &["scn", "gltf", "glb"])]);
                    ui.close();
                }
                let save_label = if dirty { "Save Scene *" } else { "Save Scene" };
                if ui.button(save_label).clicked() {
                    commands.run_system_cached(save_or_save_as_system);
                    ui.close();
                }
                if ui.button("Save Scene As...").clicked() {
                    save_file_dialog(&mut commands, &[("Scene files", &["scn"])]);
                    ui.close();
                }
            });
        });
    });

    Ok(())
}

fn keyboard_shortcuts(mut commands: Commands, input: Res<ButtonInput<KeyCode>>) {
    let ctrl = input.pressed(KeyCode::ControlLeft) || input.pressed(KeyCode::ControlRight);
    if ctrl && input.just_pressed(KeyCode::KeyS) {
        commands.run_system_cached(save_or_save_as_system);
    }
}

fn on_file_picked(
    trigger: On<FilePicked>,
    mut commands: Commands,
    scene_root: Query<Entity, With<ActiveSceneRoot>>,
) {
    let Some(asset_path) = fs_to_asset_path(&trigger.0) else {
        error!(
            "Selected file is outside the assets folder: {:?}",
            trigger.0
        );
        return;
    };
    if let Ok(entity) = scene_root.single() {
        commands
            .entity(entity)
            .despawn_related::<Children>()
            .remove::<SceneDirty>()
            .insert(OpenScene(asset_path));
    }
}

fn on_save_file_picked(
    trigger: On<SaveFilePicked>,
    mut commands: Commands,
    scene_root: Query<Entity, With<ActiveSceneRoot>>,
) {
    let path = trigger.0.with_extension("scn");
    let Some(asset_path) = fs_to_asset_path(&path) else {
        error!("Save path is outside the assets folder: {:?}", path);
        return;
    };
    if let Ok(entity) = scene_root.single() {
        commands.entity(entity).insert(ScenePath(asset_path));
    }
    commands.run_system_cached(save_scene);
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
                        let mut euler =
                            Vec3::new(yaw.to_degrees(), pitch.to_degrees(), roll.to_degrees());
                        let mut scale = transform.scale;

                        egui::Grid::new("transform_grid").show(ui, |ui| {
                            ui.label("Position");
                            ui.add(
                                egui::DragValue::new(&mut translation.x)
                                    .prefix("X: ")
                                    .speed(0.1),
                            );
                            ui.add(
                                egui::DragValue::new(&mut translation.y)
                                    .prefix("Y: ")
                                    .speed(0.1),
                            );
                            ui.add(
                                egui::DragValue::new(&mut translation.z)
                                    .prefix("Z: ")
                                    .speed(0.1),
                            );
                            ui.end_row();

                            ui.label("Rotation");
                            ui.add(
                                egui::DragValue::new(&mut euler.x)
                                    .prefix("Y: ")
                                    .speed(1.0)
                                    .suffix("°"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut euler.y)
                                    .prefix("X: ")
                                    .speed(1.0)
                                    .suffix("°"),
                            );
                            ui.add(
                                egui::DragValue::new(&mut euler.z)
                                    .prefix("Z: ")
                                    .speed(1.0)
                                    .suffix("°"),
                            );
                            ui.end_row();

                            ui.label("Scale");
                            ui.add(egui::DragValue::new(&mut scale.x).prefix("X: ").speed(0.01));
                            ui.add(egui::DragValue::new(&mut scale.y).prefix("Y: ").speed(0.01));
                            ui.add(egui::DragValue::new(&mut scale.z).prefix("Z: ").speed(0.01));
                            ui.end_row();
                        });

                        transform.translation = translation;
                        transform.rotation = Quat::from_euler(
                            EulerRot::YXZ,
                            euler.x.to_radians(),
                            euler.y.to_radians(),
                            euler.z.to_radians(),
                        );
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
                    if ui.button("Collider").clicked() {
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
                ui.add(egui::DragValue::new(radius).speed(0.1));
            });
        }
        ColliderConstructor::Rectangle { x_length, y_length } => {
            ui.horizontal(|ui| {
                ui.label("X length");
                ui.add(egui::DragValue::new(x_length).speed(0.1));
            });
            ui.horizontal(|ui| {
                ui.label("Y length");
                ui.add(egui::DragValue::new(y_length).speed(0.1));
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

fn save_or_save_as_system(world: &mut World) {
    let root = world
        .query_filtered::<(Entity, Has<ScenePath>), With<ActiveSceneRoot>>()
        .single(world);

    let Ok((_, has_path)) = root else {
        error!("No ActiveSceneRoot found, cannot save");
        return;
    };

    if has_path {
        save_scene(world);
    } else {
        let mut commands = world.commands();
        save_file_dialog(&mut commands, &[("Scene files", &["scn"])]);
    }
}

fn save_scene(world: &mut World) {
    let root = world
        .query_filtered::<Entity, With<ActiveSceneRoot>>()
        .single(world);

    let Ok(root) = root else {
        error!("No ActiveSceneRoot found, cannot save");
        return;
    };

    let scene_path = world.get::<ScenePath>(root).map(|s| s.0.clone());

    let Some(scene_path) = scene_path else {
        error!("No .scn path set, use Save As to choose a destination");
        return;
    };

    let fs_path = asset_fs_path(&scene_path);

    match save(root, world, &fs_path) {
        Ok(()) => {
            world.entity_mut(root).remove::<SceneDirty>();
            info!("Scene saved to {}", fs_path.display());
        }
        Err(e) => error!("Failed to save scene: {e}"),
    }
}
