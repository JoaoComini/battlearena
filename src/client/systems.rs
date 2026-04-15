use crate::protocol::*;
use bevy::prelude::*;
use lightyear::connection::client::Connected;
use lightyear::prelude::client::input::*;
use lightyear::prelude::input::native::*;
use lightyear::prelude::*;
use lightyear::prelude::MessageSender;

pub struct BattleArenaClientPlugin;

impl Plugin for BattleArenaClientPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectedCharacter>();
        app.add_systems(
            FixedPreUpdate,
            buffer_input.in_set(InputSystems::WriteClientInputs),
        );
        app.add_systems(Update, handle_character_selection_input);
        app.add_observer(handle_predicted_spawn);
        app.add_observer(handle_interpolated_spawn);
        app.add_observer(send_character_selection);
    }
}

/// The character the local player has selected (defaults to Peta).
#[derive(Resource, Default)]
pub struct SelectedCharacter(pub CharacterKind);

pub(crate) fn handle_character_selection_input(
    keypress: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<SelectedCharacter>,
) {
    if keypress.just_pressed(KeyCode::Digit1) {
        selection.0 = CharacterKind::Peta;
    }
    if keypress.just_pressed(KeyCode::Digit2) {
        selection.0 = CharacterKind::Comini;
    }
}

pub(crate) fn send_character_selection(
    _trigger: On<Add, Connected>,
    selection: Res<SelectedCharacter>,
    mut sender_query: Query<&mut MessageSender<SelectCharacter>>,
) {
    if let Ok(mut sender) = sender_query.single_mut() {
        sender.send::<Channel2>(SelectCharacter(selection.0));
        info!("Sent character selection: {:?}", selection.0);
    } else {
        error!("No MessageSender<SelectCharacter> found on Client entity");
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
