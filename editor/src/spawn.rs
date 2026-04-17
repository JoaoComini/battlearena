use bevy::ecs::system::SystemState;
use bevy::gltf::{Gltf, GltfMesh, GltfNode};
use bevy::prelude::*;
use scene::{load, spawn as spawn_scene, GltfPrimitiveRef};

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
        crate::free_camera::FreeCamera,
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
fn spawn_gltf_nodes(world: &mut World) {
    // Check whether we should proceed.
    {
        let open = world.resource::<OpenGltf>();
        if open.spawned {
            return;
        }
        let gltf_assets = world.resource::<Assets<Gltf>>();
        if gltf_assets.get(&open.handle.clone()).is_none() {
            return;
        }
    }

    let scene_fs_path = asset_fs_path(&world.resource::<OpenGltf>().scene_path);

    if scene_fs_path.exists() {
        world.resource_mut::<OpenGltf>().spawned = true;

        match load(&scene_fs_path) {
            Ok(scene_file) => {
                // Snapshot existing entity IDs before spawning so we can
                // identify the newly created ones afterward.
                let before: std::collections::HashSet<Entity> =
                    world.query::<Entity>().iter(world).collect();

                spawn_scene(&scene_file, world);

                // Mark every newly spawned entity as Editable.
                let new_entities: Vec<Entity> = world
                    .query::<Entity>()
                    .iter(world)
                    .filter(|id| !before.contains(id))
                    .collect();
                for id in new_entities {
                    world.entity_mut(id).insert(Editable);
                }

                info!("Loaded scene from {}", scene_fs_path.display());
            }
            Err(e) => error!("Failed to load scene: {e}"),
        }
    } else {
        // No scene file — bootstrap from GLTF nodes.
        let mut state: SystemState<(
            Res<OpenGltf>,
            Res<Assets<Gltf>>,
            Res<Assets<GltfNode>>,
            Res<Assets<GltfMesh>>,
        )> = SystemState::new(world);

        let (open, gltf_assets, gltf_nodes, gltf_meshes) = state.get(world);

        let gltf = gltf_assets.get(&open.handle).unwrap();

        // Collect all the data we need before dropping the borrows.
        let nodes: Vec<(String, Transform, String, Vec<(usize, usize)>)> = gltf
            .nodes
            .iter()
            .filter_map(|h| gltf_nodes.get(h))
            .filter_map(|node| {
                let mesh_handle = node.mesh.as_ref()?;
                let gltf_mesh = gltf_meshes.get(mesh_handle)?;
                let primitives = gltf_mesh
                    .primitives
                    .iter()
                    .map(|p| (gltf_mesh.index, p.index))
                    .collect();
                let mut transform = node.transform;
                transform.scale *= 100.0;
                Some((node.name.clone(), transform, open.path.clone(), primitives))
            })
            .collect();

        drop((open, gltf_assets, gltf_nodes, gltf_meshes));

        world.resource_mut::<OpenGltf>().spawned = true;

        for (name, transform, path, primitives) in nodes {
            let children: Vec<Entity> = primitives
                .iter()
                .map(|(mesh_index, primitive_index)| {
                    world
                        .spawn((
                            GltfPrimitiveRef {
                                path: path.clone(),
                                mesh_index: *mesh_index,
                                primitive_index: *primitive_index,
                            },
                            Editable,
                            Transform::default(),
                        ))
                        .id()
                })
                .collect();

            let node_id = world.spawn((Name::new(name), Editable, transform)).id();
            world.entity_mut(node_id).add_children(&children);
        }
    }
}
