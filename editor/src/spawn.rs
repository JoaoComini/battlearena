use avian2d::prelude::{Position, Rotation};
use bevy::prelude::*;
use import::ImportGltf;
use scene::LoadScene;

/// Marks the root entity of the active scene in the editor.
#[derive(Component)]
pub struct ActiveSceneRoot;

/// Marks the active scene as having unsaved changes.
#[derive(Component)]
pub struct SceneDirty;

/// Stores the asset-relative path of the currently loaded scene.
#[derive(Component)]
pub struct ScenePath(pub String);

/// Place this component on an entity to open a scene file.
/// Dispatches to `LoadScene` for `.scn` files or `ImportGltf` for GLTF.
#[derive(Component)]
pub struct OpenScene(pub String);

pub struct SpawnPlugin;

impl Plugin for SpawnPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(PostUpdate, transform_to_position);
        app.add_systems(Startup, setup);
        app.add_systems(Update, open_scene);
    }
}

fn open_scene(mut commands: Commands, query: Query<(Entity, &OpenScene)>) {
    for (entity, open) in &query {
        let path = open.0.clone();
        let mut entity_cmds = commands.entity(entity);
        entity_cmds.remove::<OpenScene>();
        if path.ends_with(".scn") {
            entity_cmds.insert((ScenePath(path.clone()), LoadScene(path)));
        } else {
            entity_cmds.remove::<ScenePath>().insert(ImportGltf(path));
        }
    }
}

fn transform_to_position(
    mut query: Query<
        (&GlobalTransform, &mut Position, Option<&mut Rotation>),
        Changed<GlobalTransform>,
    >,
) {
    for (transform, mut position, rotation) in &mut query {
        let translation = transform.translation();
        position.x = translation.x;
        position.y = -translation.z;
        if let Some(mut rot) = rotation {
            let (yaw, _, _) = transform
                .to_scale_rotation_translation()
                .1
                .to_euler(EulerRot::YXZ);
            *rot = Rotation::radians(yaw);
        }
    }
}

/// Derives the absolute filesystem path for an asset-relative path.
pub fn asset_fs_path(asset_path: &str) -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(asset_path)
}

/// Strips the assets root prefix from an absolute path, returning an asset-relative path.
pub fn fs_to_asset_path(path: &std::path::Path) -> Option<String> {
    let assets_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap();
    path.strip_prefix(&assets_root)
        .ok()
        .and_then(|p| p.to_str())
        .map(|s| s.to_string())
}

fn setup(mut commands: Commands) {
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

    commands.spawn((ActiveSceneRoot, Transform::default(), Visibility::default()));
}
