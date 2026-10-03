//! Data-driven game configuration.
//!
//! Everything a modder is likely to want to tweak (physics feel, brick shapes,
//! mode rules, spells, opponents) lives here and is loaded from
//! `assets/config/game.ron` at startup. Missing fields fall back to the defaults
//! below, so a mod file only needs the values it changes.

use std::path::Path;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

/// Default location of the config file, relative to the working directory.
pub const DEFAULT_CONFIG_PATH: &str = "assets/config/game.ron";

#[derive(Resource, Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct GameConfig {
    /// World units per brick cell. All other lengths are given in cells.
    pub cell_size: f32,
    /// Downward gravity in cells per second squared.
    pub gravity: f32,
    /// Drop speed added per cell of the tallest tower in the match.
    pub fall_speed_per_height: f32,
    /// Drop speed added for each player who is out.
    pub fall_speed_per_out: f32,
    /// Holding "down" speeds the brick up by this much per second...
    pub drop_acceleration: f32,
    /// ...up to this speed, in cells per second.
    pub drop_max_speed: f32,
    /// Letting go slows it back down to the drop speed by this much per second.
    pub drop_deceleration: f32,
    /// Sideways step of a move, in cells.
    pub move_step: f32,
    /// Seconds a direction must be held before the move repeats.
    pub move_repeat_delay: f32,
    /// Seconds between repeated moves while a direction is held.
    pub move_repeat_interval: f32,
    /// Distance of a nudge (its own button), in cells.
    pub nudge_distance: f32,
    /// Sideways impulse when a nudge hits a brick, on both bricks, in cells/s.
    pub nudge_impulse: f32,
    /// A move blocked by more than this many cells bumps the brick in its way.
    pub bump_threshold: f32,
    /// Bump impulse on the moving brick, and on the brick it hits, in cells/s.
    pub bump_impulse: f32,
    pub bump_impulse_other: f32,
    /// How far either side of the lane centre a steered brick's centre may go.
    pub lane_half_width: f32,
    /// A turn is tried in this many steps; if one is blocked, the brick is
    /// placed at the last free angle.
    pub rotation_steps: u32,
    /// The steered brick is shrunk by this much on each side, in cells.
    pub steer_inset: f32,
    /// Width of the base platform, in cells.
    pub platform_width: f32,
    /// Horizontal distance between neighbouring towers, in cells.
    pub tower_spacing: f32,
    /// How far above the tower top a new brick appears, in cells.
    pub spawn_gap: f32,
    /// Bricks that fall this far below the platform are counted as dropped.
    pub kill_depth: f32,
    /// Fraction of a cell used for the physics collider of each cell. Slightly
    /// under 1.0 so neighbouring bricks do not start in contact.
    pub collider_scale: f32,
    pub brick_friction: f32,
    pub brick_restitution: f32,
    /// Every brick weighs the same, whatever its size or shape.
    pub brick_mass: f32,
    pub angular_damping: f32,
    /// Physics and gameplay steps per second.
    pub physics_hz: f64,
    /// Seed for the brick randomizer, so runs are reproducible.
    pub seed: u64,
    /// Open the mode menu on launch instead of starting a match straight away.
    pub start_in_menu: bool,
    /// Mode of the next match.
    pub mode: ModeKind,
    /// Number of AI opponents in the next match.
    pub opponents: u32,
    pub race: RaceRules,
    pub survival: SurvivalRules,
    pub puzzle: PuzzleRules,
    pub magic: MagicRules,
    pub ai: AiRules,
    pub pieces: Vec<PieceDef>,
}

#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum ModeKind {
    /// Build past a finish line and keep the tower there.
    #[default]
    Race,
    /// Place a number of bricks without losing all your lives.
    Survival,
    /// Place a fixed set of bricks without crossing a laser line.
    Puzzle,
}

impl ModeKind {
    pub const ALL: [ModeKind; 3] = [ModeKind::Race, ModeKind::Survival, ModeKind::Puzzle];

