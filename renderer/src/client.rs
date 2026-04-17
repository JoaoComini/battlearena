use abilities::types::AbilityLoadout;
use bevy::picking::prelude::{Click, Pointer};
use bevy::prelude::*;
use lightyear::connection::client::ClientState;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use protocol::LocalPlayer;

pub struct BattleArenaClientRendererPlugin {
    pub name: String,
}

impl BattleArenaClientRendererPlugin {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

#[derive(Resource)]
struct GameName(String);

impl Plugin for BattleArenaClientRendererPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(GameName(self.name.clone()));
        app.insert_resource(ClearColor::default());
        app.add_systems(Startup, set_window_title);
        spawn_connect_button(app);
        app.add_systems(Update, update_button_text);
        app.add_observer(on_update_status_message);
        app.add_observer(handle_connection);
        app.add_observer(handle_disconnection);

        app.add_systems(Startup, spawn_ability_hud);
        app.add_systems(Update, update_ability_hud);
    }
}

fn set_window_title(mut window: Query<&mut Window>, game_name: Res<GameName>) {
    let mut window = window.single_mut().unwrap();
    window.title = format!("BattleArena: {}", game_name.0);
}

#[derive(Event, Debug)]
pub struct UpdateStatusMessage(pub String);

fn on_update_status_message(
    trigger: On<UpdateStatusMessage>,
    mut q: Query<&mut Text, With<StatusMessageMarker>>,
) {
    for mut text in &mut q {
        text.0 = trigger.event().0.clone();
    }
}

#[derive(Component)]
struct StatusMessageMarker;

#[derive(Component)]
pub struct ClientButton;

pub fn spawn_connect_button(app: &mut App) {
    app.world_mut()
        .spawn(Node {
            width: Val::Percent(30.0),
            height: Val::Percent(30.0),
            position_type: PositionType::Absolute,
            bottom: Val::Px(0.0),
            align_items: AlignItems::FlexEnd,
            justify_content: JustifyContent::FlexEnd,
            justify_self: JustifySelf::End,
            flex_direction: FlexDirection::Row,
            ..default()
        })
        .with_children(|parent| {
            parent.spawn((
                Text("[Client]".to_string()),
                TextColor(Color::srgb(0.9, 0.9, 0.9).with_alpha(0.4)),
                TextFont::from_font_size(18.0),
                StatusMessageMarker,
                Node {
                    padding: UiRect::all(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
            ));
            parent
                .spawn((
                    Text("Connect".to_string()),
                    TextColor(Color::srgb(0.9, 0.9, 0.9)),
                    TextFont::from_font_size(20.0),
                    ClientButton,
                    Node {
                        width: Val::Px(150.0),
                        height: Val::Px(65.0),
                        border: UiRect::all(Val::Px(5.0)),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    },
                    Button,
                ))
                .observe(
                    |_: On<Pointer<Click>>,
                     mut commands: Commands,
                     query: Query<(Entity, &Client)>| {
                        let Ok((entity, client)) = query.single() else {
                            return;
                        };
                        match client.state {
                            ClientState::Disconnected => {
                                commands.trigger(Connect { entity });
                            }
                            _ => {
                                commands.trigger(Disconnect { entity });
                            }
                        };
                    },
                );
        });
}

pub fn update_button_text(
    client: Single<&Client>,
    mut text_query: Query<&mut Text, (With<Button>, With<ClientButton>)>,
) {
    if let Ok(mut text) = text_query.single_mut() {
        match client.state {
            ClientState::Disconnecting => {
                text.0 = "Disconnecting".to_string();
            }
            ClientState::Disconnected => {
                text.0 = "Connect".to_string();
            }
            ClientState::Connecting => {
                text.0 = "Connecting".to_string();
            }
            ClientState::Connected => {
                text.0 = "Disconnect".to_string();
            }
        }
    }
}

#[derive(Component)]
pub struct ClientIdText;

pub fn handle_connection(
    trigger: On<Add, Connected>,
    query: Query<&LocalId, Or<((With<LinkOf>, With<Client>), Without<LinkOf>)>>,
    mut commands: Commands,
) {
    if let Ok(client_id) = query.get(trigger.entity) {
        commands.spawn((
            Text(format!("Client {}", client_id.0)),
            TextFont::from_font_size(30.0),
            ClientIdText,
        ));
    }
}

pub fn handle_disconnection(
    trigger: On<Add, Disconnected>,
    mut commands: Commands,
    debug_text: Query<Entity, With<ClientIdText>>,
    disconnected: Query<(Entity, &Disconnected)>,
) {
    commands.trigger(UpdateStatusMessage(format!(
        "Disconnected ({})",
        disconnected
            .get(trigger.entity)
            .map(|d| d.1.reason.as_ref())
            .unwrap_or(None)
            .unwrap_or(&"Unknown".to_string())
    )));
    for entity in debug_text.iter() {
        commands.entity(entity).despawn();
    }
}

// ── Ability HUD ──────────────────────────────────────────────────────────────

#[derive(Component)]
struct AbilitySlotNode(usize);

#[derive(Component)]
struct CooldownOverlay(usize);

fn spawn_ability_hud(mut commands: Commands) {
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            bottom: Val::Px(20.0),
            left: Val::Percent(50.0),
            flex_direction: FlexDirection::Row,
            column_gap: Val::Px(8.0),
            ..default()
        })
        .with_children(|parent| {
            for (i, label) in ["Q", "E"].iter().enumerate() {
                parent
                    .spawn((
                        AbilitySlotNode(i),
                        Node {
                            width: Val::Px(54.0),
                            height: Val::Px(54.0),
                            border: UiRect::all(Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            overflow: Overflow::clip(),
                            ..default()
                        },
                        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
                        BorderColor::all(Color::srgb(0.6, 0.6, 0.6)),
                    ))
                    .with_children(|slot| {
                        // cooldown overlay (grows from bottom)
                        slot.spawn((
                            CooldownOverlay(i),
                            Node {
                                position_type: PositionType::Absolute,
                                bottom: Val::Px(0.0),
                                left: Val::Px(0.0),
                                width: Val::Percent(100.0),
                                height: Val::Percent(0.0),
                                ..default()
                            },
                            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                        ));
                        // key label
                        slot.spawn((
                            Text(label.to_string()),
                            TextFont::from_font_size(16.0),
                            TextColor(Color::srgb(0.9, 0.9, 0.9)),
                        ));
                    });
            }
        });
}

fn update_ability_hud(
    player: Query<&AbilityLoadout, With<LocalPlayer>>,
    mut overlays: Query<(&CooldownOverlay, &mut Node)>,
) {
    let Ok(loadout) = player.single() else { return };

    // find max cooldown per slot from the AbilityDef — we approximate by
    // tracking the ratio: remaining / total is unknown without the registry
    // here, so we just show remaining seconds as a 0-5s bar (clamped).
    const MAX_CD: f32 = 5.0;

    for (overlay, mut node) in &mut overlays {
        let pct = loadout
            .slots
            .get(overlay.0)
            .map(|s| (s.cooldown_remaining / MAX_CD).clamp(0.0, 1.0) * 100.0)
            .unwrap_or(0.0);
        node.height = Val::Percent(pct);
    }
}

