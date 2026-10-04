//! The mode menu: pick Race, Survival or Puzzle, how many people play, and
//! how many AI wizards join.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase, TowerState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_menu)
            .add_systems(Update, (menu_keys, menu_gamepads, show_menu).chain());
    }
}

const MAX_TOWERS: u32 = crate::gameplay::MAX_TOWERS as u32;

#[derive(Component)]
struct MenuText;

/// The menu screen: the original's title art when staged (`menu_background`
/// filling the screen, `logo` above the options), else a dark panel.
fn spawn_menu(mut commands: Commands, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            flex_direction: FlexDirection::Column,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            row_gap: Val::Px(20.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.05, 0.06, 0.12, 0.85)),
        GlobalZIndex(10),
        MenuText,
    ));
    if let Some(background) = image("menu_background") {
        root.insert((ImageNode::new(background), BackgroundColor(Color::NONE)));
    }
    root.with_children(|parent| {
        if let Some(logo) = image("logo") {
            parent.spawn((
                ImageNode::new(logo),
                Node {
                    max_width: Val::Percent(50.0),
                    max_height: Val::Percent(30.0),
                    ..default()
                },
            ));
        }
        parent.spawn((
            Text::new(""),
            TextFont {
                font_size: FontSize::Px(26.0),
                ..default()
            },
            TextLayout::justify(Justify::Center),
            MenuLabel,
        ));
    });
}

/// The menu's options text.
#[derive(Component)]
struct MenuLabel;

/// Steps Easy, Normal, Pro, then round again.
fn next_difficulty(config: &mut GameConfig) {
    let now = config.difficulty.unwrap_or_default();
    let at = Difficulty::ALL.iter().position(|&d| d == now).unwrap_or(1);
    config.difficulty = Some(Difficulty::ALL[(at + 1) % Difficulty::ALL.len()]);
}

fn menu_keys(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut config: ResMut<GameConfig>,
    mut game: ResMut<Match>,
    towers: Query<(), With<TowerState>>,
    mut request: ResMut<MatchRequest>,
    mut left_from: Local<Option<Phase>>,
) {
    let Some(keys) = keys else { return };
    if keys.just_pressed(KeyCode::Escape) {
        match game.phase {
            Phase::Playing | Phase::Over => {
                *left_from = Some(game.phase);
                game.phase = Phase::Menu;
            }
            // Back to the match we left.
            Phase::Menu if !towers.is_empty() => {
                if let Some(phase) = left_from.take() {
                    game.phase = phase;
                }
            }
            Phase::Menu => {}
        }
        return;
    }
    if game.phase != Phase::Menu {
        return;
    }
    let index = ModeKind::ALL
        .iter()
        .position(|&m| m == config.mode)
        .unwrap_or(0);
    let count = ModeKind::ALL.len();
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW, KeyCode::Numpad8]) {
        config.mode = ModeKind::ALL[(index + count - 1) % count];
    }
    if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS, KeyCode::Numpad2]) {
        config.mode = ModeKind::ALL[(index + 1) % count];
    }
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA, KeyCode::Numpad4]) {
        config.opponents = config.opponents.saturating_sub(1);
    }
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD, KeyCode::Numpad6]) {
        config.opponents = (config.opponents + 1).min(MAX_TOWERS - config.players);
    }
    // Number keys pick how many people play (players 2 to 4 use gamepads).
    let digits = [
        KeyCode::Digit1,
        KeyCode::Digit2,
        KeyCode::Digit3,
        KeyCode::Digit4,
    ];
    if let Some(n) = digits.iter().position(|&k| keys.just_pressed(k)) {
        config.players = n as u32 + 1;
        config.opponents = config.opponents.min(MAX_TOWERS - config.players);
    }
    if keys.just_pressed(KeyCode::Tab) {
        next_difficulty(&mut config);
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        request.pending = true;
        *left_from = None;
    }
}

/// Any gamepad can pick the mode (d-pad) and start (A or Start).
fn menu_gamepads(
    mut config: ResMut<GameConfig>,
    game: Res<Match>,
    mut request: ResMut<MatchRequest>,
    gamepads: Query<&Gamepad>,
) {
    if game.phase != Phase::Menu {
        return;
    }
    let count = ModeKind::ALL.len();
    for pad in &gamepads {
        let index = ModeKind::ALL
            .iter()
            .position(|&m| m == config.mode)
            .unwrap_or(0);
        if pad.just_pressed(GamepadButton::DPadUp) {
            config.mode = ModeKind::ALL[(index + count - 1) % count];
        }
        if pad.just_pressed(GamepadButton::DPadDown) {
            config.mode = ModeKind::ALL[(index + 1) % count];
        }
        if pad.just_pressed(GamepadButton::West) {
            next_difficulty(&mut config);
        }
        if pad.any_just_pressed([GamepadButton::South, GamepadButton::Start]) {
            request.pending = true;
        }
    }
}

fn show_menu(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<(), With<TowerState>>,
    mut menu: Query<(&mut Node, &Children), With<MenuText>>,
    mut texts: Query<&mut Text, With<MenuLabel>>,
) {
    let Ok((mut node, children)) = menu.single_mut() else {
        return;
    };
    let visible = game.phase == Phase::Menu;
    node.display = if visible {
        Display::Flex
    } else {
        Display::None
    };
    if !visible {
        return;
    }
    let modes: Vec<String> = ModeKind::ALL
        .iter()
        .map(|&m| {
            let detail = match m {
                ModeKind::Race => {
                    format!("reach the line {:.0} bricks up", config.race.finish_height)
                }
                ModeKind::Survival => format!(
                    "place {} bricks, {} lives",
                    config.survival.bricks_to_place, config.survival.lives
                ),
                ModeKind::Puzzle => format!("fit {} bricks under the laser", config.puzzle.bricks),
                ModeKind::Endless => "stay above the rising line".into(),
            };
            if m == config.mode {
                format!("> {}  ({detail}) <", m.name())
            } else {
                format!("{}  ({detail})", m.name())
            }
        })
        .collect();
    let resume = if towers.is_empty() {
        ""
    } else {
        "\nEsc: back to the game"
    };
    let text = format!(
        "TRICKY TOWERS\n\n{}\n\nDifficulty: {}\nPlayers: {}   AI wizards: < {} >\n\nUp/Down: mode   Tab: difficulty   1-4: players   Left/Right: AI wizards   Enter: play\nPlayer 1: keyboard   Players 2-4: gamepads\n\nIn game: Left/Right move   Shift+Left/Right nudge   Up/Z rotate   Down drop\nQ light spell   E dark spell   R restart   Esc menu{resume}",
        modes.join("\n"),
        config.difficulty.unwrap_or_default().name(),
        config.players,
        config.opponents
    );
    for &child in children {
        if let Ok(mut t) = texts.get_mut(child) {
            t.0 = text.clone();
        }
    }
}
