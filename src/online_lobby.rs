//! The Online Battle lobby (the original's OnlineLobby scene and OnlineLobbyStateController),
//! opened by Find Online Cup, Find Online Game and Play with friends. There is no online play,
//! so it stays where the original's lobby is before it connects: four player slots, each
//! "Looking for games" (or "Creating game" when inviting friends) over a turning brick.
//! Back returns to the Online Battle screen.

use bevy::prelude::*;

use crate::art::Art;
use crate::gameplay::{Match, Phase};
use crate::menu::MenuScreen;
use crate::select::{PANEL_SLICE, SCREEN, SEAT_PANEL, label_with, place, shown, sliced};

pub struct OnlineLobbyPlugin;

impl Plugin for OnlineLobbyPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<OnlineLobby>()
            .add_systems(Startup, (spawn_lobby, spawn_invite))
            .add_systems(Update, (show_lobby, show_invite).after(crate::menu::MenuSystems));
    }
}

/// Which way into the lobby: inviting friends (Play with friends) or finding a game.
#[derive(Resource, Default, Debug)]
pub struct OnlineLobby {
    pub invite: bool,
    /// Seconds the lobby has been up, for the spinners.
    time: f32,
}

#[derive(Component)]
struct LobbyRoot;

/// What a lobby node shows: the header title, or a slot's searching words.
#[derive(Component, Clone, Copy, PartialEq)]
enum LobbyPart {
    Title,
    Searching,
    Spinner,
}

/// OnlinePlayerSlot1..4: columns from 5% to 95% of the width (as Select Players), the
/// Searching panel (the character panel's place, 179 px up) with its words 29.5 px up and
/// the spinner brick 55 px down.
const SLOT_Y: f32 = 179.0;
const WORDS_Y: f32 = 29.5;
const WORDS_BOX: (f32, f32) = (SEAT_PANEL.0 - 60.0, 110.0);
const SPINNER_Y: f32 = -55.0;
const SPINNER: (f32, f32) = (55.0, 38.0);

fn slot_x(slot: usize) -> f32 {
    (0.05 + 0.225 * (slot as f32 + 0.5)) * SCREEN.0 - SCREEN.0 / 2.0
}

/// The header's words: STATE_TITLE_LOBBY, or LOBBY_HEADER_INVITE_FRIENDS when inviting.
fn title(invite: bool) -> &'static str {
    if invite {
        "Invite friends"
    } else {
        "Find Players"
    }
}

/// A slot before the lobby connects: LOBBY_LOOKING_FOR_GAMES, or LOBBY_CREATING_GAME for
/// the one inviting.
fn searching(invite: bool) -> &'static str {
    if invite {
        "Creating game"
    } else {
        "Looking for\ngames"
    }
}

fn spawn_lobby(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let page = crate::select::spawn_page(&mut commands, art, "");
    commands.entity(page).insert(LobbyRoot);
    commands.entity(page).with_children(|page| {
        // The header's own title, which changes with the way in.
        page.spawn(place(
            crate::select::HEADER,
            (0.0, (SCREEN.1 - crate::select::HEADER.1) / 2.0),
        ))
        .with_children(|h| {
            h.spawn((
                label_with(60.0 / 10.8, title(false), crate::letters::TextLook::TITLE),
                crate::letters::LetterSpacing(-1.0),
                LobbyPart::Title,
            ));
        });
        for slot in 0..4 {
            let x = slot_x(slot);
            let mut panel = page.spawn(place(SEAT_PANEL, (x, SLOT_Y)));
            if let Some(back) = image("menu_player_select_background_deselected") {
                panel.insert(sliced(back, [PANEL_SLICE; 4]));
            }
            page.spawn(place(WORDS_BOX, (x, SLOT_Y + WORDS_Y)))
                .with_children(|w| {
                    w.spawn((
                        label_with(
                            26.0 / 10.8,
                            searching(false),
                            crate::letters::TextLook::TITLE,
                        ),
                        LobbyPart::Searching,
                    ));
                });
            if let Some(brick) = image("next_brick_default_J") {
                page.spawn((
                    ImageNode::new(brick),
                    place(SPINNER, (x, SLOT_Y + SPINNER_Y)),
                    LobbyPart::Spinner,
                    UiTransform::default(),
                ));
            }
        }
    });
}

