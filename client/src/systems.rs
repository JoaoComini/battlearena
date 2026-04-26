use avian2d::prelude::Position;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use inputs::{AbilityInput, Direction, Inputs, PlayerInput};
use lightyear::prelude::client::input::*;
use lightyear::prelude::input::native::*;
use lightyear::prelude::*;
use physics::PlayerPhysicsBundle;
use protocol::*;

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

fn log_rollback(manager: Query<&lightyear::prelude::PredictionManager>) {
    if let Ok(m) = manager.single() {
        if m.is_rollback() {
            info!("Rollback triggered");
        }
    }
}

fn log_interpolated_despawn(trigger: On<Remove, Interpolated>) {
    info!("Interpolated removed from {:?}", trigger.entity);
}

pub(crate) fn buffer_input(
    mut query: Query<(&mut ActionState<Inputs>, &Position), With<InputMarker<Inputs>>>,
    keypress: Res<ButtonInput<KeyCode>>,
    mouse_button_input: Res<ButtonInput<MouseButton>>,
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
        if let (Ok(window), Ok((camera, camera_transform))) = (windows.single(), camera_q.single())
        {
            if let Some(cursor_pos) = window.cursor_position() {
                // Unproject onto the Z=0 plane (the arena floor)
                if let Ok(ray) = camera.viewport_to_world(camera_transform, cursor_pos) {
                    if ray.direction.y.abs() > f32::EPSILON {
                        let t = -ray.origin.y / ray.direction.y;
                        if t > 0.0 {
                            let hit = ray.origin + ray.direction * t;
                            let world_pos = Vec2::new(hit.x, -hit.z);
                            let player_pos = position.0;
                            direction.angle =
                                (world_pos - player_pos).to_angle() - std::f32::consts::FRAC_PI_2;
                        }
                    }
                }
            }
        }
        let abilities = AbilityInput {
            slot1: mouse_button_input.pressed(MouseButton::Left),
            slot2: keypress.pressed(KeyCode::KeyE),
        };
        action_state.0 = Inputs::PlayerInput(PlayerInput {
            movement: direction,
            abilities,
        });
    }
}

pub(crate) fn handle_predicted_spawn(
    trigger: On<Add, (PlayerId, Predicted)>,
    predicted: Query<(), (With<PlayerId>, With<Predicted>)>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    if predicted.get(entity).is_err() {
        return;
    }
    commands.entity(entity).insert((
        PlayerPhysicsBundle::default(),
        InputMarker::<Inputs>::default(),
        LocalPlayer,
    ));
}

pub(crate) fn handle_interpolated_spawn(
    trigger: On<Add, (PlayerId, Interpolated)>,
    interpolated: Query<(), (With<PlayerId>, With<Interpolated>)>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    if interpolated.get(entity).is_err() {
        return;
    }
    commands
        .entity(entity)
        .insert(PlayerPhysicsBundle::default());
}
