//! The mode menu: pick Race, Survival or Puzzle and how many AI wizards to
//! play against.

use bevy::prelude::*;

use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase, TowerState};

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_menu)
            .add_systems(Update, (menu_keys, show_menu).chain());
    }
}

const MAX_OPPONENTS: u32 = 3;

#[derive(Component)]
struct MenuText;

fn spawn_menu(mut commands: Commands) {
    commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.06, 0.12, 0.85)),
            GlobalZIndex(10),
            MenuText,
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(26.0),
                    ..default()
                },
                TextLayout::justify(Justify::Center),
            ));
        });
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
    if keys.any_just_pressed([KeyCode::ArrowUp, KeyCode::KeyW]) {
        config.mode = ModeKind::ALL[(index + count - 1) % count];
    }
    if keys.any_just_pressed([KeyCode::ArrowDown, KeyCode::KeyS]) {
        config.mode = ModeKind::ALL[(index + 1) % count];
    }
    if keys.any_just_pressed([KeyCode::ArrowLeft, KeyCode::KeyA]) {
        config.opponents = config.opponents.saturating_sub(1);
    }
    if keys.any_just_pressed([KeyCode::ArrowRight, KeyCode::KeyD]) {
        config.opponents = (config.opponents + 1).min(MAX_OPPONENTS);
    }
    if keys.any_just_pressed([KeyCode::Enter, KeyCode::Space]) {
        request.pending = true;
        *left_from = None;
    }
}

fn show_menu(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<(), With<TowerState>>,
    mut menu: Query<(&mut Node, &Children), With<MenuText>>,
    mut texts: Query<&mut Text>,
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
        "TRICKY TOWERS\n\n{}\n\nOpponents: < {} >\n\nUp/Down: mode   Left/Right: opponents   Enter: play{resume}",
        modes.join("\n"),
        config.opponents
    );
    if let Some(&child) = children.first()
        && let Ok(mut t) = texts.get_mut(child)
    {
        t.0 = text;
    }
}
