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
    /// The difficulty the menu picked; when set, its values from `levels` replace the
    /// mode rules at the start of each match. Left out, the rules are used as written.
    #[serde(default)]
    pub difficulty: Option<Difficulty>,
    #[serde(default)]
    pub levels: Levels,
    /// Human players in the next match (1 to 4); player 1 uses the keyboard,
    /// the others gamepads.
    pub players: u32,
    /// Number of AI opponents in the next match.
    pub opponents: u32,
    pub race: RaceRules,
    pub survival: SurvivalRules,
    pub puzzle: PuzzleRules,
    #[serde(default)]
    pub endless: EndlessRules,
    pub magic: MagicRules,
    pub ai: AiRules,
    #[serde(default)]
    pub wizards: WizardLook,
    #[serde(default)]
    pub moments: MatchMoments,
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
    /// Build as high as you can before a rising line catches the top of your tower.
    Endless,
}

impl ModeKind {
    pub const ALL: [ModeKind; 4] = [
        ModeKind::Race,
        ModeKind::Survival,
        ModeKind::Puzzle,
        ModeKind::Endless,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ModeKind::Race => "Race",
            ModeKind::Survival => "Survival",
            ModeKind::Puzzle => "Puzzle",
            ModeKind::Endless => "Endless Race",
        }
    }

    /// The mode whose art and music this one uses: Endless Race is dressed as a race.
    pub fn look(self) -> ModeKind {
        match self {
            ModeKind::Endless => ModeKind::Race,
            mode => mode,
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
    /// Everyone gets the same bricks in the same order (Pro).
    pub same_bricks: bool,
    /// Spell lines also hand out dark spells (not on Easy).
    pub dark_magic: bool,
    /// Wind (Pro): its strength swings between these two on a sine cycle. It blows one
    /// way, picked at random, for the whole match. Left out, there is no wind.
    pub wind: Option<(f32, f32)>,
    /// The hold countdown (3, 2, 1) shows this far above the finish line for the
    /// leading tower, and for the others.
    #[serde(default = "default_countdown_above")]
    pub countdown_above: (f32, f32),
    /// Seconds for one swing of the wind's strength.
    pub wind_period: f32,
    /// Force per point of strength on the first brick each wind row hits.
    pub wind_force: f32,
    /// Wind rows, 1 cell apart from half a cell under the tower top down; each row is
    /// this much weaker than the one above.
    pub wind_rows: u32,
    pub wind_row_falloff: f32,
    /// Each row's ray starts this far upwind of the lane centre (and reaches as far past).
    pub wind_reach: f32,
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
            same_bricks: false,
            dark_magic: true,
            wind: None,
            countdown_above: default_countdown_above(),
            wind_period: 5.0,
            wind_force: 0.8,
            wind_rows: 10,
            wind_row_falloff: 0.02,
            wind_reach: 10.0,
        }
    }
}

/// Endless Race: a pacer line rises from below and chases the top of the tower.
/// Every spell line a player picks up is a checkpoint: their pacer stops, eases back
/// down a little, waits, then rises faster. Values from the original.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct EndlessRules {
    /// The pacer slides in from here to `start_height` when the run starts...
    pub intro_from: f32,
    pub intro_seconds: f32,
    /// ...and starts rising from here, in cells from the platform top.
    pub start_height: f32,
    /// Pacer speed at the start and its ceiling, in cells/s.
    pub start_speed: f32,
    pub max_speed: f32,
    /// Speed added at the first checkpoint; each checkpoint adds this much more to
    /// the step, up to `max_step`.
    pub first_step: f32,
    pub step_growth: f32,
    pub max_step: f32,
    /// Spell lines (checkpoints) are this far apart...
    pub line_gap: f32,
    /// ...and above this height each gap is one cell longer than the last.
    pub line_gap_grows_after: f32,
    /// At a checkpoint the pacer eases down this many cells over `ease_seconds`, then
    /// waits `wait_seconds` before rising again.
    pub ease_drop: f32,
    pub ease_seconds: f32,
    pub wait_seconds: f32,
    /// Score per cell of tower height (rounded up).
    pub score_per_cell: f32,
    /// Base drop speed, in cells/s (?).
    pub drop_speed: f32,
}