    pub fn name(self) -> &'static str {
        match self {
            ModeKind::Race => "Race",
            ModeKind::Survival => "Survival",
            ModeKind::Puzzle => "Puzzle",
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct RaceRules {
    /// Finish line height above the platform, in cells.
    pub finish_height: f32,
    /// Seconds the tower must stay past the line to win (?).
    pub hold_seconds: f32,
    /// Seconds into the race before the finish line starts coming down.
    pub finish_drop_after: f32,
    /// Cells per second the finish line comes down.
    pub finish_drop_speed: f32,
    /// Base drop speed, in cells/s.
    pub drop_speed: f32,
}

impl Default for RaceRules {
    /// The original's Normal difficulty.
    fn default() -> Self {
        Self {
            finish_height: 45.0,
            hold_seconds: 3.0,
            finish_drop_after: 120.0,
            finish_drop_speed: 0.2,
            drop_speed: 3.0,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SurvivalRules {
    pub lives: u32,
    /// Bricks each player gets; dropped ones count too.
    pub bricks_to_place: u32,
    /// Base drop speed, in cells/s.
    pub drop_speed: f32,
    /// After losing a life, seconds before another brick can cost one.
    pub invincible_seconds: f32,
    /// Seconds after the last brick before the tower is finished.
    pub finish_countdown: f32,
}

impl Default for SurvivalRules {
    fn default() -> Self {
        Self {
            lives: 3,
            bricks_to_place: 66,
            drop_speed: 2.0,
            invincible_seconds: 1.5,
            finish_countdown: 3.0,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PuzzleRules {
    /// Bricks to place.
    pub bricks: u32,
    /// Height of the laser line, in cells. Any settled brick above it fails the puzzle.
    pub line_height: f32,
    /// Seconds everything must rest below the line after the last brick lands.
    pub settle_seconds: f32,
    /// Base drop speed, in cells/s (?).
    pub drop_speed: f32,
    /// The light spell every player starts with; there are no spell lines.
    pub start_spell: Option<SpellKind>,
    /// A dropped brick costs no life; the laser comes this much lower instead.
    pub drop_penalty: f32,
}

impl Default for PuzzleRules {
    fn default() -> Self {
        Self {
            bricks: 10,
            line_height: 6.0,
            settle_seconds: 2.0,
            drop_speed: 3.0,
            start_spell: Some(SpellKind::Ivy),
            drop_penalty: 0.5,
        }
    }
}

/// Spells, named as in the original. Light spells help the caster; dark spells
/// curse every other player. See `docs/research/spell-awards.md`.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum SpellKind {
    /// Light: your current brick vines itself to every brick it lands on.
    Ivy,
    /// Light: your current brick turns to stone when it lands.
    Petrify,
    /// Light: zaps away the brick you placed last.
    Undo,
    /// Light (Survival): a heart orb you can touch with a brick for a life.
    ExtraLife,
    /// Light: a small stone ledge beside your tower.
    Minibase,
    /// Light, when far behind: your next bricks drop and petrify by themselves.
    AutoBuild,
    /// Dark: the current brick grows to twice its size.
    Large,
    /// Dark: the next bricks fall much faster.
    Fast,
    /// Dark: the current brick cannot be rotated.
    NoRotate,
    /// Dark: the current brick keeps turning by itself.
    AutoRotate,
    /// Dark: the next bricks are slippery as ice.
    Ice,
    /// Dark: fog over the top of the tower.
    Mist,
    /// Dark: a balloon catches the current brick and slows it right down.
    Slow,
    /// Dark: the current brick becomes an odd shape.
    Mystery,
    /// Dark: a bubble lifts the last placed brick off the tower for a while.
    Bubble,
    /// Dark: grass covers the top of the tower (looks only).
    Grassy,
}

impl SpellKind {
    pub fn is_dark(self) -> bool {
        !matches!(
            self,
            SpellKind::Ivy
                | SpellKind::Petrify
                | SpellKind::Undo
                | SpellKind::ExtraLife
                | SpellKind::Minibase
                | SpellKind::AutoBuild
        )
    }

    pub fn name(self) -> &'static str {
        match self {
            SpellKind::Ivy => "Ivy",
            SpellKind::Petrify => "Petrify",
            SpellKind::Undo => "Undo",
            SpellKind::ExtraLife => "Extra Life",
            SpellKind::Minibase => "Minibase",
            SpellKind::AutoBuild => "Auto Build",
            SpellKind::Large => "Large",
            SpellKind::Fast => "Fast",
            SpellKind::NoRotate => "No Rotate",
            SpellKind::AutoRotate => "Auto Rotate",
            SpellKind::Ice => "Ice",
            SpellKind::Mist => "Mist",
            SpellKind::Slow => "Balloon",
            SpellKind::Mystery => "Mystery",
            SpellKind::Bubble => "Bubble",
            SpellKind::Grassy => "Grassy",
        }
    }
}

/// One dark draw: one or more effects cast together.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Curse {
    pub spells: Vec<SpellKind>,
    pub weight: u32,
    /// Bricks every effect lasts. `None` uses each effect's own length
    /// (Fast and Ice last several bricks, the rest one).
    #[serde(default)]
    pub bricks: Option<u32>,
}

impl Curse {
    pub fn new(spells: &[SpellKind], weight: u32, bricks: Option<u32>) -> Self {
        Self {
            spells: spells.to_vec(),
            weight,
            bricks,
        }
    }

    pub fn name(&self) -> String {
        let names: Vec<_> = self.spells.iter().map(|s| s.name()).collect();
        names.join(" + ")
    }
}

/// A named group of curses to draw from.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct DarkSet {
    pub name: String,
    pub curses: Vec<Curse>,
}

/// One rubber-banding rule: the first rule whose conditions hold picks the
/// light spells on offer, and optionally the dark set.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct AwardRule {
    /// Holds only in this mode.
    pub mode: Option<ModeKind>,
    /// Holds only when the tower has (true) or hasn't (false) lost a life.
    pub hurt: Option<bool>,
    /// Holds when the race finish is at most this many cells above the tower.
    pub finish_within: Option<f32>,
    /// Holds when the leader's tower is at least this many cells taller.
    pub behind_leader: Option<f32>,
    /// Light spells and their weights.
    pub light: Vec<(SpellKind, u32)>,
    /// Dark set to draw from; `None` picks it from the light spell.
    pub dark: Option<String>,
    /// Which dark set follows each light spell when `dark` is `None`.
    pub dark_after: Vec<(SpellKind, String)>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct MagicRules {
    pub enabled: bool,
    /// Cells between one spell line and the next, with two players.
    pub line_spacing: f32,
    /// Extra spacing per player beyond two, as a fraction.
    pub line_spacing_per_extra_player: f32,
    /// Race: seconds a new line waits before sliding down to the tower.
    pub line_wait: f32,
    /// Extra wait per player beyond two, as a fraction.
    pub line_wait_per_extra_player: f32,
    /// Race: cells per second a waiting line slides down.
    pub line_slide_speed: f32,
    /// A line further above the tower than this slides faster, by half the excess.
    pub line_catch_up_beyond: f32,
    /// How much of that excess is added to the slide speed.
    pub line_catch_up_rate: f32,
    /// A line further above the tower than this slides without waiting.
    pub line_rush_beyond: f32,
    /// Lines this close under the finish are hidden.
    pub line_hide_near_finish: f32,
    /// The first rule that holds picks the spell pair. Puzzle has no spell lines.
    pub award_rules: Vec<AwardRule>,
    pub dark_sets: Vec<DarkSet>,

    /// Ivy joins bricks closer than this, in cells.
    pub ivy_reach: f32,
    /// Seconds Undo's zap lasts before the brick is gone.
    pub undo_seconds: f32,
    /// Minibase ledge: distance from the tower centre, in cells, min and max.
    pub minibase_offset: (f32, f32),
    /// Minibase ledge height above the tower (or `minibase_floor`, if higher).
    pub minibase_lift: f32,
    pub minibase_floor: f32,
    /// Minibase ledge width, in cells (?).
    pub minibase_width: f32,
    /// Extra Life orb: x from the tower centre, and height above the tower
    /// (or `life_orb_floor`, if higher).
    pub life_orb_x: f32,
    pub life_orb_lift: f32,
    pub life_orb_floor: f32,
    pub auto_build_bricks: u32,
    /// Size of a Large brick, and how much each further Large adds.
    pub large_scale: f32,
    pub large_stack: f32,
    pub fast_bricks: u32,
    /// A Fast brick falls at least this much faster than the drop speed, in cells/s.
    pub fast_extra_speed: f32,
    pub ice_bricks: u32,
    pub ice_friction: f32,
    /// Auto Rotate: seconds between turns, including the turn itself.
    pub auto_rotate_interval: f32,
    /// Shapes Mystery can turn a brick into, drawn from a bag.
    pub mystery_pieces: Vec<PieceDef>,
    /// Bubble lift as an upward acceleration in cells/s²: at the start, after
    /// `bubble_ease` seconds, and for the rest of `bubble_seconds`.
    pub bubble_lift: (f32, f32, f32),
    pub bubble_ease: f32,
    pub bubble_seconds: f32,
    /// Balloon: fall speed, fall speed while holding down, and the fastest
    /// it moves sideways, in cells/s.
    pub slow_fall_speed: f32,
    pub slow_fast_fall_speed: f32,
    pub slow_side_speed: f32,
    pub mist_seconds: f32,
    /// Height of the fog over the tower top, in cells.
    pub mist_height: f32,
    pub grassy_radius: f32,
}

impl Default for MagicRules {
    fn default() -> Self {
        use SpellKind::*;
        let set = |name: &str, curses: Vec<Curse>| DarkSet {
            name: name.into(),
            curses,
        };
        let c = Curse::new;
        Self {
            enabled: true,
            line_spacing: 9.0,
            line_spacing_per_extra_player: 0.25,
            line_wait: 18.0,
            line_wait_per_extra_player: 0.25,
            line_slide_speed: 0.5,
            line_catch_up_beyond: 9.0,
            line_catch_up_rate: 0.5,
            line_rush_beyond: 19.0,
            line_hide_near_finish: 2.0,
            award_rules: vec![
                AwardRule {
                    mode: Some(ModeKind::Race),
                    finish_within: Some(15.0),
                    light: vec![(Ivy, 1), (Undo, 1)],
                    dark: Some("normal".into()),
                    ..default()
                },
                AwardRule {
                    mode: Some(ModeKind::Race),
                    behind_leader: Some(15.0),
                    light: vec![(AutoBuild, 2), (Petrify, 1), (Minibase, 1)],
                    dark: Some("far_behind".into()),
                    ..default()
                },
                AwardRule {
                    mode: Some(ModeKind::Race),
                    behind_leader: Some(8.0),
                    light: vec![(Petrify, 1), (Minibase, 1)],
                    dark: Some("behind".into()),
                    ..default()
                },
                AwardRule {
                    mode: Some(ModeKind::Race),
                    light: vec![(Ivy, 10), (Undo, 10), (Petrify, 3)],
                    dark_after: vec![
                        (Ivy, "normal".into()),
                        (Undo, "normal".into()),
                        (Petrify, "stone".into()),
                    ],
                    ..default()
                },
                AwardRule {
                    mode: Some(ModeKind::Survival),
                    hurt: Some(true),
                    light: vec![
                        (ExtraLife, 4),
                        (Ivy, 3),
                        (Minibase, 1),
                        (Petrify, 1),
                        (Undo, 2),
                    ],
                    dark_after: survival_dark_after(),
                    ..default()
                },
                AwardRule {
                    mode: Some(ModeKind::Survival),
                    light: vec![(Ivy, 3), (Minibase, 1), (Petrify, 1), (Undo, 2)],
                    dark_after: survival_dark_after(),
                    ..default()
                },
            ],
            dark_sets: vec![
                set(
                    "normal",
                    vec![
                        c(&[Large], 10, None),
                        c(&[Grassy], 10, None),
                        c(&[Bubble], 10, None),
                        c(&[Mystery], 10, None),
                        c(&[AutoRotate], 10, None),
                        c(&[Slow], 6, None),
                    ],
                ),
                set("stone", vec![c(&[Slow], 10, None), c(&[Ice], 10, None)]),
                set(
                    "survival",
                    vec![
                        c(&[Large], 10, None),
                        c(&[Mystery], 10, None),
                        c(&[NoRotate], 5, None),
                        c(&[Mist], 3, None),
                    ],
                ),
                // Ghost (10) and three rare combos still to come.
                set(
                    "survival_stone",
                    vec![
                        c(&[NoRotate], 7, None),
                        c(&[Mist], 10, None),
                        c(&[Fast], 10, None),
                    ],
                ),
                set(
                    "behind",
                    vec![
                        c(&[Slow], 10, None),
                        c(&[Ice], 10, None),
                        c(&[Large, Mystery], 4, None),
                        c(&[Large, AutoRotate], 4, None),
                        c(&[Large, Ice], 4, Some(1)),
                    ],
                ),
                set(
                    "far_behind",
                    vec![
                        c(&[Slow], 4, None),
                        c(&[Ice], 4, None),
                        c(&[Large, Mystery], 10, None),
                        c(&[Large, AutoRotate], 10, None),
                        c(&[Large, Ice], 10, Some(1)),
                        c(&[Ice, AutoRotate, Mystery], 3, Some(4)),
                    ],
                ),
            ],
            ivy_reach: 0.2,
            undo_seconds: 0.75,
            minibase_offset: (3.0, 5.0),
            minibase_lift: 3.0,
            minibase_floor: 2.0,
            minibase_width: 3.0,
            life_orb_x: -5.0,
            life_orb_lift: 3.0,
            life_orb_floor: 10.0,
            auto_build_bricks: 5,
            large_scale: 2.0,
            large_stack: 0.5,
            fast_bricks: 5,
            fast_extra_speed: 7.5,
            ice_bricks: 4,
            ice_friction: 0.06,
            auto_rotate_interval: 1.65,
            mystery_pieces: default_mystery_pieces(),
            bubble_lift: (21.5, 18.0, 17.5),
            bubble_ease: 1.0,
            bubble_seconds: 3.0,
            slow_fall_speed: 3.0,
            slow_fast_fall_speed: 6.0,
            slow_side_speed: 1.0,
            mist_seconds: 15.0,
            mist_height: 5.0,
            grassy_radius: 5.0,
        }
    }
}

impl MagicRules {
    /// Cells between spell lines with `players` players in the match.
    pub fn spacing(&self, players: usize) -> f32 {
        let extra = players.saturating_sub(2) as f32;
        self.line_spacing * (1.0 + self.line_spacing_per_extra_player * extra)
    }

    /// Seconds a new race line waits with `players` players in the match.
    pub fn wait(&self, players: usize) -> f32 {
        let extra = players.saturating_sub(2) as f32;
        self.line_wait * (1.0 + self.line_wait_per_extra_player * extra)
    }

    pub fn dark_set(&self, name: &str) -> Option<&DarkSet> {
        self.dark_sets.iter().find(|s| s.name == name)
    }
}

/// Survival's dark set after each light spell. After Extra Life the original
/// uses harsher combos (still to be read); the stone set stands in for now.
fn survival_dark_after() -> Vec<(SpellKind, String)> {
    use SpellKind::*;
    vec![
        (Ivy, "survival".into()),
        (Undo, "survival".into()),
        (Petrify, "survival_stone".into()),
        (Minibase, "survival_stone".into()),
        (ExtraLife, "survival_stone".into()),
    ]
}

/// The three Mystery shapes: a 2x4 block, a ten-cell blob (exact cells still
/// to be checked) and a table on two feet.
pub fn default_mystery_pieces() -> Vec<PieceDef> {
    vec![
        PieceDef::new(
            "Block",
            [0.80, 0.55, 0.85],
            &[
                (0, 0),
                (1, 0),
                (2, 0),
                (3, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
            ],
        )
        .with_mass(1.5),
        PieceDef::new(
            "Blob",
            [0.55, 0.80, 0.70],
            &[
                (1, 0),
                (2, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
            ],
        )
        .with_mass(1.75),
        PieceDef::new(
            "Table",
            [0.85, 0.70, 0.45],
            &[
                (0, 0),
                (3, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (3, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (3, 2),
            ],
        )
        .with_mass(1.75),
    ]
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct AiRules {
    /// Seconds an opponent waits before acting on a new brick.
    pub think_delay: f32,
    /// Seconds between an opponent's moves.
    pub step_interval: f32,
    /// Chance (0 to 1) an opponent misjudges where to put a brick.
    pub mistake_chance: f32,
}

impl Default for AiRules {
    fn default() -> Self {
        Self {
            think_delay: 0.4,
            step_interval: 0.08,
            mistake_chance: 0.15,
        }
    }
}

/// One brick shape: a set of unit cells on a grid.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct PieceDef {
    pub name: String,
    /// sRGB colour, each channel 0.0 to 1.0.
    pub color: [f32; 3],
    /// Cell coordinates (x right, y up). Any polyomino works, not just tetrominoes.
    pub cells: Vec<(i32, i32)>,
    /// Mass relative to a normal brick.
    #[serde(default = "one")]
    pub mass: f32,
}

fn one() -> f32 {
    1.0
}

impl PieceDef {
    pub fn new(name: &str, color: [f32; 3], cells: &[(i32, i32)]) -> Self {
        Self {
            name: name.to_string(),
            color,
            cells: cells.to_vec(),
            mass: 1.0,
        }
    }

    pub fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    /// Cell centres in cell units, shifted so the bounding box is centred on the
    /// origin. Rotating about the origin then matches how the original pivots.
    pub fn centered_cells(&self) -> Vec<Vec2> {
        let (min_x, max_x) = minmax(self.cells.iter().map(|c| c.0));
        let (min_y, max_y) = minmax(self.cells.iter().map(|c| c.1));
        let center = Vec2::new(min_x as f32 + max_x as f32, min_y as f32 + max_y as f32) * 0.5;
        self.cells
            .iter()
            .map(|&(x, y)| Vec2::new(x as f32, y as f32) - center)
            .collect()
    }
}

fn minmax(values: impl Iterator<Item = i32>) -> (i32, i32) {
    values.fold((i32::MAX, i32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)))
}

/// The seven classic tetrominoes, in the original's spawn orientation
/// (T, J and L hang their extra cell under the bar).
pub fn default_pieces() -> Vec<PieceDef> {
    vec![
        PieceDef::new("I", [0.30, 0.80, 0.95], &[(0, 0), (1, 0), (2, 0), (3, 0)]),
        PieceDef::new("O", [0.95, 0.85, 0.30], &[(0, 0), (1, 0), (0, 1), (1, 1)]),
        PieceDef::new("T", [0.70, 0.40, 0.90], &[(0, 1), (1, 1), (2, 1), (1, 0)]),
        PieceDef::new("S", [0.45, 0.85, 0.40], &[(0, 0), (1, 0), (1, 1), (2, 1)]),
        PieceDef::new("Z", [0.95, 0.40, 0.40], &[(0, 1), (1, 1), (1, 0), (2, 0)]),
        PieceDef::new("J", [0.35, 0.50, 0.95], &[(0, 1), (1, 1), (2, 1), (2, 0)]),
        PieceDef::new("L", [0.95, 0.60, 0.25], &[(0, 1), (1, 1), (2, 1), (0, 0)]),
    ]
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            cell_size: 32.0,
            gravity: 18.0,
            fall_speed_per_height: 0.1,
            fall_speed_per_out: 2.0,
            drop_acceleration: 60.0,
            drop_max_speed: 60.0,
            drop_deceleration: 100.0,
            move_step: 0.5,
            move_repeat_delay: 0.2,
            move_repeat_interval: 0.05,
            nudge_distance: 1.0,
            nudge_impulse: 4.5,
            bump_threshold: 0.25,
            bump_impulse: 0.6,
            bump_impulse_other: 1.2,
            lane_half_width: 7.0,
            rotation_steps: 6,
            steer_inset: 0.15,
            platform_width: 9.0,
            tower_spacing: 16.0,
            spawn_gap: 17.0,
            kill_depth: 8.5,
            collider_scale: 0.98,
            brick_friction: 0.4,
            brick_restitution: 0.0,
            brick_mass: 1.0,
            angular_damping: 0.05,
            physics_hz: 50.0,
            seed: 0x7457_4552,
            start_in_menu: false,
            mode: ModeKind::Race,
            opponents: 0,
            race: RaceRules::default(),
            survival: SurvivalRules::default(),
            puzzle: PuzzleRules::default(),
            magic: MagicRules::default(),
            ai: AiRules::default(),
            pieces: default_pieces(),
        }
    }
}

impl GameConfig {
    /// Base drop speed of the current mode, in cells/s.
    pub fn drop_speed(&self) -> f32 {
        match self.mode {
            ModeKind::Race => self.race.drop_speed,
            ModeKind::Survival => self.survival.drop_speed,
            ModeKind::Puzzle => self.puzzle.drop_speed,
        }
    }

    pub fn from_ron(text: &str) -> Result<Self, ron::error::SpannedError> {
        ron::from_str(text)
    }

    /// Loads the config from `path`, falling back to defaults if the file is
    /// missing or invalid (the error is logged, not fatal).
    pub fn load_or_default(path: impl AsRef<Path>) -> Self {
        let path = path.as_ref();
        match std::fs::read_to_string(path) {
            Ok(text) => match Self::from_ron(&text) {
                Ok(config) => config.validated(),
                Err(err) => {
                    error!("invalid config {}: {err}; using defaults", path.display());
                    Self::default()
                }
            },
            Err(_) => {
                warn!("no config at {}; using defaults", path.display());
                Self::default()
            }
        }
    }

    /// Replaces values that would break the game with defaults.
    fn validated(mut self) -> Self {
        self.pieces.retain(|p| !p.cells.is_empty());
        if self.pieces.is_empty() {
            warn!("config has no usable pieces; using the default set");
            self.pieces = default_pieces();
        }
        if self.cell_size <= 0.0 {
            self.cell_size = Self::default().cell_size;
        }
        self.magic.line_spacing = self.magic.line_spacing.max(0.5);
        self.magic.mystery_pieces.retain(|p| !p.cells.is_empty());
        self.rotation_steps = self.rotation_steps.max(1);
        if self.physics_hz <= 0.0 {
            self.physics_hz = Self::default().physics_hz;
        }
        self
    }

    /// Converts a length in cells to world units.
    pub fn cells(&self, n: f32) -> f32 {
        n * self.cell_size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shipped_config_matches_defaults() {
        let text = include_str!("../assets/config/game.ron");
        let config = GameConfig::from_ron(text).expect("assets/config/game.ron should parse");
        assert_eq!(config, GameConfig::default());
    }

    /// Rewrites the shipped config from the defaults:
    /// `TT_WRITE_CONFIG=1 cargo test write_shipped_config`. Does nothing otherwise,
    /// so `cargo test -- --ignored` never touches the file.
    #[test]
    fn write_shipped_config() {
        if std::env::var_os("TT_WRITE_CONFIG").is_none() {
            return;
        }
        let pretty = ron::ser::PrettyConfig::new()
            .depth_limit(3)
            .struct_names(false)
            .separate_tuple_members(false);
        let body = ron::ser::to_string_pretty(&GameConfig::default(), pretty).unwrap();
        let text = format!(
            "// Game configuration, Normal difficulty. Every field is optional: anything left\n\
             // out uses the built-in default. Lengths are in brick cells and times in seconds.\n\
             // What each field means is documented in src/config.rs; where the numbers come\n\
             // from is in docs/research/.\n{body}\n"
        );
        std::fs::write(
            concat!(env!("CARGO_MANIFEST_DIR"), "/assets/config/game.ron"),
            text,
        )
        .unwrap();
    }

    #[test]
    fn partial_config_keeps_defaults() {
        let config = GameConfig::from_ron("(gravity: 9.0, puzzle: (bricks: 4))").unwrap();
        assert_eq!(config.gravity, 9.0);
        assert_eq!(config.puzzle.bricks, 4);
        assert_eq!(
            config.puzzle.line_height,
            PuzzleRules::default().line_height
        );
        assert_eq!(config.pieces, default_pieces());
    }

    #[test]
    fn pieces_are_centered() {
        for piece in default_pieces() {
            let cells = piece.centered_cells();
            let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
            let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
            assert_eq!(min + max, Vec2::ZERO, "piece {} is off-centre", piece.name);
        }
    }
}
