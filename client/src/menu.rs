use std::net::SocketAddr;

use bevy::prelude::*;
use bevy_inspector_egui::bevy_egui::{egui, EguiContexts, EguiPrimaryContextPass};
use lightyear::connection::client::Connected;
use lightyear::prelude::client::*;
use lightyear::prelude::*;
use protocol::{CharacterKey, LobbyChannel, SelectCharacter};
use shared::{CLIENT_PORT, SHARED_SETTINGS};

use crate::setup::{BattleArenaClient, ClientTransports};

#[derive(States, Clone, Debug, Default, PartialEq, Eq, Hash)]
pub enum AppState {
    #[default]
    MainMenu,
    InGame,
}

#[derive(Resource)]
pub struct MenuState {
    pub server_addr: String,
    pub selected_char: CharacterKey,
    pub addr_error: Option<String>,
}

impl Default for MenuState {
    fn default() -> Self {
        Self {
            server_addr: "127.0.0.1:5888".to_string(),
            selected_char: CharacterKey::Comini,
            addr_error: None,
        }
    }
}

#[derive(Resource, Default)]
struct PendingConnect(bool);

#[derive(Resource, Default)]
struct PendingCharacterSelect(Option<CharacterKey>);

#[derive(Resource, Default)]
pub struct PauseMenuOpen(pub bool);

#[derive(Resource, Default)]
struct PendingExit(bool);

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<AppState>();
        app.init_resource::<MenuState>();
        app.init_resource::<PendingConnect>();
        app.init_resource::<PendingCharacterSelect>();
        app.init_resource::<PauseMenuOpen>();
        app.init_resource::<PendingExit>();

        // Egui systems must live in EguiPrimaryContextPass — EguiPlugin::default()
        // uses multi-pass mode where begin_pass is only called inside
        // run_egui_context_pass_loop_system (PostUpdate).
        app.add_systems(
            EguiPrimaryContextPass,
            (
                show_menu.run_if(in_state(AppState::MainMenu)),
                show_pause_menu.run_if(in_state(AppState::InGame)),
            ),
        );

        app.add_systems(
            Update,
            (
                try_connect.run_if(in_state(AppState::MainMenu)),
                send_initial_character_selection.run_if(in_state(AppState::InGame)),
                toggle_pause.run_if(in_state(AppState::InGame)),
            ),
        );

        app.add_systems(OnEnter(AppState::MainMenu), reset_on_main_menu);

        app.add_observer(on_connected);
        app.add_observer(on_disconnected_for_exit);
    }
}

// ── Main menu ────────────────────────────────────────────────────────────────

fn show_menu(
    mut contexts: EguiContexts,
    mut menu: ResMut<MenuState>,
    mut pending: ResMut<PendingConnect>,
    mut commands: Commands,
) {
    let Ok(ctx) = contexts.ctx_mut() else { return };
    egui::CentralPanel::default().show(ctx, |ui| {
        if pending.0 {
            ui.vertical_centered(|ui| {
                ui.add_space(200.0);
                ui.label(egui::RichText::new("Connecting...").size(32.0));
            });
            return;
        }

        ui.vertical_centered(|ui| {
            ui.add_space(80.0);
            ui.label(egui::RichText::new("BATTLE ARENA").size(56.0).strong());
            ui.add_space(50.0);

            ui.label(egui::RichText::new("Server Address").size(14.0));
            ui.add(
                egui::TextEdit::singleline(&mut menu.server_addr)
                    .desired_width(250.0)
                    .hint_text("127.0.0.1:5888"),
            );
            if let Some(ref err) = menu.addr_error.clone() {
                ui.colored_label(egui::Color32::RED, err);
            }

            ui.add_space(24.0);
            ui.label(egui::RichText::new("Character").size(14.0));
            ui.horizontal(|ui| {
                let btn_width = 110.0;
                let total = btn_width * 2.0 + 12.0;
                let available = ui.available_width();
                if available > total {
                    ui.add_space((available - total) / 2.0);
                }
                if ui
                    .add_sized(
                        [btn_width, 40.0],
                        egui::Button::selectable(menu.selected_char == CharacterKey::Comini, "Comini"),
                    )
                    .clicked()
                {
                    menu.selected_char = CharacterKey::Comini;
                }
                ui.add_space(12.0);
                if ui
                    .add_sized(
                        [btn_width, 40.0],
                        egui::Button::selectable(menu.selected_char == CharacterKey::Kaps, "Kaps"),
                    )
                    .clicked()
                {
                    menu.selected_char = CharacterKey::Kaps;
                }
            });

            ui.add_space(32.0);
            if ui
                .add_sized(
                    [200.0, 50.0],
                    egui::Button::new(egui::RichText::new("PLAY").size(22.0)),
                )
                .clicked()
            {
                match menu.server_addr.parse::<SocketAddr>() {
                    Ok(addr) => {
                        menu.addr_error = None;
                        let client_id = std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .map(|d| d.as_micros() as u64)
                            .unwrap_or(1);
                        commands.spawn(BattleArenaClient {
                            client_id,
                            client_port: CLIENT_PORT,
                            server_addr: addr,
                            conditioner: None,
                            transport: ClientTransports::Udp,
                            shared: SHARED_SETTINGS,
                        });
                        pending.0 = true;
                    }
                    Err(_) => {
                        menu.addr_error =
                            Some("Invalid address — expected format: 127.0.0.1:5888".to_string());
                    }
                }
            }

            ui.add_space(8.0);
            let _ = ui.add_sized(
                [200.0, 40.0],
                egui::Button::new(egui::RichText::new("OPTIONS").size(18.0)),
            );
        });
    });
}

