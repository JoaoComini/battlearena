use crate::protocol::*;
use crate::shared;
use bevy::prelude::*;
use lightyear::prelude::client::input::*;
use lightyear::prelude::input::native::*;
use lightyear::prelude::*;

pub struct ExampleClientPlugin;

impl Plugin for ExampleClientPlugin {
    fn build(&self, app: &mut App) {
        // Inputs must be buffered in WriteClientInputs so lightyear picks them
        // up at the correct point in the tick.
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystems::WriteClientInputs),
        );
        app.add_systems(FixedUpdate, player_movement);

        app.add_observer(handle_predicted_spawn);
        app.add_observer(handle_interpolated_spawn);
    }
}

/// Sample keyboard state and write it into the ActionState buffer.
/// Must run in FixedPreUpdate / WriteClientInputs — do not move it.
fn buffer_input(
    mut query: Query<&mut ActionState<Inputs>, With<InputMarker<Inputs>>>,
    keypress: Res<ButtonInput<KeyCode>>,
) {
    let Ok(mut action_state) = query.single_mut() else {
        return;
    };

    let direction = Direction {
        up:    keypress.pressed(KeyCode::KeyW) || keypress.pressed(KeyCode::ArrowUp),
        down:  keypress.pressed(KeyCode::KeyS) || keypress.pressed(KeyCode::ArrowDown),
        left:  keypress.pressed(KeyCode::KeyA) || keypress.pressed(KeyCode::ArrowLeft),
        right: keypress.pressed(KeyCode::KeyD) || keypress.pressed(KeyCode::ArrowRight),
    };

    action_state.0 = Inputs::Direction(direction);
}

/// Apply buffered inputs to the locally-predicted player entity each fixed tick.
fn player_movement(
    mut position_query: Query<(&mut PlayerPosition, &ActionState<Inputs>), With<Predicted>>,
) {
    for (position, input) in position_query.iter_mut() {
        shared::shared_movement_behaviour(position, input);
    }
}

/// When lightyear spawns our Predicted entity, attach the InputMarker so that
/// buffer_input knows which entity to write inputs to.
pub fn handle_predicted_spawn(
    trigger: On<Add, (PlayerId, Predicted)>,
    mut predicted: Query<&mut PlayerColor, With<Predicted>>,
    mut commands: Commands,
) {
    let entity = trigger.entity;
    if let Ok(mut color) = predicted.get_mut(entity) {
        // Make the local player slightly desaturated to distinguish it visually.
        let hsva = Hsva {
            saturation: 0.4,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
        commands
            .entity(entity)
            .insert(InputMarker::<Inputs>::default());
    }
}

/// When an Interpolated (remote) player entity is spawned, reduce its saturation
/// so remote players look visually distinct from the local player.
pub fn handle_interpolated_spawn(
    trigger: On<Add, Interpolated>,
    mut interpolated: Query<&mut PlayerColor>,
) {
    if let Ok(mut color) = interpolated.get_mut(trigger.entity) {
        let hsva = Hsva {
            saturation: 0.1,
            ..Hsva::from(color.0)
        };
        color.0 = Color::from(hsva);
    }
}
