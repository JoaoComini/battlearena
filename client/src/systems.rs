use protocol::*;
use bevy::prelude::*;
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
    mut query: Query<&mut ActionState<Inputs>, With<InputMarker<Inputs>>>,
    keypress: Res<ButtonInput<KeyCode>>,
) {
    if let Ok(mut action_state) = query.single_mut() {
        let mut direction = Direction {
            up: false,
            down: false,
            left: false,
            right: false,
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
