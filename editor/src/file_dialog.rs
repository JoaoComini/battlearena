use bevy::prelude::*;
use bevy::tasks::{block_on, poll_once, AsyncComputeTaskPool, Task};
use rfd::AsyncFileDialog;
use std::path::PathBuf;

#[derive(Event)]
pub struct FilePicked(pub PathBuf);

#[derive(Resource)]
struct PendingFileDialog(Task<Option<PathBuf>>);

pub struct FileDialogPlugin;

impl Plugin for FileDialogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, poll_file_dialog);
    }
}

pub fn open_file_dialog(commands: &mut Commands, filters: &[(&str, &[&str])]) {
    let mut dialog = AsyncFileDialog::new();
    for (name, extensions) in filters {
        dialog = dialog.add_filter(*name, extensions);
    }

    let task = AsyncComputeTaskPool::get().spawn(async move {
        dialog.pick_file().await.map(|f| f.path().to_path_buf())
    });

    commands.insert_resource(PendingFileDialog(task));
}

fn poll_file_dialog(mut commands: Commands, mut pending: Option<ResMut<PendingFileDialog>>) {
    let Some(ref mut pending) = pending else { return };

    if let Some(result) = block_on(poll_once(&mut pending.0)) {
        commands.remove_resource::<PendingFileDialog>();
        if let Some(path) = result {
            commands.trigger(FilePicked(path));
        }
    }
}
