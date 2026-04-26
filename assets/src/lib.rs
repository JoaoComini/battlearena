use bevy::asset::io::AssetSourceBuilder;
use bevy::prelude::*;

pub struct AssetPlugin;

impl Plugin for AssetPlugin {
    fn build(&self, app: &mut App) {
        app.register_asset_source(
            "abilities",
            AssetSourceBuilder::platform_default(
                concat!(env!("CARGO_MANIFEST_DIR"), ""),
                None,
            ),
        );
    }
}