/// The Spinner: a quick 15 degree wind-up (0.2 s), a quarter turn back past it (0.45 s,
/// both in-out elastic), then 0.6 s still; a quarter turn every 1.25 s.
fn spinner_angle(t: f32) -> f32 {
    let cycle = 1.25;
    let turns = (t / cycle).floor();
    let at = t - turns * cycle;
    let step = if at < 0.2 {
        15.0 * crate::gameplay::in_out_elastic(at / 0.2)
    } else if at < 0.65 {
        15.0 - 105.0 * crate::gameplay::in_out_elastic((at - 0.2) / 0.45)
    } else {
        -90.0
    };
    -90.0 * turns + step
}

#[allow(clippy::type_complexity)]
fn show_lobby(
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut lobby: ResMut<OnlineLobby>,
    mut roots: Query<&mut Visibility, With<LobbyRoot>>,
    mut parts: Query<(
        &LobbyPart,
        Option<&mut Text>,
        &mut Node,
        Option<&mut UiTransform>,
    )>,
    mut was: Local<Option<bool>>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Lobby;
    for mut v in &mut roots {
        v.set_if_neq(shown(on));
    }
    if !on {
        *was = None;
        return;
    }
    lobby.time += time.delta_secs();
    let fresh = *was != Some(lobby.invite);
    if fresh {
        lobby.time = 0.0;
        *was = Some(lobby.invite);
    }
    let angle = spinner_angle(lobby.time);
    for (part, text, _, ui) in &mut parts {
        match part {
            LobbyPart::Title | LobbyPart::Searching if fresh => {
                let words = if *part == LobbyPart::Title {
                    title(lobby.invite)
                } else {
                    searching(lobby.invite)
                };
                if let Some(mut text) = text {
                    text.0 = words.to_uppercase();
                }
            }
            LobbyPart::Spinner => {
                if let Some(mut ui) = ui {
                    // Unity turns anticlockwise, Bevy's UI clockwise.
                    ui.rotation = Rot2::degrees(-angle);
                }
            }
            _ => {}
        }
    }
}

#[derive(Component)]
struct InviteRoot;
#[derive(Component)]
struct InviteSpinner;

/// AcceptOnlineInvite: the modal tile over the whole screen, "Accepting Invite..." in the
/// middle (EnzoBlackMilkyWhite46) and the spinner brick 100 px in from the bottom right.
fn spawn_invite(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgb(0.12, 0.1, 0.2)),
        GlobalZIndex(23),
        Visibility::Hidden,
        InviteRoot,
    ));
    if let Some(tile) = image("modal_tile") {
        root.insert((
            ImageNode::new(tile).with_mode(NodeImageMode::Tiled {
                tile_x: true,
                tile_y: true,
                stretch_value: 1.0,
            }),
            BackgroundColor(Color::NONE),
        ));
    }
    root.with_children(|r| {
        r.spawn(place((SCREEN.0, 46.0), (0.0, 0.0))).with_child(label_with(
            46.0 / 10.8,
            "Accepting Invite...",
            crate::letters::TextLook::TITLE,
        ));
        if let Some(brick) = image("next_brick_default_J") {
            r.spawn((
                ImageNode::new(brick),
                place(SPINNER, (SCREEN.0 / 2.0 - 100.0, -SCREEN.1 / 2.0 + 100.0)),
                UiTransform::default(),
                InviteSpinner,
            ));
        }
    });
}

fn show_invite(
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut roots: Query<&mut Visibility, With<InviteRoot>>,
    mut spinners: Query<&mut UiTransform, With<InviteSpinner>>,
    mut since: Local<f32>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Invite;
    for mut v in &mut roots {
        v.set_if_neq(shown(on));
    }
    if !on {
        *since = 0.0;
        return;
    }
    *since += time.delta_secs();
    for mut ui in &mut spinners {
        ui.rotation = Rot2::degrees(-spinner_angle(*since));
    }
}

#[cfg(test)]
mod tests {
    use super::spinner_angle;

    #[test]
    fn the_spinner_turns_a_quarter_every_cycle() {
        assert!(spinner_angle(0.0).abs() < 1e-3);
        assert!((spinner_angle(1.0) + 90.0).abs() < 1e-3);
        assert!((spinner_angle(2.25) + 180.0).abs() < 1e-3);
    }
}
