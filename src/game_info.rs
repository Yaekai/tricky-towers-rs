//! The Game Info popup, opened in a match with the Game info key (Tab) or a pad's Y,
//! laid out from the PC readout (docs/research/gap-scan.md, section 7): a 750×550
//! panel 265 px from the top (so centred), "Game Info" (size 46), "Game Mode:" (22)
//! with the mode's name (32), then a row per player with the wizard's picture, the
//! player's name (26) and wizard level (18). Online players who left show a cross;
//! there's no online play here.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::cups::XpBook;
use crate::gameplay::{Match, Phase, Tower, TowerSkin};
use crate::keybinding::{Action, KeyBindings};
use crate::select::{SCREEN, label, place, shown, sliced};

pub struct GameInfoPlugin;

impl Plugin for GameInfoPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameInfo>()
            .add_systems(Startup, spawn_game_info)
            .add_systems(Update, (toggle_game_info, show_game_info).chain());
    }
}

/// Whether the popup is open.
#[derive(Resource, Debug, Default)]
pub struct GameInfo {
    pub open: bool,
}

/// Layout in px of a 1920×1080 screen from its centre (y up), from GameInfoPopup and
/// GameInfoItem: the panel hangs 265 px under the screen's top and is 80 px a player
/// plus 230 tall (550 for 4); the title's middle sits 7 px over its top, "Game Mode:" 69.2
/// and the mode name 103.3 px under it; rows are 630x65, 80 px apart from 160 px under
/// the top, with a 48 px avatar 13 px in and the text 155 px in, the name at the top
/// and the level at the bottom, 5 px from the row's edges.
const PANEL: (f32, f32) = (750.0, 550.0);
const PANEL_TOP: f32 = 265.0;
const TITLE: (f32, f32) = (282.0, 46.0);
const MODE_LABEL: (f32, f32) = (205.8, 22.3);
const MODE_NAME: (f32, f32) = (171.7, 32.0);
const ROW: (f32, f32) = (630.0, 65.0);
const FIRST_ROW: f32 = 82.5;
const ROW_STEP: f32 = 80.0;
const AVATAR: f32 = 48.0;
const AVATAR_LEFT: f32 = 13.0;
const TEXT_LEFT: f32 = 155.0;
const TEXT_INSET: f32 = 5.0;
const NAME_SIZE: f32 = 26.0;
const LEVEL_SIZE: f32 = 18.0;
const ROWS: usize = 4;

fn panel_y() -> f32 {
    SCREEN.1 / 2.0 - PANEL_TOP - PANEL.1 / 2.0
}

fn vh(px: f32) -> f32 {
    px / SCREEN.1 * 100.0
}

#[derive(Component)]
struct GameInfoRoot;

#[derive(Component, Clone, Copy, PartialEq)]
enum InfoPart {
    Mode,
    Row(usize),
    Avatar(usize),
    Name(usize),
    Level(usize),
}

fn spawn_game_info(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let y = panel_y();
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            GlobalZIndex(14),
            Visibility::Hidden,
            GameInfoRoot,
        ))
        .with_children(|root| {
            let mut panel = root.spawn((
                place(PANEL, (0.0, y)),
                BackgroundColor(Color::srgba(0.12, 0.1, 0.25, 0.95)),
            ));
            // The Endless leaderboard's panel (gph_popupBackgroundPanel), 9-sliced.
            if let Some(back) = image("menu_leaderboard_panel") {
                panel.insert((
                    sliced(back, [40.0, 40.0, 37.0, 51.0]),
                    BackgroundColor(Color::NONE),
                ));
            }
            let line = |root: &mut ChildSpawnerCommands,
                        (dy, size): (f32, f32),
                        words: &str,
                        part: Option<InfoPart>| {
                root.spawn(place((PANEL.0, size * 1.4), (0.0, y + dy)))
                    .with_children(|t| {
                        let mut text = t.spawn(label(vh(size), words));
                        if let Some(part) = part {
                            text.insert(part);
                        }
                    });
            };
            line(root, TITLE, "Game Info", None);
            line(root, MODE_LABEL, "Game Mode:", None);
            line(root, MODE_NAME, "", Some(InfoPart::Mode));
            for i in 0..ROWS {
                let row_y = y + FIRST_ROW - ROW_STEP * i as f32;
                let left = -ROW.0 / 2.0;
                root.spawn((
                    place(ROW, (0.0, row_y)),
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
                    InfoPart::Row(i),
                ));
                root.spawn((
                    place((AVATAR, AVATAR), (left + AVATAR_LEFT + AVATAR / 2.0, row_y)),
                    ImageNode {
                        color: Color::NONE,
                        ..default()
                    },
                    InfoPart::Avatar(i),
                ));
                let text_x = left + TEXT_LEFT;
                let text_w = ROW.0 - 175.0;
                let edge = ROW.1 / 2.0 - TEXT_INSET;
                for (dy, size, part) in [
                    (edge - NAME_SIZE * 0.7, NAME_SIZE, InfoPart::Name(i)),
                    (-edge + LEVEL_SIZE * 0.7, LEVEL_SIZE, InfoPart::Level(i)),
                ] {
                    root.spawn(Node {
                        justify_content: JustifyContent::FlexStart,
                        ..place((text_w, size * 1.4), (text_x + text_w / 2.0, row_y + dy))
                    })
                    .with_children(|t| {
                        t.spawn((label(vh(size), ""), part));
                    });
                }
            }
        });
}

