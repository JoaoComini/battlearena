use crate::free_camera::FreeCamera;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::prelude::*;
use scene::{load_scene, GltfPrimitiveRef};

/// Marks an entity as editable and visible in the hierarchy panel.
#[derive(Component, Reflect, Default)]
#[reflect(Component)]
#[require(Visibility)]
pub struct Editable;

/// The GLTF file currently open in the editor.
#[derive(Resource)]
pub struct OpenGltf {
    pub path: String,
    pub scene_path: String,
    pub handle: Handle<Gltf>,
    pub spawned: bool,
}

impl OpenGltf {
    pub fn new(path: &str, asset_server: &AssetServer) -> Self {
        let scene_path = path
            .trim_end_matches(".glb")
            .trim_end_matches(".gltf")
            .to_string()
            + ".scn.ron";
        Self {
            path: path.to_string(),
            scene_path,
            handle: asset_server.load(path.to_string()),
            spawned: false,
        }
    }
}

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup);
        app.add_systems(Update, spawn_gltf_nodes);
    }
}

/// Derives the absolute filesystem path for an asset-relative path.
pub fn asset_fs_path(asset_path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(asset_path)
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera,
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,
            ..default()
        },
        Transform::from_xyz(4.0, 8.0, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    let path = "assets/models/arena.glb";
    commands.insert_resource(OpenGltf::new(path, &asset_server));
}

/// Waits for the `Gltf` asset to load, then either loads the saved scene or
/// spawns entities from the GLTF nodes directly.
fn spawn_gltf_nodes(
    mut commands: Commands,
    mut open: ResMut<OpenGltf>,
    gltf_assets: Res<Assets<Gltf>>,
    gltf_nodes: Res<Assets<GltfNode>>,
    gltf_meshes: Res<Assets<GltfMesh>>,
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
) {
    if open.spawned {
        return;
    }

    let Some(gltf) = gltf_assets.get(&open.handle) else {
        return;
    };

    let scene_fs_path = asset_fs_path(&open.scene_path);

    if scene_fs_path.exists() {
        load_scene(&asset_server, &mut scene_spawner, &open.scene_path);
        info!("Loaded scene from {}", scene_fs_path.display());
    } else {
        // No scene file — bootstrap from GLTF nodes.
        for node_handle in gltf.nodes.iter() {
            let Some(node) = gltf_nodes.get(node_handle) else {
                continue;
            };
            let Some(mesh_handle) = &node.mesh else {
                continue;
            };
            let Some(gltf_mesh) = gltf_meshes.get(mesh_handle) else {
                continue;
            };

            let mut transform = node.transform;
            transform.scale *= 100.0;

            let primitive_entities: Vec<Entity> = gltf_mesh
                .primitives
                .iter()
                .map(|primitive| {
                    commands
                        .spawn((
                            GltfPrimitiveRef {
                                path: open.path.clone(),
                                mesh_index: gltf_mesh.index,
                                primitive_index: primitive.index,
                            },
                            Editable,
                            Transform::default(),
                        ))
                        .id()
                })
                .collect();

            let mut node_entity = commands.spawn((
                Name::new(node.name.clone()),
                Editable,
                transform,
            ));
            node_entity.add_children(&primitive_entities);
        }
    }

    open.spawned = true;
}