impl Default for EndlessRules {
    fn default() -> Self {
        Self {
            intro_from: -15.0,
            intro_seconds: 1.0,
            start_height: -5.0,
            start_speed: 0.2,
            max_speed: 1.5,
            first_step: 0.05,
            step_growth: 0.01,
            max_step: 0.1,
            line_gap: 10.0,
            line_gap_grows_after: 150.0,
            ease_drop: 2.0,
            ease_seconds: 1.0,
            wait_seconds: 2.0,
            score_per_cell: 10.0,
            drop_speed: 3.0,
        }
    }
}

impl EndlessRules {
    /// Height of spell line `n` (0 for the first), in cells: 10, 20, ... 150, 161, 173, ...
    pub fn line_height(&self, n: u32) -> f32 {
        let (mut height, mut gap) = (0.0, self.line_gap);
        for _ in 0..=n {
            if height >= self.line_gap_grows_after {
                gap += 1.0;
            }
            height += gap;
        }
        height
    }

    /// Score for a tower this tall.
    pub fn score(&self, height: f32) -> u32 {
        (height.max(0.0) * self.score_per_cell).ceil() as u32
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
    /// Everyone gets the same bricks in the same order (Pro).
    pub same_bricks: bool,
    /// Spell lines also hand out dark spells (Normal only).
    pub dark_magic: bool,
    /// Curse waves hit everyone instead of dark spells (Pro, "Magic Waves").
    pub curse_waves: bool,
    pub waves: WaveRules,
}

impl Default for SurvivalRules {
    fn default() -> Self {
        Self {
            lives: 3,
            bricks_to_place: 66,
            drop_speed: 2.0,
            invincible_seconds: 1.5,
            finish_countdown: 3.0,
            same_bricks: false,
            dark_magic: true,
            curse_waves: false,
            waves: WaveRules::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PuzzleRules {
    /// Bricks to place.
    pub bricks: u32,
    /// Lowest laser height, in cells. Any settled brick above it ends your round.
    pub line_height: f32,
    /// Each round the laser is up to this much higher, at random (the same for everyone).
    pub line_random: f32,
    /// Seconds everything must rest below the line after the last brick lands.
    pub settle_seconds: f32,
    /// Base drop speed, in cells/s (?).
    pub drop_speed: f32,
    /// The light spell every player starts with; there are no spell lines.
    pub start_spell: Option<SpellKind>,
    /// A dropped brick costs no life; the laser comes this much lower instead.
    pub drop_penalty: f32,
    /// The floor each round is built on, in place of a flat base.
    pub floor: PuzzleFloor,
    /// How far down the floor's columns reach, in cells.
    pub floor_depth: f32,
    /// Easy and Normal floors: columns, chance (%) a middle column is missing, and
    /// how many cells lower than the top a column can be.
    pub easy_floor: (u32, u32, u32),
    pub normal_floor: (u32, u32, u32),
    /// Pro floor: a pillar this wide, with a 1-cell ledge on this share (%) of rows.
    pub pro_floor: (u32, u32),
    /// The lane is this much wider than in the other modes...
    #[serde(default = "default_puzzle_lane_scale")]
    pub lane_scale: f32,
    /// ...and a tower wider than that steers out to its widest settled brick plus this.
    #[serde(default = "default_puzzle_lane_margin")]
    pub lane_margin: f32,
}

fn default_puzzle_lane_scale() -> f32 {
    1.2
}

fn default_countdown_above() -> (f32, f32) {
    (10.0, 7.0)
}

fn default_puzzle_lane_margin() -> f32 {
    1.5
}

impl Default for PuzzleRules {
    fn default() -> Self {
        Self {
            bricks: 10,
            line_height: 3.2,
            line_random: 4.0,
            settle_seconds: 2.0,
            drop_speed: 3.0,
            start_spell: Some(SpellKind::Ivy),
            drop_penalty: 0.5,
            floor: PuzzleFloor::Normal,
            floor_depth: 6.0,
            easy_floor: (5, 0, 2),
            normal_floor: (6, 40, 4),
            pro_floor: (3, 70),
            lane_scale: 1.2,
            lane_margin: 1.5,
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
    /// Dark: skulls carry off the last placed brick and drop it beside the tower.
    Ghost,
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
            SpellKind::Ghost => "Ghost",
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
    /// Times a cursed or Mystery brick that is lost comes back where it fell.
    pub respawn_limit: u32,
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
    /// Ghost: seconds the brick is held, and how far up and outward it is carried.
    pub ghost_seconds: f32,
    pub ghost_lift: f32,
    pub ghost_carry: f32,
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
                // Endless Race: light spells only.
                AwardRule {
                    mode: Some(ModeKind::Endless),
                    light: vec![(Ivy, 2), (Undo, 2), (Petrify, 1)],
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
                // Three rare combos still to come.
                set(
                    "survival_stone",
                    vec![
                        c(&[NoRotate], 7, None),
                        c(&[Mist], 10, None),
                        c(&[Fast], 10, None),
                        c(&[Ghost], 10, None),
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
            respawn_limit: 3,
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
            ghost_seconds: 4.0,
            ghost_lift: 3.0,
            ghost_carry: 3.0,
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

/// The original's three difficulties. Each sets the values in [`Levels`].
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Difficulty {
    Easy,
    #[default]
    Normal,
    Pro,
}

impl Difficulty {
    pub const ALL: [Difficulty; 3] = [Difficulty::Easy, Difficulty::Normal, Difficulty::Pro];

    pub fn name(self) -> &'static str {
        match self {
            Difficulty::Easy => "Easy",
            Difficulty::Normal => "Normal",
            Difficulty::Pro => "Pro",
        }
    }
}

/// How the Puzzle floor is built each round (see docs/research/modes.md).
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PuzzleFloor {
    /// Just the base.
    Plain,
    /// 5 columns, each up to 2 cells lower than the top.
    Easy,
    /// 6 columns, 40% of them missing, each up to 4 cells lower.
    #[default]
    Normal,
    /// A 3-wide pillar with 1-cell ledges sticking out on 70% of the rows.
    Pro,
}

/// The values one difficulty changes, per mode.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Level {
    pub race_finish_height: f32,
    pub race_finish_drop_after: f32,
    pub race_drop_speed: f32,
    pub race_same_bricks: bool,
    pub race_dark_magic: bool,
    /// Race wind strength range, if any.
    #[serde(default)]
    pub race_wind: Option<(f32, f32)>,
    pub survival_bricks: u32,
    pub survival_drop_speed: f32,
    pub survival_dark_magic: bool,
    pub survival_same_bricks: bool,
    pub survival_curse_waves: bool,
    /// Lowest laser height and how much higher it can be.
    pub puzzle_laser: (f32, f32),
    pub puzzle_start_spell: Option<SpellKind>,
    pub puzzle_floor: PuzzleFloor,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Levels {
    pub easy: Level,
    pub normal: Level,
    pub pro: Level,
}

impl Default for Levels {
    /// The original's values (docs/research/modes.md and spell-awards.md).
    fn default() -> Self {
        let normal = Level {
            race_finish_height: 45.0,
            race_finish_drop_after: 120.0,
            race_drop_speed: 3.0,
            race_same_bricks: false,
            race_dark_magic: true,
            race_wind: None,
            survival_bricks: 66,
            survival_drop_speed: 2.0,
            survival_dark_magic: true,
            survival_same_bricks: false,
            survival_curse_waves: false,
            puzzle_laser: (3.2, 4.0),
            puzzle_start_spell: Some(SpellKind::Ivy),
            puzzle_floor: PuzzleFloor::Normal,
        };
        Self {
            easy: Level {
                race_finish_height: 26.0,
                race_finish_drop_after: 90.0,
                race_drop_speed: 2.0,
                race_dark_magic: false,
                survival_bricks: 33,
                survival_dark_magic: false,
                puzzle_laser: (5.2, 2.0),
                puzzle_start_spell: Some(SpellKind::Undo),
                puzzle_floor: PuzzleFloor::Easy,
                ..normal.clone()
            },
            pro: Level {
                race_finish_height: 38.0,
                race_drop_speed: 4.0,
                race_same_bricks: true,
                race_wind: Some((3.0, 4.0)),
                survival_drop_speed: 4.0,
                survival_dark_magic: false,
                survival_same_bricks: true,
                survival_curse_waves: true,
                puzzle_start_spell: None,
                puzzle_floor: PuzzleFloor::Pro,
                ..normal.clone()
            },
            normal,
        }
    }
}

/// Survival Pro's curse waves: every so often a curse hits everyone for a while.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct WaveRules {
    /// Seconds between waves, random in this range.
    pub every: (f32, f32),
    /// Waves before the next level applies.
    pub waves_per_level: u32,
    /// Harder and harder levels; the last one stays.
    pub levels: Vec<WaveLevel>,
    /// The curses a wave can bring, and the shortest and longest each lasts, in seconds.
    pub curses: Vec<(SpellKind, f32, f32)>,
}

/// One level of curse waves.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct WaveLevel {
    /// How many different curses a wave brings at once.
    pub curses: u32,
    /// Seconds each wave lasts is its curse's range times this.
    pub length_scale: f32,
}

impl Default for WaveRules {
    /// Every 12 to 18 s; every 2 waves the next of 4 levels, with longer waves and
    /// double or triple curses. The per-level numbers are a guess (?) until read
    /// from the original.
    fn default() -> Self {
        Self {
            every: (12.0, 18.0),
            waves_per_level: 2,
            levels: vec![
                WaveLevel {
                    curses: 1,
                    length_scale: 1.0,
                },
                WaveLevel {
                    curses: 1,
                    length_scale: 1.25,
                },
                WaveLevel {
                    curses: 2,
                    length_scale: 1.25,
                },
                WaveLevel {
                    curses: 3,
                    length_scale: 1.5,
                },
            ],
            // Large and Mist are the original's; the rest are a guess (?).
            curses: vec![
                (SpellKind::Large, 6.0, 8.0),
                (SpellKind::Mist, 14.0, 14.0),
                (SpellKind::Fast, 6.0, 8.0),
                (SpellKind::Ice, 6.0, 8.0),
                (SpellKind::NoRotate, 6.0, 8.0),
                (SpellKind::AutoRotate, 6.0, 8.0),
                (SpellKind::Mystery, 6.0, 8.0),
            ],
        }
    }
}

/// How the end of a match looks: banners, spotlights and the trophy. Values from the
/// original (docs/research/match-moments.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct MatchMoments {
    /// Size of banner text (WINNER!, WIZARD OUT!) and of the countdown, in pixels.
    pub banner_size: f32,
    pub countdown_size: f32,
    /// Shadow under match text, as a share of the text size, and its colour (0-255).
    pub shadow_drop: f32,
    pub shadow_color: [u8; 3],
    /// WINNER! takes this long to move in.
    pub winner_in: f32,
    /// WIZARD OUT! moves in, stays, then moves out, in seconds.
    pub out_times: (f32, f32, f32),
    /// The winner's beam: seconds before it fades in, the fade, its width in cells, and
    /// its opacity in Race, Survival and Puzzle.
    pub spotlight_delay: f32,
    pub spotlight_fade: f32,
    pub spotlight_width: f32,
    pub spotlight_alpha: (f32, f32, f32),
    /// The dark veil over losing lanes: width in cells and final opacity.
    pub veil_width: f32,
    pub veil_alpha: f32,
    /// The trophy pops from this share of its size to full over this many seconds.
    pub trophy_pop: (f32, f32),
    /// Degrees a second the sunrays behind WINNER! turn (?).
    pub sunray_spin: f32,
    /// Seconds after a match ends before the "Game Completed" popup (?).
    pub popup_after: f32,
}

impl Default for MatchMoments {
    fn default() -> Self {
        Self {
            banner_size: 96.0,
            countdown_size: 72.0,
            shadow_drop: 0.06,
            shadow_color: [227, 33, 61],
            winner_in: 2.6,
            out_times: (2.0, 2.0, 1.0),
            spotlight_delay: 0.5,
            spotlight_fade: 1.0,
            spotlight_width: 1.5,
            spotlight_alpha: (151.0 / 255.0, 130.0 / 255.0, 1.0),
            veil_width: 10.0,
            veil_alpha: 0.1,
            trophy_pop: (0.4, 0.6),
            sunray_spin: 20.0,
            popup_after: 4.0,
        }
    }
}

/// Where the wizards float and how lively they are (see docs/research/wizards.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct WizardLook {
    /// Cells right of the lane centre the wizard floats at.
    pub offset_x: f32,
    /// The same with one tower: further out on the left, facing the tower.
    pub solo_offset_x: f32,
    /// Cells above the tower's height.
    pub lift: f32,
    /// Bob sideways and up and down, in cells (sideways at 1×time, up at 2×time).
    pub bob: (f32, f32),
    /// The wizard plays Action on every n-th placed brick, n drawn from this range.
    pub action_every: (u32, u32),
    /// Seconds before another Laugh or Startle may play.
    pub reaction_cooldown: f32,
    /// Shake on losing a life: cells sideways and up, and seconds.
    pub shake: (f32, f32, f32),
}

impl Default for WizardLook {
    fn default() -> Self {
        Self {
            offset_x: 5.5,
            solo_offset_x: -11.0,
            lift: 4.0,
            bob: (0.5, 0.55),
            action_every: (3, 8),
            reaction_cooldown: 7.0,
            shake: (0.1, 0.13, 0.2),
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
            difficulty: None,
            levels: Levels::default(),
            players: 1,
            opponents: 0,
            race: RaceRules::default(),
            survival: SurvivalRules::default(),
            puzzle: PuzzleRules::default(),
            endless: EndlessRules::default(),
            magic: MagicRules::default(),
            ai: AiRules::default(),
            wizards: WizardLook::default(),
            moments: MatchMoments::default(),
            pieces: default_pieces(),
        }
    }
}

impl GameConfig {
    /// Copies the picked difficulty's values into the mode rules (nothing if none
    /// is picked).
    pub fn apply_difficulty(&mut self) {
        let Some(difficulty) = self.difficulty else {
            return;
        };
        let level = match difficulty {
            Difficulty::Easy => self.levels.easy.clone(),
            Difficulty::Normal => self.levels.normal.clone(),
            Difficulty::Pro => self.levels.pro.clone(),
        };
        self.race.finish_height = level.race_finish_height;
        self.race.finish_drop_after = level.race_finish_drop_after;
        self.race.drop_speed = level.race_drop_speed;
        self.race.same_bricks = level.race_same_bricks;
        self.race.dark_magic = level.race_dark_magic;
        self.race.wind = level.race_wind;
        self.survival.bricks_to_place = level.survival_bricks;
        self.survival.drop_speed = level.survival_drop_speed;
        self.survival.dark_magic = level.survival_dark_magic;
        self.survival.same_bricks = level.survival_same_bricks;
        self.survival.curse_waves = level.survival_curse_waves;
        (self.puzzle.line_height, self.puzzle.line_random) = level.puzzle_laser;
        self.puzzle.start_spell = level.puzzle_start_spell;
        self.puzzle.floor = level.puzzle_floor;
    }

    /// Whether spell lines hand out dark spells in `mode`.
    pub fn dark_magic(&self, mode: ModeKind) -> bool {
        match mode {
            ModeKind::Race => self.race.dark_magic,
            ModeKind::Survival => self.survival.dark_magic,
            ModeKind::Puzzle | ModeKind::Endless => false,
        }
    }

    /// Whether everyone gets the same bricks in `mode`.
    pub fn same_bricks(&self, mode: ModeKind) -> bool {
        match mode {
            ModeKind::Race => self.race.same_bricks,
            ModeKind::Survival => self.survival.same_bricks,
            ModeKind::Puzzle | ModeKind::Endless => true,
        }
    }

    /// How far either side of its lane centre a steered brick may go in the current
    /// mode, in cells, for a tower whose widest settled brick reaches `tower_half_width`
    /// from the centre. Puzzle lanes are wider and grow with the tower.
    pub fn lane_reach(&self, tower_half_width: f32) -> f32 {
        match self.mode {
            ModeKind::Puzzle => (self.lane_half_width * self.puzzle.lane_scale)
                .max(tower_half_width + self.puzzle.lane_margin),
            _ => self.lane_half_width,
        }
    }

    /// Base drop speed of the current mode, in cells/s.
    pub fn drop_speed(&self) -> f32 {
        match self.mode {
            ModeKind::Race => self.race.drop_speed,
            ModeKind::Survival => self.survival.drop_speed,
            ModeKind::Puzzle => self.puzzle.drop_speed,
            ModeKind::Endless => self.endless.drop_speed,
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
