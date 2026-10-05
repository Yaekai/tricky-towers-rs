//! Player 1's statistics and the Stats popup from Settings, laid out from the PC readout
//! (docs/research/gap-scan.md, section 7): two panels side by side over a dark modal,
//! Game stats (730×900: per mode, games and wins in columns Easy / Normal / Pro / Total
//! with their small icons) and Other stats (730×695: cups, time played, high scores),
//! with "BACK" under them. Kept between runs in `stats.ron`.
//!
//! What counts: local battles only (not trials or Endless runs), from player 1's tower.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, Phase, Tower, TowerState, TowerStatus};
use crate::select::{SCREEN, label, place, shown, sliced};

pub struct StatsPlugin;

impl Plugin for StatsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Stats>()
            .init_resource::<StatsMenu>()
            .add_systems(Startup, spawn_stats_popup)
            .add_systems(Update, (count_stats, show_stats_popup));
    }
}

/// The battle modes the Game stats panel lists, top first.
pub const MODES: [ModeKind; 3] = [ModeKind::Race, ModeKind::Survival, ModeKind::Puzzle];
pub const DIFFICULTIES: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Normal, Difficulty::Pro];

/// Games and wins in one mode at one difficulty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub mode: ModeKind,
    pub difficulty: Difficulty,
    pub games: u32,
    pub wins: u32,
}

/// Player 1's statistics. Kept in `file` when one is set (the game sets it; tests and
/// offscreen pictures don't).
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Stats {
    pub records: Vec<Record>,
    pub cups_won: u32,
    pub seconds_played: f32,
    #[serde(skip)]
    pub file: Option<PathBuf>,
    #[serde(skip)]
    counted: u64,
    #[serde(skip)]
    cup_counted: bool,
}

impl Stats {
    /// Games and wins in `mode` at `difficulty`, or over every difficulty with `None`.
    pub fn get(&self, mode: ModeKind, difficulty: Option<Difficulty>) -> (u32, u32) {
        self.records
            .iter()
            .filter(|r| r.mode == mode && difficulty.is_none_or(|d| r.difficulty == d))
            .fold((0, 0), |(g, w), r| (g + r.games, w + r.wins))
    }

    pub fn add(&mut self, mode: ModeKind, difficulty: Difficulty, won: bool) {
        let at = match self
            .records
            .iter()
            .position(|r| r.mode == mode && r.difficulty == difficulty)
        {
            Some(at) => at,
            None => {
                self.records.push(Record {
                    mode,
                    difficulty,
                    ..default()
                });
                self.records.len() - 1
            }
        };
        self.records[at].games += 1;
        self.records[at].wins += won as u32;
    }

    pub fn load(file: PathBuf) -> Self {
        let mut stats: Self = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        stats.file = Some(file);
        stats
    }

    pub fn save(&self) {
        let Some(file) = &self.file else {
            return;
        };
        let written = file
            .parent()
            .map_or(Ok(()), std::fs::create_dir_all)
            .and_then(|_| {
                let text = ron::to_string(self).map_err(std::io::Error::other)?;
                std::fs::write(file, text)
            });
        if let Err(err) = written {
            warn!("could not save stats to {}: {err}", file.display());
        }
    }
}

/// Counts time played and each finished local battle, and cups player 1 wins.
fn count_stats(
    time: Res<Time>,
    game: Res<Match>,
    config: Res<GameConfig>,
    cup: Option<Res<crate::cups::Cup>>,
    towers: Query<(&Tower, &TowerState)>,
    mut stats: ResMut<Stats>,
) {
    if game.phase == Phase::Playing {
        stats.seconds_played += time.delta_secs();
    }
    let battle = config.trial.is_none() && MODES.contains(&game.mode);
    if game.phase == Phase::Over && stats.counted != game.round && battle {
        stats.counted = game.round;
        if let Some((_, you)) = towers.iter().find(|(t, _)| t.is_first_player()) {
            let won = you.status == TowerStatus::Won;
            stats.add(game.mode, config.difficulty.unwrap_or_default(), won);
            stats.save();
        }
    }
    // A cup ends with standings; player 1's tower first means a cup won.
    let standings = cup.as_ref().and_then(|c| c.standings.clone());
    match standings {
        Some(order) if !stats.cup_counted => {
            stats.cup_counted = true;
            let first = towers
                .iter()
                .find(|(t, _)| t.is_first_player())
                .map(|(t, _)| t.index);
            if order.first().copied() == first && first.is_some() {
                stats.cups_won += 1;
                stats.save();
            }
        }
        None => stats.cup_counted = false,
        _ => {}
    }
}

