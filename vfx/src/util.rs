use bevy::prelude::*;

pub fn pos2_to_vec3(x: f32, y: f32, height: f32) -> Vec3 {
    Vec3::new(x, height, -y)
}
