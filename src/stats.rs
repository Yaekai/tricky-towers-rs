//! Player 1's statistics and the Stats popup from Settings, laid out from the PC readout
//! (docs/research/gap-scan.md, section 7): two panels side by side over a dark modal,
//! Game stats (730×900: per mode, games and wins in columns Easy / Normal / Pro / Total
//! with their small icons) and Other stats (730×695: cups, time played, high scores),
//! with the menu's BACK hint in the corner (navbar.rs). Kept between runs in `stats.ron`.
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
    /// StatsPopup's other stats, player 1's in every mode: bricks placed, bubbled bricks
    /// that came down and settled ("Bubbles stomped"), the spells cast (by name), the
    /// highest curse wave reached and the fastest won Race on Normal (seconds).
    pub bricks_placed: u32,
    pub bubbles_stomped: u32,
    pub spells_used: Vec<crate::config::SpellKind>,
    pub highest_wave: u32,
    pub fastest_normal_race: Option<f32>,
    #[serde(skip)]
    pub file: Option<PathBuf>,
    #[serde(skip)]
    counted: u64,
    /// Bricks player 1 had placed when last counted, the casts seen, this match's waves.
    #[serde(skip)]
    seen_placed: (u64, u32),
    #[serde(skip)]
    seen_casts: u64,
    #[serde(skip)]
    waves: (u64, u32, bool),
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
#[allow(clippy::too_many_arguments)]
fn count_stats(
    time: Res<Time>,
    game: Res<Match>,
    config: Res<GameConfig>,
    cup: Option<Res<crate::cups::Cup>>,
    towers: Query<(&Tower, &TowerState)>,
    log: Option<Res<crate::spells::SpellLog>>,
    mut unbubbled: RemovedComponents<crate::spells::Bubbled>,
    bricks: Query<&crate::gameplay::Brick>,
    mut stats: ResMut<Stats>,
) {
    if game.phase == Phase::Playing {
        stats.seconds_played += time.delta_secs();
    }
    let first = towers.iter().find(|(t, _)| t.is_first_player());
    // Bricks placed, spells cast, bubbles that came down and curse waves reached.
    if let Some((tower, state)) = first {
        let (round, placed) = stats.seen_placed;
        if round != game.round {
            stats.seen_placed = (game.round, state.bricks_placed);
        } else if state.bricks_placed > placed {
            stats.bricks_placed += state.bricks_placed - placed;
            stats.seen_placed.1 = state.bricks_placed;
        }
        if let Some(log) = log.as_deref() {
            let new = log.1.saturating_sub(stats.seen_casts) as usize;
            stats.seen_casts = log.1;
            for cast in log.0.iter().rev().take(new) {
                if cast.caster == tower.index && !stats.spells_used.contains(&cast.spell) {
                    stats.spells_used.push(cast.spell);
                }
            }
        }
        for entity in unbubbled.read() {
            if bricks.get(entity).is_ok_and(|b| towers.get(b.tower).is_ok_and(|(t, _)| t.index == tower.index)) {
                stats.bubbles_stomped += 1;
            }
        }
        let (round, count, on) = stats.waves;
        let (count, mut on) = if round == game.round { (count, on) } else { (0, false) };
        let mut count = count;
        let waving = state.wave_left > 0.0 || state.wave_bricks > 0;
        if waving && !on {
            count += 1;
            stats.highest_wave = stats.highest_wave.max(count);
        }
        on = waving;
        stats.waves = (game.round, count, on);
    }
    let battle = config.trial.is_none() && MODES.contains(&game.mode);
    if game.phase == Phase::Over && stats.counted != game.round && battle {
        stats.counted = game.round;
        if let Some((_, you)) = first {
            let won = you.status == TowerStatus::Won;
            let difficulty = config.difficulty.unwrap_or_default();
            stats.add(game.mode, difficulty, won);
            // The fastest Race won on Normal (GoToRoof with levelId RACE_NORMAL).
            if won && game.mode == ModeKind::Race && difficulty == Difficulty::Normal {
                let time = game.elapsed;
                stats.fastest_normal_race = Some(stats.fastest_normal_race.map_or(time, |t| t.min(time)));
            }
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
/// Numbers and the stat names beside them (SubStatsItem: 26 and 19).
const TEXT: f32 = 26.0;
const NAME_TEXT: f32 = 19.0;
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
    BricksPlaced,
    TimePlayed,
    Medals,
    LocalMatches,
    Best(usize),
    Bubbles,
    SpellsUsed,
    HighestWave,
    FastestRace,
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
                    sliced(back, [40.0, 40.0, 37.0, 51.0]),
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
                    t.spawn(label(vh(NAME_TEXT), &name));
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
        // StatsPopup's rows, in its order (the English of each STATS_ key).
        let rows: Vec<(String, StatText)> = vec![
            ("Bricks placed".into(), StatText::BricksPlaced),
            ("Time played".into(), StatText::TimePlayed),
            ("Medals".into(), StatText::Medals),
            ("Local matches started".into(), StatText::LocalMatches),
            ("Endless survival score".into(), StatText::Best(1)),
            ("Endless puzzle score".into(), StatText::Best(2)),
            ("Bubbles stomped".into(), StatText::Bubbles),
            ("Spells used".into(), StatText::SpellsUsed),
            ("Highest wave".into(), StatText::HighestWave),
            ("Fastest normal race".into(), StatText::FastestRace),
        ];
        // Each row is OtherStatsItem: a 45 px strip (gph_popupTextBackground02) across the
        // container (the panel less 40 px a side, its top 75 px down), 55 px apart; the name
        // 20 px in at 19 px in grey-blue and the value from 535 px at 26 px in dark navy
        // (EnzoBlackBasicWhite tinted by each text's colour).
        let width = OTHER_PANEL.0 - 80.0;
        let panel_top = SCREEN.1 / 2.0 - TOP + 300.0;
        let left = ox - width / 2.0;
        for (i, (name, part)) in rows.into_iter().enumerate() {
            let y = panel_top - 75.0 - 22.5 - 55.0 * i as f32;
            let mut strip = root.spawn(place((width, 45.0), (ox, y)));
            if let Some(back) = image("popup_text_background") {
                strip.insert(ImageNode::new(back));
            }
            root.spawn(Node {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..place((370.0, 45.0), (left + 20.0 + 185.0, y))
            })
            .with_children(|t| {
                t.spawn(crate::select::label_with(vh(19.0), &name, crate::letters::TextLook {
                    face: Some(Color::srgb(0.435, 0.474, 0.627)),
                    ..default()
                }));
            });
            root.spawn(Node {
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..place((width - 555.0, 45.0), (left + 535.0 + (width - 555.0) / 2.0, y))
            })
            .with_children(|t| {
                t.spawn((
                    crate::select::label_with(vh(26.0), "", crate::letters::TextLook {
                        face: Some(Color::srgb(0.094, 0.122, 0.239)),
                        ..default()
                    }),
                    part,
                ));
            });
        }
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
    config: Res<GameConfig>,
    progress: Option<Res<crate::trials::TrialProgress>>,
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
            StatText::BricksPlaced => stats.bricks_placed.to_string(),
            StatText::TimePlayed => hours_minutes(stats.seconds_played),
            StatText::Medals => format!(
                "{}/{}",
                progress.as_ref().map_or(0, |p| p.medals(&config.trial_tiers)),
                config.trials.len() as u32 * config.trial_tiers.medals_per_trial
            ),
            StatText::LocalMatches => stats.records.iter().map(|r| r.games).sum::<u32>().to_string(),
            StatText::Best(i) => board
                .as_ref()
                .and_then(|b| b.scores[i].first().copied())
                .unwrap_or(0)
                .to_string(),
            StatText::Bubbles => stats.bubbles_stomped.to_string(),
            // 17 regular spells, plus one for any super and one for any super duper curse
            // (SpellName); our curses don't combine yet, so those two stay unearned.
            StatText::SpellsUsed => format!("{}/19", stats.spells_used.len().min(17)),
            StatText::HighestWave => stats.highest_wave.to_string(),
            StatText::FastestRace => stats.fastest_normal_race.map_or("--:--:--".to_string(), |t| {
                let cs = (t * 100.0).round() as u32;
                format!("{:02}:{:02}:{:02}", cs / 6000, cs / 100 % 60, cs % 100)
            }),
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
