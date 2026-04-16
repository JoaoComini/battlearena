use inputs::{Direction, Inputs};
use physics::PlayerPhysicsBundle;
use protocol::*;
use avian2d::prelude::Position;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use lightyear::prelude::client::input::*;
use lightyear::prelude::input::native::*;
use lightyear::prelude::*;

pub struct BattleArenaClientPlugin;

impl Plugin for BattleArenaClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystems::WriteClientInputs),
        );
        app.add_observer(handle_predicted_spawn);
        app.add_observer(handle_interpolated_spawn);
    }
}

pub(crate) fn buffer_input(
    mut query: Query<(&mut ActionState<Inputs>, &Position), With<InputMarker<Inputs>>>,
    keypress: Res<ButtonInput<KeyCode>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    camera_q: Query<(&Camera, &GlobalTransform)>,
) {
    if let Ok((mut action_state, position)) = query.single_mut() {
        let mut direction = Direction {
            up: false,
            down: false,
            left: false,
            right: false,
            angle: 0.0,
        };
        if keypress.pressed(KeyCode::KeyW) || keypress.pressed(KeyCode::ArrowUp) {
            direction.up = true;
        }
        if keypress.pressed(KeyCode::KeyS) || keypress.pressed(KeyCode::ArrowDown) {
            direction.down = true;
        }
        if keypress.pressed(KeyCode::KeyA) || keypress.pressed(KeyCode::ArrowLeft) {
            direction.left = true;
        }
        if keypress.pressed(KeyCode::KeyD) || keypress.pressed(KeyCode::ArrowRight) {
            direction.right = true;
        }
        if let (Ok(window), Ok((camera, camera_transform))) =
            (windows.single(), camera_q.single())
        {
            if let Some(cursor_pos) = window.cursor_position() {
                // Unproject onto the Z=0 plane (the arena floor)
                if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    let t = -ray.origin.z / ray.direction.z;
                    if t > 0.0 {
                        let world_pos = (ray.origin + ray.direction * t).xy();
                        let player_pos = position.0;
                        direction.angle = (world_pos - player_pos).to_angle() - std::f32::consts::FRAC_PI_2;
                    }
                }
            }
        }
        action_state.0 = Inputs::Direction(direction);
    }
}

pub(crate) fn handle_predicted_spawn(
    trigger: On<Add, (PlayerId, Predicted)>,
    mut query: Query<&mut PlayerColor, With<Predicted>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(mut color) = query.get_mut(entity) else {
        return;
    };
    let hsva = Hsva {
        saturation: 0.4,
        ..Hsva::from(color.0)
    };
    color.0 = Color::from(hsva);
    commands.entity(entity).insert((
        PlayerPhysicsBundle::default(),
        InputMarker::<Inputs>::default(),
    ));
}

pub(crate) fn handle_interpolated_spawn(
    trigger: On<Add, (PlayerId, Interpolated)>,
    mut query: Query<&mut PlayerColor, With<Interpolated>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    let Ok(mut color) = query.get_mut(entity) else {
        return;
    };
    let hsva = Hsva {
        saturation: 0.1,
        ..Hsva::from(color.0)
    };
    color.0 = Color::from(hsva);
    commands
        .entity(entity)
        .insert(PlayerPhysicsBundle::default());
}
