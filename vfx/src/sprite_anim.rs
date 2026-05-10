use bevy::math::Affine2;
use bevy::prelude::*;

#[derive(Clone)]
pub enum AnimSource {
    /// Single PNG sprite sheet, frames arranged in a grid (columns × rows).
    Sheet {
        image: Handle<Image>,
        cols: u32,
        rows: u32,
    },
    /// Individual PNG per frame.
    Sequence {
        frames: Vec<Handle<Image>>,
    },
}

#[derive(Component, Clone)]
pub struct SpriteAnim {
    pub source: AnimSource,
    pub frame_count: usize,
    pub current_frame: usize,
    pub timer: Timer,
    pub looping: bool,
}

impl SpriteAnim {
    pub fn sheet(
        image: Handle<Image>,
        cols: u32,
        rows: u32,
        frame_count: usize,
        secs_per_frame: f32,
    ) -> Self {
        Self {
            source: AnimSource::Sheet { image, cols, rows },
            frame_count,
            current_frame: 0,
            timer: Timer::from_seconds(secs_per_frame, TimerMode::Repeating),
            looping: false,
        }
    }

    pub fn sequence(frames: Vec<Handle<Image>>, secs_per_frame: f32) -> Self {
        let frame_count = frames.len();
        Self {
            source: AnimSource::Sequence { frames },
            frame_count,
            current_frame: 0,
            timer: Timer::from_seconds(secs_per_frame, TimerMode::Repeating),
            looping: false,
        }
    }

    /// Returns the StandardMaterial for this anim's first frame.
    pub fn initial_material(&self) -> StandardMaterial {
        match &self.source {
            AnimSource::Sheet { image, cols, rows } => StandardMaterial {
                base_color_texture: Some(image.clone()),
                uv_transform: frame_uv(0, *cols, *rows),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            },
            AnimSource::Sequence { frames } => StandardMaterial {
                base_color_texture: Some(frames[0].clone()),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                ..default()
            },
        }
    }
}

/// Compute the UV affine transform to show frame `index` in a cols×rows grid.
fn frame_uv(index: usize, cols: u32, rows: u32) -> Affine2 {
    let col = (index as u32) % cols;
    let row = (index as u32) / cols;
    let sw = 1.0 / cols as f32;
    let sh = 1.0 / rows as f32;
    Affine2::from_scale_angle_translation(
        Vec2::new(sw, sh),
        0.0,
        Vec2::new(col as f32 * sw, row as f32 * sh),
    )
}

pub fn tick_sprite_anims(
    mut query: Query<(Entity, &mut SpriteAnim, &MeshMaterial3d<StandardMaterial>)>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (entity, mut anim, mat_handle) in &mut query {
        anim.timer.tick(time.delta());
        if !anim.timer.just_finished() {
            continue;
        }

        let next = anim.current_frame + 1;
        if next >= anim.frame_count {
            if anim.looping {
                anim.current_frame = 0;
            } else {
                commands.entity(entity).despawn();
                continue;
            }
        } else {
            anim.current_frame = next;
        }

        let Some(mat) = materials.get_mut(mat_handle) else { continue };

        match &anim.source {
            AnimSource::Sheet { image, cols, rows } => {
                mat.base_color_texture = Some(image.clone());
                mat.uv_transform = frame_uv(anim.current_frame, *cols, *rows);
            }
            AnimSource::Sequence { frames } => {
                mat.base_color_texture = Some(frames[anim.current_frame].clone());
                mat.uv_transform = Affine2::IDENTITY;
            }
        }
    }
}
