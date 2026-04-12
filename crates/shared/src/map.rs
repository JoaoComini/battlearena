/// Axis-aligned obstacle definition in Pos2 space (x right, y up).
pub struct ObstacleDef {
    pub center_x: f32,
    pub center_y: f32,
    pub half_x: f32,
    pub half_y: f32,
}

pub const MAP_OBSTACLES: &[ObstacleDef] = &[
    ObstacleDef { center_x:  300.0, center_y:  200.0, half_x: 50.0, half_y: 50.0 },
    ObstacleDef { center_x: -200.0, center_y: -150.0, half_x: 70.0, half_y: 40.0 },
    ObstacleDef { center_x:  100.0, center_y: -300.0, half_x: 40.0, half_y: 60.0 },
];