/// Whether the popup is open.
#[derive(Resource, Debug, Default)]
pub struct StatsMenu {
    pub open: bool,
}

/// Layout in px of a 1920×1080 screen from its centre (y up). Panel sizes are from the
/// readout; their gap, heights on screen, the row layout and text sizes are guesses (?).
const GAME_PANEL: (f32, f32) = (730.0, 900.0);
const OTHER_PANEL: (f32, f32) = (730.0, 695.0);
const GAP: f32 = 40.0;
const TOP: f32 = 470.0;
const COLUMNS_X: [f32; 4] = [20.0, 140.0, 260.0, 380.0];
const ICON: f32 = 48.0;
const TEXT: f32 = 28.0;
const ROW_STEP: f32 = 62.0;

fn vh(px: f32) -> f32 {
    px / SCREEN.1 * 100.0
}

#[derive(Component)]
struct StatsRoot;

/// A number in the Game stats table: mode, column (0-2 a difficulty, 3 the total),
/// and whether it's wins.
#[derive(Component, Clone, Copy)]
enum StatText {
    Cell(usize, usize, bool),
    CupsWon,
    TimePlayed,
    Best(usize),
}

fn spawn_stats_popup(mut commands: Commands, art: Option<Res<Art>>) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let mut root = commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.7)),
        GlobalZIndex(12),
        Visibility::Hidden,
        StatsRoot,
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
    root.with_children(|root| {
        let panel_x = |left: bool| (GAME_PANEL.0 + GAP) / 2.0 * if left { -1.0 } else { 1.0 };
        for (size, left, title) in [
            (GAME_PANEL, true, "Game stats"),
            (OTHER_PANEL, false, "Other stats"),
        ] {
            let (x, y) = (panel_x(left), SCREEN.1 / 2.0 - TOP + 300.0 - size.1 / 2.0);
            let mut panel = root.spawn((
                place(size, (x, y)),
                BackgroundColor(Color::srgba(0.12, 0.1, 0.25, 0.95)),
            ));
            if let Some(back) = image("menu_leaderboard_panel") {
                panel.insert((
                    sliced(back, [40.0, 40.0, 51.0, 37.0]),
                    BackgroundColor(Color::NONE),
                ));
            }
            root.spawn(place((size.0, 60.0), (x, y + size.1 / 2.0 - 60.0)))
                .with_children(|t| {
                    t.spawn(label(vh(40.0), title));
                });
        }
        // Game stats: column icons, then per mode a games row and a wins row.
        let gx = panel_x(true) - GAME_PANEL.0 / 2.0 + 250.0;
        let top = SCREEN.1 / 2.0 - TOP + 300.0 - GAME_PANEL.1 / 2.0 + GAME_PANEL.1 / 2.0 - 140.0;
        for (c, icon) in [
            "stats_icon_easy",
            "stats_icon_normal",
            "stats_icon_special",
            "stats_icon_total",
        ]
        .iter()
        .enumerate()
        {
            let at = (gx + COLUMNS_X[c], top);
            let mut node = root.spawn(place((ICON, ICON), at));
            match image(icon) {
                Some(art) => {
                    node.insert(ImageNode::new(art));
                }
                None => {
                    let words = ["Easy", "Normal", "Special", "Total"][c];
                    node.with_children(|t| {
                        t.spawn(label(vh(20.0), words));
                    });
                }
            }
        }
        for (m, mode) in MODES.iter().enumerate() {
            for (k, wins) in [false, true].into_iter().enumerate() {
                let y = top - 80.0 - ROW_STEP * (2 * m + k) as f32 - 20.0 * m as f32;
                let left = panel_x(true) - GAME_PANEL.0 / 2.0 + 40.0;
                let name = if wins {
                    "Wins".to_string()
                } else {
                    format!("{} games", mode.name())
                };
                root.spawn(Node {
                    justify_content: JustifyContent::FlexStart,
                    ..place((220.0, 40.0), (left + 110.0, y))
                })
                .with_children(|t| {
                    t.spawn(label(vh(TEXT), &name));
                });
                for (c, column_x) in COLUMNS_X.iter().enumerate() {
                    root.spawn(place((110.0, 40.0), (gx + column_x, y)))
                        .with_children(|t| {
                            t.spawn((label(vh(TEXT), "0"), StatText::Cell(m, c, wins)));
                        });
                }
            }
        }
        // Other stats: cups won, time played, the Endless bests.
        let ox = panel_x(false);
        let otop = SCREEN.1 / 2.0 - TOP + 300.0 - OTHER_PANEL.1 / 2.0 + OTHER_PANEL.1 / 2.0 - 150.0;
        let mut rows: Vec<(String, StatText)> = vec![
            ("Cups won".into(), StatText::CupsWon),
            ("Time played".into(), StatText::TimePlayed),
        ];
        for (i, name) in ["Endless Race", "Endless Survival", "Endless Puzzle"]
            .iter()
            .enumerate()
        {
            rows.push((format!("{name} best"), StatText::Best(i)));
        }
        for (i, (name, part)) in rows.into_iter().enumerate() {
            let y = otop - ROW_STEP * i as f32;
            let left = ox - OTHER_PANEL.0 / 2.0 + 40.0;
            root.spawn(Node {
                justify_content: JustifyContent::FlexStart,
                ..place((400.0, 40.0), (left + 200.0, y))
            })
            .with_children(|t| {
                t.spawn(label(vh(TEXT), &name));
            });
            root.spawn(place((200.0, 40.0), (ox + OTHER_PANEL.0 / 2.0 - 140.0, y)))
                .with_children(|t| {
                    t.spawn((label(vh(TEXT), ""), part));
                });
        }
        root.spawn(place((300.0, 60.0), (0.0, -SCREEN.1 / 2.0 + 60.0)))
            .with_children(|t| {
                t.spawn(label(vh(46.0), "BACK"));
            });
    });
}

