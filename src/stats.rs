//! Player 1's statistics and the Stats popup from Settings, laid out from the PC readout
//! (docs/research/gap-scan.md, section 7): two panels side by side over a dark modal,
//! Game stats (730×900: per mode, Won / Draw (Puzzle) / Lost / Ratio in columns Easy /
//! Normal / Special / Total under their small icons) and Other stats (730×695: ten rows),
//! with the menu's BACK hint in the corner (navbar.rs). Kept between runs in `stats.ron`.
//!
//! What counts: local battles only (not trials or Endless runs), from player 1's tower.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, Phase, Tower, TowerState, TowerStatus};
use crate::select::{SCREEN, place, shown, sliced};

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

/// Games, wins and draws (Puzzle) in one mode at one difficulty.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Record {
    pub mode: ModeKind,
    pub difficulty: Difficulty,
    pub games: u32,
    pub wins: u32,
    #[serde(default)]
    pub draws: u32,
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

    /// Won, drawn and lost in `mode` at `difficulty`, or over every difficulty with `None`.
    pub fn outcomes(&self, mode: ModeKind, difficulty: Option<Difficulty>) -> (u32, u32, u32) {
        self.records
            .iter()
            .filter(|r| r.mode == mode && difficulty.is_none_or(|d| r.difficulty == d))
            .fold((0, 0, 0), |(w, d, l), r| {
                (w + r.wins, d + r.draws, l + r.games.saturating_sub(r.wins + r.draws))
            })
    }

    /// A drawn match (a Puzzle tie): counted as a game and a draw.
    pub fn add_draw(&mut self, mode: ModeKind, difficulty: Difficulty) {
        self.add(mode, difficulty, false);
        if let Some(r) = self.records.iter_mut().find(|r| r.mode == mode && r.difficulty == difficulty) {
            r.draws += 1;
        }
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
            // A Puzzle round nobody wins is a draw (the game stats' Draw row).
            if !won && game.mode == ModeKind::Puzzle && game.winner.is_none() {
                stats.add_draw(game.mode, difficulty);
            } else {
                stats.add(game.mode, difficulty, won);
            }
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

/// Layout in px of a 1920×1080 screen from its centre (y up), from StatsPopup.
const GAME_PANEL: (f32, f32) = (730.0, 900.0);
const OTHER_PANEL: (f32, f32) = (730.0, 695.0);
/// Both panels' top, up from the screen's middle.
const PANEL_TOP: f32 = 450.0;

fn vh(px: f32) -> f32 {
    px / SCREEN.1 * 100.0
}

#[derive(Component)]
struct StatsRoot;

/// A number in the Game stats table: mode, row (0 won, 1 drawn, 2 lost, 3 ratio) and
/// column (0-2 a difficulty, 3 the total).
#[derive(Component, Clone, Copy)]
enum StatText {
    Cell(usize, usize, usize),
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

/// StatsPopup's layout in px of a 1920 x 1080 screen from its middle (y up): the Game panel
/// (730 x 900) with its right edge 25 left of the middle, the Other panel (730 x 695) with its
/// left edge 25 right of it, both topped 450 up; gold titles on the panels' top edges; the
/// column icons 97 under the Game panel's top; each panel's items in a container 40 in from
/// the sides and 75 under the top. A mode item (StatsItem) is its name (30 px, black) over
/// rows (SubStatsItem: 45 px strips 52 apart, from 50 under the item's top) of Won, Draw
/// (Puzzle), Lost and Ratio, the items 10 apart after their 40 px margin.
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
    let name_look = crate::letters::TextLook {
        face: Some(Color::srgb(0.435, 0.474, 0.627)),
        ..default()
    };
    let value_look = crate::letters::TextLook {
        face: Some(Color::srgb(0.094, 0.122, 0.239)),
        ..default()
    };
    root.with_children(|root| {
        let game_left = -25.0 - GAME_PANEL.0;
        let other_left = 25.0;
        for (size, left, title) in [
            (GAME_PANEL, game_left, "Game stats"),
            (OTHER_PANEL, other_left, "Other stats"),
        ] {
            let mut panel = root.spawn((
                place(size, (left + size.0 / 2.0, PANEL_TOP - size.1 / 2.0)),
                BackgroundColor(Color::srgba(0.12, 0.1, 0.25, 0.95)),
            ));
            if let Some(back) = image("menu_leaderboard_panel") {
                panel.insert((
                    sliced(back, [40.0, 40.0, 37.0, 51.0]),
                    BackgroundColor(Color::NONE),
                ));
            }
            root.spawn(place((size.0, 60.0), (left + size.0 / 2.0, PANEL_TOP + 9.0)))
                .with_children(|t| {
                    t.spawn(crate::select::label_with(vh(46.0), title, crate::letters::TextLook::GOLD));
                });
        }
        // A 45 px strip with a name and values, as SubStatsItem / OtherStatsItem.
        let strip = |root: &mut ChildSpawnerCommands, left: f32, width: f32, y: f32| {
            let mut strip = root.spawn(place((width, 45.0), (left + width / 2.0, y)));
            if let Some(back) = image("popup_text_background") {
                strip.insert(ImageNode::new(back));
            }
        };
        // Column icons over the Easy, Normal, Special and Total columns.
        let inner = game_left + 40.0;
        let width = GAME_PANEL.0 - 80.0;
        for (icon, size, x) in [
            ("stats_icon_easy", (26.0, 25.0), inner + 262.0),
            ("stats_icon_normal", (41.0, 27.0), inner + 377.0),
            ("stats_icon_special", (52.0, 31.0), inner + 493.0),
            ("stats_icon_total", (18.0, 20.0), inner + width - 19.0 - 10.0),
        ] {
            let mut node = root.spawn(place(size, (x, PANEL_TOP - 97.0)));
            if let Some(art) = image(icon) {
                node.insert(ImageNode::new(art));
            }
        }
        let mut top = PANEL_TOP - 75.0;
        for (m, mode) in MODES.iter().enumerate() {
            root.spawn(Node {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..place((200.0, 50.0), (inner + 20.0 + 100.0, top - 25.0))
            })
            .with_children(|t| {
                t.spawn(crate::select::label_with(vh(30.0), mode.name(), crate::letters::TextLook {
                    face: Some(Color::BLACK),
                    ..default()
                }));
            });
            let rows: &[(usize, &str)] = if *mode == ModeKind::Puzzle {
                &[(0, "Won"), (1, "Draw"), (2, "Lost"), (3, "Ratio")]
            } else {
                &[(0, "Won"), (2, "Lost"), (3, "Ratio")]
            };
            for (i, &(row, name)) in rows.iter().enumerate() {
                let y = top - 50.0 - 22.5 - 52.0 * i as f32;
                strip(root, inner, width, y);
                root.spawn(Node {
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::Center,
                    ..place((200.0, 45.0), (inner + 20.0 + 100.0, y))
                })
                .with_children(|t| {
                    t.spawn(crate::select::label_with(vh(19.0), name, name_look));
                });
                // Easy, Normal, Pro centred in 115 px boxes from 205, 320, 435; the total
                // right-aligned from 535.
                for (c, (x, w, right)) in [
                    (205.0, 115.0, false),
                    (320.0, 115.0, false),
                    (435.0, 115.0, false),
                    (535.0, width - 555.0, true),
                ]
                .into_iter()
                .enumerate()
                {
                    root.spawn(Node {
                        justify_content: if right { JustifyContent::FlexEnd } else { JustifyContent::Center },
                        align_items: AlignItems::Center,
                        ..place((w, 45.0), (inner + x + w / 2.0, y))
                    })
                    .with_children(|t| {
                        t.spawn((crate::select::label_with(vh(26.0), "0", value_look), StatText::Cell(m, row, c)));
                    });
                }
            }
            top -= 50.0 + 52.0 * rows.len() as f32 + 40.0 - 50.0 + 10.0;
        }
        // Other stats: StatsPopup's rows, in its order (the English of each STATS_ key).
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
        let inner = other_left + 40.0;
        let width = OTHER_PANEL.0 - 80.0;
        for (i, (name, part)) in rows.into_iter().enumerate() {
            let y = PANEL_TOP - 75.0 - 22.5 - 55.0 * i as f32;
            strip(root, inner, width, y);
            root.spawn(Node {
                justify_content: JustifyContent::FlexStart,
                align_items: AlignItems::Center,
                ..place((370.0, 45.0), (inner + 20.0 + 185.0, y))
            })
            .with_children(|t| {
                t.spawn(crate::select::label_with(vh(19.0), &name, name_look));
            });
            root.spawn(Node {
                justify_content: JustifyContent::FlexEnd,
                align_items: AlignItems::Center,
                ..place((width - 555.0, 45.0), (inner + 535.0 + (width - 555.0) / 2.0, y))
            })
            .with_children(|t| {
                t.spawn((crate::select::label_with(vh(26.0), "", value_look), part));
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
            StatText::Cell(m, row, c) => {
                let (won, drawn, lost) = stats.outcomes(MODES[m], DIFFICULTIES.get(c).copied());
                match row {
                    0 => won.to_string(),
                    1 => drawn.to_string(),
                    2 => lost.to_string(),
                    // GameStatsItem: (won + drawn in Puzzle) / lost, or that count with no
                    // losses, or "-" with neither; two decimals.
                    _ => {
                        let good = (won + if MODES[m] == ModeKind::Puzzle { drawn } else { 0 }) as f32;
                        if lost > 0 {
                            format!("{:.2}", good / lost as f32)
                        } else if good > 0.0 {
                            format!("{good:.2}")
                        } else {
                            "-".to_string()
                        }
                    }
                }
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
