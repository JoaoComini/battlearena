use bevy::prelude::*;
use inputs::{Inputs, PlayerInput};
use lightyear::prelude::input::native::{ActionState, InputMarker};
use lightyear::prelude::*;
use protocol::{AbilityChannel, UseAbility};

pub struct AbilityClientPlugin;

impl Plugin for AbilityClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedUpdate, send_ability_requests);
    }
}

fn send_ability_requests(
    mut sender: Query<&mut MessageSender<UseAbility>>,
    inputs: Query<&ActionState<Inputs>, With<InputMarker<Inputs>>>,
) {
    let Ok(mut sender) = sender.single_mut() else { return };
    let Ok(action_state) = inputs.single() else { return };

    let Inputs::PlayerInput(PlayerInput { abilities, .. }) = &action_state.0;

    if abilities.slot1 {
        sender.send::<AbilityChannel>(UseAbility { slot: 0 });
    }
    if abilities.slot2 {
        sender.send::<AbilityChannel>(UseAbility { slot: 1 });
    }
}