/// "1h 05m" style (?: the original's time format not read).
fn hours_minutes(seconds: f32) -> String {
    let minutes = (seconds / 60.0) as u32;
    format!("{}h {:02}m", minutes / 60, minutes % 60)
}

#[allow(clippy::type_complexity)]
fn show_stats_popup(
    menu: Res<StatsMenu>,
    screen: Res<crate::menu::MenuScreen>,
    game: Res<Match>,
    stats: Res<Stats>,
    board: Option<Res<crate::single::Leaderboard>>,
    mut root: Query<&mut Visibility, With<StatsRoot>>,
    mut texts: Query<(&StatText, &mut Text)>,
) {
    let on = menu.open && game.phase == Phase::Menu && *screen == crate::menu::MenuScreen::Settings;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    for (part, mut text) in &mut texts {
        let words = match *part {
            StatText::Cell(m, c, wins) => {
                let (games, won) = stats.get(MODES[m], DIFFICULTIES.get(c).copied());
                if wins { won } else { games }.to_string()
            }
            StatText::CupsWon => stats.cups_won.to_string(),
            StatText::TimePlayed => hours_minutes(stats.seconds_played),
            StatText::Best(i) => board
                .as_ref()
                .and_then(|b| b.scores[i].first().copied())
                .map_or("-".to_string(), |s| s.to_string()),
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
    fn records_add_up_per_difficulty_and_in_total() {
        let mut stats = Stats::default();
        stats.add(ModeKind::Race, Difficulty::Easy, true);
        stats.add(ModeKind::Race, Difficulty::Easy, false);
        stats.add(ModeKind::Race, Difficulty::Pro, true);
        stats.add(ModeKind::Puzzle, Difficulty::Normal, false);
        assert_eq!(stats.get(ModeKind::Race, Some(Difficulty::Easy)), (2, 1));
        assert_eq!(stats.get(ModeKind::Race, None), (3, 2));
        assert_eq!(stats.get(ModeKind::Survival, None), (0, 0));
        assert_eq!(
            stats.get(ModeKind::Puzzle, Some(Difficulty::Normal)),
            (1, 0)
        );
        assert_eq!(hours_minutes(3_900.0), "1h 05m");
    }
}