/// Tab (the Game info key) or a pad's Y opens and closes it during a match; it
/// closes by itself when the match ends or the game goes to the menu.
fn toggle_game_info(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    bindings: Option<Res<KeyBindings>>,
    pads: Query<&Gamepad>,
    game: Res<Match>,
    mut info: ResMut<GameInfo>,
) {
    if game.phase != Phase::Playing {
        info.open = false;
        return;
    }
    let key = bindings
        .as_ref()
        .map_or(Action::GameInfo.default_key(), |b| b.key(Action::GameInfo));
    let pressed = keys.is_some_and(|k| k.just_pressed(key))
        || pads.iter().any(|p| p.just_pressed(GamepadButton::North));
    if pressed {
        info.open = !info.open;
    }
}

/// The player's name on the row (?: the original shows profile names).
fn player_name(tower: &Tower) -> String {
    match tower.player {
        Some(p) => format!("Player {}", p + 1),
        None => "Computer".to_string(),
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn show_game_info(
    info: Res<GameInfo>,
    game: Res<Match>,
    config: Res<GameConfig>,
    xp: Option<Res<XpBook>>,
    art: Option<Res<Art>>,
    towers: Query<(&Tower, Option<&TowerSkin>)>,
    mut root: Query<&mut Visibility, With<GameInfoRoot>>,
    mut parts: Query<(&InfoPart, &mut Visibility, Option<&mut ImageNode>), Without<GameInfoRoot>>,
    mut texts: Query<(&InfoPart, &mut Text)>,
) {
    for mut v in &mut root {
        v.set_if_neq(shown(info.open));
    }
    if !info.open {
        return;
    }
    let mut list: Vec<(&Tower, Option<&TowerSkin>)> = towers.iter().collect();
    list.sort_by_key(|(t, _)| t.index);
    for (part, mut v, image) in &mut parts {
        if let InfoPart::Row(i) | InfoPart::Avatar(i) = *part {
            v.set_if_neq(shown(i < list.len()));
        }
        // The row's wizard picture (`wizard_N`, the game-info icons), if staged.
        if let (InfoPart::Avatar(i), Some(mut image)) = (*part, image) {
            let wizard = list
                .get(i)
                .map_or(i, |(t, skin)| skin.map_or(t.index, |s| s.wizard));
            let picture = art
                .as_deref()
                .and_then(|a| a.image(&format!("wizard_{}", wizard + 1)));
            match picture {
                Some(picture) if image.image != picture => {
                    image.image = picture;
                    image.color = Color::WHITE;
                }
                None if image.color != Color::NONE => image.color = Color::NONE,
                _ => {}
            }
        }
    }
    for (part, mut text) in &mut texts {
        let words = match *part {
            InfoPart::Mode => game.mode.name().to_string(),
            InfoPart::Name(i) => list.get(i).map_or(String::new(), |(t, _)| player_name(t)),
            InfoPart::Level(i) => match list.get(i) {
                Some((tower, _)) => {
                    let points = tower
                        .player
                        .and_then(|p| xp.as_ref().and_then(|x| x.players.get(p).copied()))
                        .unwrap_or(0);
                    let level = config.xp.level(points);
                    format!("Level {level}: {}", config.xp.name(level))
                }
                None => String::new(),
            },
            _ => continue,
        };
        if text.0 != words {
            text.0 = words;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_panel_is_265_px_from_the_top() {
        let top = panel_y() + PANEL.1 / 2.0;
        assert_eq!(SCREEN.1 / 2.0 - top, PANEL_TOP);
    }
}
