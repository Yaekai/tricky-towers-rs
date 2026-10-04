//! The "Game Completed" popup after a match: each player's place and the
//! original's three buttons, Next match, Rematch and Main menu.
//!
//! The original also counts up the XP gained and says "Level Up!" or "Level
//! Down!"; there are no wizard levels here yet, so those lines are left out.

use bevy::prelude::*;

use crate::config::GameConfig;
use crate::gameplay::{Match, MatchRequest, Phase, Tower, TowerState, placings};
use crate::menu::{MenuScreen, MenuSystems};

pub struct PopupPlugin;

impl Plugin for PopupPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Popup>()
            .add_systems(Startup, spawn_popup)
            .add_systems(
                Update,
                (time_popup, popup_input, show_popup)
                    .chain()
                    .after(MenuSystems),
            );
    }
}

/// The popup's buttons, left to right.
pub const POPUP_BUTTONS: [&str; 3] = ["Next match", "Rematch", "Main menu"];

/// Whether the popup is up, and its highlighted button.
#[derive(Resource, Default)]
pub struct Popup {
    pub shown: bool,
    pub selected: usize,
    /// Seconds since the match ended.
    clock: f32,
    round: u64,
}

#[derive(Component)]
struct PopupRoot;
#[derive(Component)]
struct PopupPlaces;
#[derive(Component)]
struct PopupButton(usize);

/// "1st Place" to "4th Place".
pub fn place_name(place: usize) -> String {
    let suffix = match place {
        1 => "st",
        2 => "nd",
        3 => "rd",
        _ => "th",
    };
    format!("{place}{suffix} Place")
}

fn spawn_popup(mut commands: Commands) {
    let text = |size: f32, value: &str| {
        (
            Text::new(value),
            TextFont {
                font_size: FontSize::Vh(size),
                ..default()
            },
            TextColor(Color::WHITE),
            TextLayout::justify(Justify::Center),
        )
    };
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(21),
            Visibility::Hidden,
            PopupRoot,
        ))
        .with_children(|root| {
            root.spawn((
                Node {
                    flex_direction: FlexDirection::Column,
                    align_items: AlignItems::Center,
                    row_gap: Val::Vh(2.5),
                    padding: UiRect::axes(Val::Vh(6.0), Val::Vh(4.0)),
                    ..default()
                },
                BackgroundColor(Color::srgba(0.08, 0.06, 0.14, 0.9)),
            ))
            .with_children(|panel| {
                panel.spawn(text(6.0, "Game Completed"));
                panel.spawn((text(4.5, ""), PopupPlaces));
                panel
                    .spawn(Node {
                        column_gap: Val::Vh(4.0),
                        ..default()
                    })
                    .with_children(|row| {
                        for (i, label) in POPUP_BUTTONS.iter().enumerate() {
                            row.spawn((text(3.5, label), PopupButton(i)));
                        }
                    });
            });
        });
}

/// Puts the popup up a while after the match ends, and takes it down when a
/// new match starts or the menu opens.
fn time_popup(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut popup: ResMut<Popup>,
) {
    if game.phase != Phase::Over || popup.round != game.round {
        popup.round = game.round;
        popup.clock = 0.0;
        popup.shown = false;
        return;
    }
    popup.clock += time.delta_secs();
    if !popup.shown && popup.clock >= config.moments.popup_after {
        popup.shown = true;
        popup.selected = 0;
    }
}

/// Left/Right (or the d-pad) pick a button, Enter or A takes it. Next match
/// and Rematch both start the same mode again (there are no cups between
/// matches yet); Main menu goes back to the title tiles.
fn popup_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    gamepads: Query<&Gamepad>,
    mut popup: ResMut<Popup>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut request: ResMut<MatchRequest>,
) {
    if !popup.shown || game.phase != Phase::Over {
        return;
    }
    let count = POPUP_BUTTONS.len();
    let (mut left, mut right, mut pick) = (false, false, false);
    if let Some(keys) = keys {
        left |= keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::Numpad4]);
        right |= keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::Numpad6]);
        pick |= keys.any_just_pressed([KeyCode::Enter, KeyCode::Space, KeyCode::NumpadEnter]);
    }
    for pad in &gamepads {
        left |= pad.just_pressed(GamepadButton::DPadLeft);
        right |= pad.just_pressed(GamepadButton::DPadRight);
        pick |= pad.just_pressed(GamepadButton::South);
    }
    if left {
        popup.selected = (popup.selected + count - 1) % count;
    }
    if right {
        popup.selected = (popup.selected + 1) % count;
    }
    if !pick {
        return;
    }
    match POPUP_BUTTONS[popup.selected] {
        "Main menu" => {
            game.phase = Phase::Menu;
            *screen = MenuScreen::Main;
        }
        _ => request.pending = true,
    }
    popup.shown = false;
}

fn show_popup(
    popup: Res<Popup>,
    game: Res<Match>,
    towers: Query<(&Tower, &TowerState)>,
    mut root: Query<&mut Visibility, With<PopupRoot>>,
    mut places: Query<&mut Text, With<PopupPlaces>>,
    mut buttons: Query<(&PopupButton, &mut TextColor)>,
) {
    for mut v in &mut root {
        v.set_if_neq(if popup.shown {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    if !popup.shown {
        return;
    }
    let list: Vec<_> = towers.iter().map(|(t, s)| (t.index, s)).collect();
    let order = placings(&game, &list);
    let mut humans: Vec<(usize, usize)> = towers
        .iter()
        .filter_map(|(t, _)| t.player.map(|p| (p, t.index)))
        .collect();
    humans.sort();
    let lines: Vec<String> = humans
        .iter()
        .map(|&(player, index)| {
            let place = order.iter().position(|&i| i == index).unwrap_or(0) + 1;
            if humans.len() == 1 {
                place_name(place)
            } else {
                format!("Player {}: {}", player + 1, place_name(place))
            }
        })
        .collect();
    let text = lines.join("\n");
    for mut t in &mut places {
        if t.0 != text {
            t.0 = text.clone();
        }
    }
    for (button, mut color) in &mut buttons {
        let want = if button.0 == popup.selected {
            Color::srgb(1.0, 0.82, 0.25)
        } else {
            Color::WHITE
        };
        if color.0 != want {
            color.0 = want;
        }
    }
}
