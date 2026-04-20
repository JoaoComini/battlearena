use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use rfd::AsyncFileDialog;
use std::path::PathBuf;

fn assets_dir() -> PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("assets")
}

#[derive(Event)]
pub struct FilePicked(pub PathBuf);

#[derive(Event)]
pub struct SaveFilePicked(pub PathBuf);

#[derive(Resource)]
struct PendingOpenDialog(Task<Option<PathBuf>>);

#[derive(Resource)]
struct PendingSaveDialog(Task<Option<PathBuf>>);

pub struct FileDialogPlugin;

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (poll_open_dialog, poll_save_dialog));
    }
}

pub fn open_file_dialog(commands: &mut Commands, filters: &[(&str, &[&str])]) {
    let mut dialog = AsyncFileDialog::new().set_directory(assets_dir());
    for (name, extensions) in filters {
        dialog = dialog.add_filter(*name, extensions);
    }

    let task = AsyncComputeTaskPool::get().spawn(async move {
        dialog.pick_file().await.map(|f| f.path().to_path_buf())
    });

    commands.insert_resource(PendingOpenDialog(task));
}

pub fn save_file_dialog(commands: &mut Commands, filters: &[(&str, &[&str])]) {
    let mut dialog = AsyncFileDialog::new().set_directory(assets_dir());
    for (name, extensions) in filters {
        dialog = dialog.add_filter(*name, extensions);
    }

    let task = AsyncComputeTaskPool::get().spawn(async move {
        dialog.save_file().await.map(|f| f.path().to_path_buf())
    });

    commands.insert_resource(PendingSaveDialog(task));
}

fn poll_open_dialog(mut commands: Commands, mut pending: Option<ResMut<PendingOpenDialog>>) {
    let Some(ref mut pending) = pending else { return };

    if let Some(result) = block_on(poll_once(&mut pending.0)) {
        commands.remove_resource::<PendingOpenDialog>();
        if let Some(path) = result {
            commands.trigger(FilePicked(path));
        }
    }
}

fn poll_save_dialog(mut commands: Commands, mut pending: Option<ResMut<PendingSaveDialog>>) {
    let Some(ref mut pending) = pending else { return };

    if let Some(result) = block_on(poll_once(&mut pending.0)) {
        commands.remove_resource::<PendingSaveDialog>();
        if let Some(path) = result {
            commands.trigger(SaveFilePicked(path));
        }
    }
}
