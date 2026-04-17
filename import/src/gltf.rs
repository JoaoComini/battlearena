use bevy::gltf::{Gltf, GltfAssetLabel, GltfMesh, GltfNode};
use bevy::prelude::*;

use crate::{MaterialPath, MeshPath};

pub(crate) fn spawn_nodes(
    gltf: &Gltf,
    gltf_path: &str,
    parent: Entity,
    gltf_nodes: &Assets<GltfNode>,
    gltf_meshes: &Assets<GltfMesh>,
    commands: &mut Commands,
) {
    type PrimitiveInfo = (String, Option<String>);
    let nodes: Vec<(String, Transform, Vec<PrimitiveInfo>)> = gltf
        .nodes
        .iter()
        .filter_map(|h| gltf_nodes.get(h))
        .filter_map(|node| {
            let mesh_handle = node.mesh.as_ref()?;
            let gltf_mesh = gltf_meshes.get(mesh_handle)?;
            let primitives = gltf_mesh
                .primitives
                .iter()
                .map(|p| {
                    let mesh_path = GltfAssetLabel::Primitive {
                        mesh: gltf_mesh.index,
                        primitive: p.index,
                    }
                    .from_asset(gltf_path.to_owned())
                    .to_string();
                    let material_path = p.material.as_ref().and_then(|mat_handle| {
                        let idx = gltf.materials.iter().position(|m| m == mat_handle)?;
                        Some(
                            GltfAssetLabel::Material {
                                index: idx,
                                is_scale_inverted: false,
                            }
                            .from_asset(gltf_path.to_owned())
                            .to_string(),
                        )
                    });
                    (mesh_path, material_path)
                })
                .collect();
            let mut transform = node.transform;
            transform.scale *= 100.0;
            Some((node.name.clone(), transform, primitives))
        })
        .collect();

    for (name, transform, primitives) in nodes {
        let node_id = commands.spawn((Name::new(name), transform)).id();
        commands.entity(parent).add_child(node_id);
        for (mesh_path, material_path) in primitives {
            let mut primitive_cmds = commands.spawn((
                MeshPath(mesh_path),
                Transform::default(),
                Visibility::default(),
            ));
            if let Some(mat_path) = material_path {
                primitive_cmds.insert(MaterialPath(mat_path));
            }
            let child = primitive_cmds.id();
            commands.entity(node_id).add_child(child);
        }
    }
}