fn try_connect(
    mut pending: ResMut<PendingConnect>,
    client: Option<Single<Entity, With<Client>>>,
    mut commands: Commands,
) {
    if !pending.0 {
        return;
    }
    let Some(client_entity) = client else {
        return;
    };
    pending.0 = false;
    commands.trigger(Connect {
        entity: client_entity.into_inner(),
    });
}

fn on_connected(
    _trigger: On<Add, Connected>,
    menu: Res<MenuState>,
    mut pending: ResMut<PendingCharacterSelect>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    pending.0 = Some(menu.selected_char.clone());
    next_state.set(AppState::InGame);
}

fn send_initial_character_selection(
    mut pending: ResMut<PendingCharacterSelect>,
    mut sender: Query<&mut MessageSender<SelectCharacter>>,
) {
    if pending.0.is_none() {
        return;
    }
    let Ok(mut sender) = sender.single_mut() else {
        return;
    };
    let Some(key) = pending.0.take() else {
        return;
    };
    sender.send::<LobbyChannel>(SelectCharacter { key });
}

// ── In-game pause menu ───────────────────────────────────────────────────────

fn toggle_pause(keyboard: Res<ButtonInput<KeyCode>>, mut pause_open: ResMut<PauseMenuOpen>) {
    if keyboard.just_pressed(KeyCode::Escape) {
        pause_open.0 = !pause_open.0;
    }
}

fn show_pause_menu(
    mut contexts: EguiContexts,
    mut pause_open: ResMut<PauseMenuOpen>,
    client: Option<Single<Entity, With<Client>>>,
    mut pending_exit: ResMut<PendingExit>,
    mut commands: Commands,
) {
    if !pause_open.0 {
        return;
    }

    let Ok(ctx) = contexts.ctx_mut() else { return };

    // Dim background
    egui::Area::new(egui::Id::new("pause_backdrop"))
        .fixed_pos(egui::pos2(0.0, 0.0))
        .order(egui::Order::Background)
        .show(ctx, |ui| {
            let screen = ui.ctx().content_rect();
            ui.painter()
                .rect_filled(screen, 0.0, egui::Color32::from_black_alpha(160));
        });

    egui::Window::new("##pause_menu")
        .title_bar(false)
        .resizable(false)
        .collapsible(false)
        .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
        .min_width(220.0)
        .show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.add_space(12.0);
                ui.label(egui::RichText::new("PAUSED").size(28.0).strong());
                ui.add_space(20.0);

                let _ = ui.add_sized(
                    [180.0, 44.0],
                    egui::Button::new(egui::RichText::new("Options").size(18.0)),
                );

                ui.add_space(8.0);

                if ui
                    .add_sized(
                        [180.0, 44.0],
                        egui::Button::new(egui::RichText::new("Exit").size(18.0)),
                    )
                    .clicked()
                {
                    pause_open.0 = false;
                    if let Some(client_entity) = client {
                        commands.trigger(Disconnect {
                            entity: client_entity.into_inner(),
                        });
                    }
                    pending_exit.0 = true;
                }

                ui.add_space(12.0);
            });
        });
}

fn on_disconnected_for_exit(
    trigger: On<Add, Disconnected>,
    pending_exit: Res<PendingExit>,
    mut next_state: ResMut<NextState<AppState>>,
    mut commands: Commands,
) {
    if !pending_exit.0 {
        return;
    }
    commands.entity(trigger.entity).despawn();
    next_state.set(AppState::MainMenu);
}

fn reset_on_main_menu(
    mut pause_open: ResMut<PauseMenuOpen>,
    mut pending_exit: ResMut<PendingExit>,
    mut pending_connect: ResMut<PendingConnect>,
    mut pending_select: ResMut<PendingCharacterSelect>,
) {
    pause_open.0 = false;
    pending_exit.0 = false;
    pending_connect.0 = false;
    pending_select.0 = None;
}
