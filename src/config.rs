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
    /// How the screen is split into one lane per tower.
    pub lanes: Lanes,
    /// How the title tiles move.
    pub menu: MenuLook,
    /// How the start screen's logo comes in.
    pub start: StartLook,
    /// The splash when a brick is lost in the water.
    pub splash: SplashLook,
    /// Cast beams, lightning, spell clouds and the Large grow.
    pub casts: CastLook,
    /// Experience per match and the wizard levels it buys.
    pub xp: XpRules,
    /// Local cups (guesses until read).
    pub cups: CupRules,
    /// Per-mode skies and weather: Race Pro night, Puzzle sandstorm and eclipse,
    /// Survival curse-wave sky.
    pub weather: WeatherLook,
    /// What the water does to a lost brick.
    pub water: WaterRules,
    /// Each base's collider, as read from the original.
    pub bases: Bases,
    pub roofs: Roofs,
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
    /// Endless Survival and Endless Puzzle (guesses until read).
    pub endless_play: EndlessPlay,
    /// The Trials, in order.
    pub trials: Vec<TrialDef>,
    /// The trials' own floors by the original's name (docs/research/trials-obstacles-roofs.md).
    pub trial_floors: std::collections::BTreeMap<String, TrialFloor>,
    /// The trial tiers: each holds `TrialTiers::size` trials in order and opens at a
    /// medal count.
    pub trial_tiers: TrialTiers,
    /// Each tower's wizard (0 for wizard 1) and brick pack, by tower index; set from
    /// Select Players. Towers without an entry use wizard = index and the default pack.
    #[serde(skip)]
    pub skins: Vec<(usize, BrickPack)>,
    /// The trial being played, as an index into `trials`; set by the Trials screen
    /// and cleared by the other menus. Not saved.
    #[serde(skip)]
    pub trial: Option<usize>,
    pub magic: MagicRules,
    pub ai: AiRules,
    #[serde(default)]
    pub wizards: WizardLook,
    #[serde(default)]
    pub moments: MatchMoments,
    #[serde(default)]
    pub hud: HudLook,
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
    /// Survival with no brick limit: place as many bricks as you can before the
    /// lives run out (public rules; details in [`EndlessPlay`] are guesses).
    EndlessSurvival,
    /// Puzzle that goes on: keep placing bricks under a laser that rises, until a
    /// brick crosses it (public rules; details in [`EndlessPlay`] are guesses).
    EndlessPuzzle,
}

impl ModeKind {
    pub const ALL: [ModeKind; 6] = [
        ModeKind::Race,
        ModeKind::Survival,
        ModeKind::Puzzle,
        ModeKind::Endless,
        ModeKind::EndlessSurvival,
        ModeKind::EndlessPuzzle,
    ];

    pub fn name(self) -> &'static str {
        match self {
            ModeKind::Race => "Race",
            ModeKind::Survival => "Survival",
            ModeKind::Puzzle => "Puzzle",
            ModeKind::Endless => "Endless Race",
            ModeKind::EndlessSurvival => "Endless Survival",
            ModeKind::EndlessPuzzle => "Endless Puzzle",
        }
    }

    /// The mode whose art and music this one uses: each Endless mode is dressed
    /// as its battle mode.
    pub fn look(self) -> ModeKind {
        match self {
            ModeKind::Endless => ModeKind::Race,
            ModeKind::EndlessSurvival => ModeKind::Survival,
            ModeKind::EndlessPuzzle => ModeKind::Puzzle,
            mode => mode,
        }
    }

    /// The mode whose rules this one plays by. Endless Survival and Endless Puzzle
    /// follow Survival and Puzzle apart from how they end; Endless Race has its own.
    pub fn rules(self) -> ModeKind {
        match self {
            ModeKind::EndlessSurvival => ModeKind::Survival,
            ModeKind::EndlessPuzzle => ModeKind::Puzzle,
            mode => mode,
        }
    }

    /// The single-player Endless modes, scored on a leaderboard.
    pub fn is_endless(self) -> bool {
        matches!(
            self,
            ModeKind::Endless | ModeKind::EndlessSurvival | ModeKind::EndlessPuzzle
        )
    }

    /// Row on the Endless screen and leaderboard: Race, Survival, Puzzle.
    pub fn endless_index(self) -> Option<usize> {
        match self {
            ModeKind::Endless => Some(0),
            ModeKind::EndlessSurvival => Some(1),
            ModeKind::EndlessPuzzle => Some(2),
            _ => None,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct RaceRules {
    /// Finish line height above the platform, in cells.
    pub finish_height: f32,
    /// Seconds the tower must stay past the line to win: the original's 3-2-1
    /// countdown, one number a second.
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
    /// Base drop speed, in cells/s (the original's Endless Race: 4).
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
            drop_speed: 4.0,
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

/// Endless Survival and Endless Puzzle. Built from public descriptions of the
/// original (three lives, as many bricks as you can; puzzle rules until you fail),
/// so every value here is a guess `(?)` until it is read from the install
/// (queued in docs/pc-tasks.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct EndlessPlay {
    /// Endless Survival (docs/research/modes.md): lives are the game's maximum
    /// health setting, 3.
    pub survival_lives: u32,
    pub survival_drop_speed: f32,
    /// Spell lines give one of these at random, light only.
    pub survival_light: Vec<SpellKind>,
    /// Curse waves 15 to 29 s apart, each lasting a number of bricks.
    pub survival_waves: WaveRules,
    /// Endless Puzzle: one life, drop speed 2, the laser starts at 6.2.
    pub puzzle_drop_speed: f32,
    pub laser_start: f32,
    /// Level 1 allows `first_bricks`; each level after allows
    /// min(`most_bricks`, `first_bricks` + `bricks_per_level` × level) more.
    pub first_bricks: u32,
    pub bricks_per_level: u32,
    pub most_bricks: u32,
    /// A level-up: wait, the laser rises `laser_rise` cells over `rise_seconds`,
    /// wait again, then the laser zaps and `level_spell` is given.
    pub laser_rise: f32,
    pub rise_seconds: f32,
    pub level_wait: (f32, f32),
    pub level_spell: Option<SpellKind>,
    /// After the rise, `obstacles` 1-cell blocks pop in `obstacle_delay` s apart
    /// (before the second wait), each drawn by weight from the free spots: rows
    /// from the tower top + 1 up to the new laser, columns at these (x from the
    /// lane centre, weight) on both sides (docs/research/trials-obstacles-roofs.md).
    pub obstacles: u32,
    pub obstacle_delay: f32,
    pub obstacle_columns: Vec<(f32, f32)>,
    /// The Pro ledge look (0-2) of each of the original's 11 block pieces.
    pub obstacle_looks: Vec<u32>,
}

impl Default for EndlessPlay {
    fn default() -> Self {
        use SpellKind::{Fast, Ghost, Ice, Large, Mist, Mystery, NoRotate as NoTurn};
        let w = Wave::bricks;
        Self {
            survival_lives: 3,
            survival_drop_speed: 4.0,
            survival_light: vec![SpellKind::Ivy, SpellKind::Minibase, SpellKind::Petrify],
            survival_waves: WaveRules {
                gap: (15, 30),
                levels: vec![
                    vec![
                        w(&[Large], 2),
                        w(&[Mist], 3),
                        w(&[Mystery], 2),
                        w(&[Fast], 4),
                        w(&[NoTurn], 3),
                    ],
                    vec![
                        w(&[Large], 3),
                        w(&[Mystery], 3),
                        w(&[Ice], 3),
                        w(&[Fast], 5),
                        w(&[NoTurn], 4),
                        w(&[Fast, Large], 2),
                        w(&[Mystery, NoTurn], 2),
                        w(&[Mist, Large], 2),
                    ],
                    vec![
                        w(&[Large], 4),
                        w(&[Ice], 4),
                        w(&[NoTurn], 5),
                        w(&[Ghost], 2),
                        w(&[Mist, NoTurn], 4),
                        w(&[Mystery, Ice], 3),
                        w(&[NoTurn, Fast], 4),
                        w(&[NoTurn, Large], 3),
                    ],
                    vec![
                        w(&[Fast], 5),
                        w(&[Large], 7),
                        w(&[NoTurn], 6),
                        w(&[Ice], 5),
                        w(&[Mystery], 6),
                        w(&[Ghost, Fast], 3),
                        w(&[Ice, Large], 4),
                        w(&[Mystery, Ice], 5),
                        w(&[Mist, Mystery], 5),
                        w(&[Large, Ghost], 5),
                        w(&[Mystery, Large, NoTurn], 4),
                        w(&[Large, Fast, NoTurn], 5),
                    ],
                ],
                // Lift, hold, shake and waves per level as in Survival Pro (?).
                ..WaveRules::default()
            },
            puzzle_drop_speed: 2.0,
            laser_start: 6.2,
            first_bricks: 6,
            bricks_per_level: 2,
            most_bricks: 26,
            laser_rise: 4.0,
            rise_seconds: 0.5,
            level_wait: (0.3, 0.3),
            level_spell: Some(SpellKind::Undo),
            obstacles: 4,
            obstacle_delay: 0.1,
            obstacle_columns: vec![
                (6.5, 2.0),
                (5.5, 0.5),
                (4.5, 0.3),
                (3.5, 0.2),
                (2.5, 0.1),
                (1.5, 0.05),
                (0.5, 0.01),
            ],
            obstacle_looks: vec![0, 1, 2, 0, 1, 2, 2, 1, 0, 1, 0],
        }
    }
}

impl EndlessPlay {
    /// Bricks allowed in Endless Puzzle level `level` (1 is the first).
    pub fn level_bricks(&self, level: u32) -> u32 {
        if level <= 1 {
            return self.first_bricks;
        }
        (self.first_bricks + self.bricks_per_level * (level - 1)).min(self.most_bricks)
    }

    /// Seconds a level-up lasts: wait, rise, the blocks popping in, wait.
    pub fn level_up_seconds(&self) -> f32 {
        self.level_wait.0
            + self.rise_seconds
            + self.obstacles as f32 * self.obstacle_delay
            + self.level_wait.1
    }

    /// Where the level-up's blocks go: cell centres (x from the lane centre, y),
    /// each block filling the cell under a row line from `tower + 1` up to the
    /// laser. `roll` gives numbers in 0..1.
    pub fn obstacle_spots(
        &self,
        tower: f32,
        laser: f32,
        mut roll: impl FnMut() -> f32,
    ) -> Vec<Vec2> {
        let (low, high) = (tower.ceil() as i32 + 1, laser.floor() as i32);
        let mut spots: Vec<(Vec2, f32)> = Vec::new();
        for &(x, weight) in &self.obstacle_columns {
            for row in low..=high {
                for side in [x, -x] {
                    spots.push((Vec2::new(side, row as f32 - 0.5), weight));
                }
            }
        }
        let mut picked = Vec::new();
        for _ in 0..self.obstacles {
            let total: f32 = spots.iter().map(|s| s.1).sum();
            if spots.is_empty() || total <= 0.0 {
                break;
            }
            let target = roll() * total;
            let mut sum = 0.0;
            let i = spots
                .iter()
                .position(|s| {
                    sum += s.1;
                    sum >= target
                })
                .unwrap_or(spots.len() - 1);
            picked.push(spots.remove(i).0);
        }
        picked
    }

    /// Puts Endless Survival's or Endless Puzzle's rules over the battle mode's.
    pub fn apply(&self, config: &mut GameConfig) {
        match config.mode {
            ModeKind::EndlessSurvival => {
                let rules = &mut config.survival;
                rules.lives = self.survival_lives;
                rules.drop_speed = self.survival_drop_speed;
                rules.bricks_to_place = u32::MAX;
                rules.dark_magic = false;
                rules.curse_waves = true;
                rules.waves = self.survival_waves.clone();
                config.magic.award_rules = vec![AwardRule {
                    light: self.survival_light.iter().map(|&s| (s, 1)).collect(),
                    ..default()
                }];
            }
            ModeKind::EndlessPuzzle => {
                let rules = &mut config.puzzle;
                rules.drop_speed = self.puzzle_drop_speed;
                rules.line_height = self.laser_start;
                rules.line_random = 0.0;
                rules.start_spell = None;
            }
            _ => {}
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
    /// Base drop speed, in cells/s (2 on every difficulty in the original).
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
            drop_speed: 2.0,
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

/// One Trial: a single-player challenge in Race, Survival or Puzzle, with an enemy
/// wizard who doesn't build but casts dark spells on a schedule
/// (docs/research/modes.md). The shipped list is the original's 50
/// (src/trial_list.rs).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct TrialDef {
    pub name: String,
    /// Race, Survival or Puzzle.
    pub mode: ModeKind,
    /// Race: the finish height and the seconds you have to hold it by.
    pub finish_height: Option<f32>,
    pub seconds: Option<f32>,
    /// Survival and Puzzle: bricks to place. Survival: lives.
    pub bricks: Option<u32>,
    pub lives: Option<u32>,
    /// Puzzle: the laser height (no random part in a trial, ?).
    pub laser: Option<f32>,
    /// Base drop speed, if not the mode's Normal one.
    pub drop_speed: Option<f32>,
    /// A light spell in hand at the start.
    pub start_spell: Option<SpellKind>,
    /// The trial's own floor (a key of `trial_floors`), else the mode's base.
    pub floor: Option<String>,
    /// Race trials: the rival tower (1-13) standing in the enemy's lane.
    pub rival: Option<u32>,
    /// The enemy wizard's casts.
    pub enemy: Vec<EnemyCast>,
    /// A fixed brick order (shape names), else random bricks.
    pub order: Vec<String>,
    /// The trial's own spell lines, else none.
    pub lines: Option<TrialLines>,
}

/// A trial's spell lines: how far apart (cells, plus 25% per extra player), and the
/// light spell each gives, in order; `true` casts it the moment it's collected.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct TrialLines {
    pub every: f32,
    pub spells: Vec<(SpellKind, bool)>,
}

impl Default for TrialDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            mode: ModeKind::Race,
            finish_height: None,
            seconds: None,
            bricks: None,
            lives: None,
            laser: None,
            drop_speed: None,
            start_spell: None,
            floor: None,
            rival: None,
            enemy: Vec::new(),
            order: Vec::new(),
            lines: None,
        }
    }
}

/// A trial's floor: static boxes in cells (x from the lane centre, y from the base's
/// top) and the pictures drawn on it (art key, spot, mirrored, turn in degrees).
/// keep_base adds them to the mode's base instead of replacing it.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Default)]
#[serde(default)]
pub struct TrialFloor {
    pub keep_base: bool,
    pub boxes: Vec<BaseBox>,
    pub art: Vec<(String, f32, f32, bool, f32)>,
}

/// Brick packs (brick skins), from docs/research/skins.md. Each player picks one on
/// Select Players; it changes the art of that player's seven normal shapes only.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq, Hash, Default)]
pub enum BrickPack {
    #[default]
    Default,
    Christmas,
    Spring,
    Halloween,
    Candy,
    Hologram,
    Diamond,
    Galaxy,
}

impl BrickPack {
    pub const ALL: [BrickPack; 8] = [
        BrickPack::Default,
        BrickPack::Christmas,
        BrickPack::Spring,
        BrickPack::Halloween,
        BrickPack::Candy,
        BrickPack::Hologram,
        BrickPack::Diamond,
        BrickPack::Galaxy,
    ];

    pub fn name(self) -> &'static str {
        match self {
            BrickPack::Default => "Default",
            BrickPack::Christmas => "Christmas",
            BrickPack::Spring => "Spring",
            BrickPack::Halloween => "Halloween",
            BrickPack::Candy => "Candy",
            BrickPack::Hologram => "Hologram",
            BrickPack::Diamond => "Diamond",
            BrickPack::Galaxy => "Galaxy",
        }
    }

    /// The pack's word in art names (`brick_<key>_<shape>`, `next_brick_<key>_<shape>`).
    pub fn key(self) -> &'static str {
        match self {
            BrickPack::Default => "default",
            BrickPack::Christmas => "christmas",
            BrickPack::Spring => "spring",
            BrickPack::Halloween => "halloween",
            BrickPack::Candy => "candy",
            BrickPack::Hologram => "hologram",
            BrickPack::Diamond => "diamond",
            BrickPack::Galaxy => "galaxy",
        }
    }

    /// The art a normal brick of `shape` uses with this pack. Hologram draws the
    /// default textures (through its glitch effect); Galaxy has no texture of its own
    /// and is drawn as a starfield, so it starts from the default too.
    pub fn brick_art(self, shape: &str) -> String {
        match self {
            BrickPack::Default | BrickPack::Hologram | BrickPack::Galaxy => {
                format!("brick_{shape}")
            }
            pack => format!("brick_{}_{shape}", pack.key()),
        }
    }
}

/// The wizards there are (14: 4 base, 4 Spirit Animal, 6 Indie; docs/research/skins.md).
pub const WIZARD_COUNT: usize = 14;

/// The original's trial tiers (fan wiki, docs/research/wiki-notes.md): 50 trials in 5
/// tiers of 10, each opening at a number of medals. Each trial beaten is one medal,
/// out of as many medals as there are trials.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct TrialTiers {
    /// Trials per tier.
    pub size: usize,
    /// Each tier's name and the medals it needs to open, easiest first.
    pub tiers: Vec<(String, u32)>,
    /// Medals for beating a trial.
    pub medals_per_trial: u32,
}

impl Default for TrialTiers {
    fn default() -> Self {
        Self {
            size: 10,
            tiers: [
                ("Rookie", 0),
                ("Apprentice", 8),
                ("Pro", 17),
                ("Expert", 26),
                ("Master", 36),
            ]
            .map(|(name, medals)| (name.to_string(), medals))
            .to_vec(),
            medals_per_trial: 1,
        }
    }
}

impl TrialTiers {
    /// The tier trial `index` (0 for the first) belongs to; past the last tier,
    /// the last one.
    pub fn tier_of(&self, index: usize) -> usize {
        (index / self.size.max(1)).min(self.tiers.len().saturating_sub(1))
    }

    /// Whether trial `index` is open with `medals` medals.
    pub fn is_open(&self, index: usize, medals: u32) -> bool {
        self.tiers
            .get(self.tier_of(index))
            .is_none_or(|(_, need)| medals >= *need)
    }
}

/// One line of the enemy wizard's script: when it casts, and what.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct EnemyCast {
    pub when: CastWhen,
    /// Cast together, like a dark spell pair's curse.
    pub spells: Vec<SpellKind>,
}

/// When the enemy wizard casts. The original's trials use `Height` and `Brick`
/// (docs/research/trials.md); the others are extra kinds for mods and tests.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub enum CastWhen {
    /// Once, when your tower reaches this many cells.
    Height(f32),
    /// Once, on your Nth brick (counting from 1; 0 counts as the first).
    Brick(u32),
    /// Every this many seconds, first after one period.
    Every(f32),
    /// Once, this many seconds in.
    At(f32),
    /// Once, when this share (0 to 1) of the trial's bricks has been used.
    BricksUsed(f32),
}

impl TrialDef {
    /// Puts this trial's rules into the config, over the Normal difficulty's.
    pub fn apply(&self, config: &mut GameConfig) {
        config.mode = self.mode;
        config.players = 1;
        config.opponents = 0;
        match self.mode {
            ModeKind::Race => {
                if let Some(height) = self.finish_height {
                    config.race.finish_height = height;
                }
                // A trial's clock replaces the finish line coming down.
                config.race.finish_drop_after = f32::MAX;
                if let Some(speed) = self.drop_speed {
                    config.race.drop_speed = speed;
                }
            }
            ModeKind::Survival => {
                if let Some(bricks) = self.bricks {
                    config.survival.bricks_to_place = bricks;
                }
                if let Some(lives) = self.lives {
                    config.survival.lives = lives;
                }
                if let Some(speed) = self.drop_speed {
                    config.survival.drop_speed = speed;
                }
            }
            ModeKind::Puzzle => {
                if let Some(bricks) = self.bricks {
                    config.puzzle.bricks = bricks;
                }
                if let Some(laser) = self.laser {
                    config.puzzle.line_height = laser;
                    config.puzzle.line_random = 0.0;
                }
                if let Some(speed) = self.drop_speed {
                    config.puzzle.drop_speed = speed;
                }
                config.puzzle.start_spell = self.start_spell;
            }
            _ => {}
        }
    }

    /// The bricks a Survival or Puzzle trial gives, after `apply`.
    pub fn brick_target(&self, config: &GameConfig) -> Option<u32> {
        match self.mode {
            ModeKind::Survival => Some(config.survival.bricks_to_place),
            ModeKind::Puzzle => Some(config.puzzle.bricks),
            _ => None,
        }
    }
}

/// A few sample trials, one per mode, so the Trials screen and enemy wizard can be
/// played and tested. All goals and schedules are guesses (?).
pub fn sample_trials() -> Vec<TrialDef> {
    vec![
        TrialDef {
            name: "Sample Race".into(),
            mode: ModeKind::Race,
            finish_height: Some(15.0),
            seconds: Some(90.0),
            enemy: vec![EnemyCast {
                when: CastWhen::Every(25.0),
                spells: vec![SpellKind::Large],
            }],
            ..default()
        },
        TrialDef {
            name: "Sample Survival".into(),
            mode: ModeKind::Survival,
            bricks: Some(20),
            lives: Some(3),
            enemy: vec![
                EnemyCast {
                    when: CastWhen::BricksUsed(0.5),
                    spells: vec![SpellKind::Mystery],
                },
                EnemyCast {
                    when: CastWhen::At(40.0),
                    spells: vec![SpellKind::Slow],
                },
            ],
            ..default()
        },
        TrialDef {
            name: "Sample Puzzle".into(),
            mode: ModeKind::Puzzle,
            bricks: Some(8),
            laser: Some(5.0),
            start_spell: Some(SpellKind::Undo),
            ..default()
        },
    ]
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
    /// Holds only on these difficulties; empty means any.
    pub difficulties: Vec<Difficulty>,
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
    /// Extra wait per player beyond two, as a fraction, on these difficulties only.
    pub line_wait_per_extra_player: f32,
    pub line_wait_extra_on: Vec<Difficulty>,
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
    /// Minibase ledge width, in cells: the flat top of the original's ledge piece (it
    /// also has a raised lip on the outer side, not modelled).
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
    /// Auto Rotate: seconds per cycle; in each the brick winds back
    /// `auto_rotate_back` (degrees, seconds), swings `auto_rotate_swing` the other
    /// way (90° net), then waits out the cycle; both parts ease in-out elastic.
    pub auto_rotate_interval: f32,
    pub auto_rotate_back: (f32, f32),
    pub auto_rotate_swing: (f32, f32),
    /// No Rotate: a refused turn shakes the brick's picture this long, jumping up to
    /// `no_rotate_jump` cells each way every frame (divided by the brick's scale).
    pub no_rotate_shake: f32,
    pub no_rotate_jump: f32,
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
            line_wait_extra_on: vec![Difficulty::Normal, Difficulty::Pro],
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
                // Far behind is 15 cells, 20 on Pro.
                AwardRule {
                    mode: Some(ModeKind::Race),
                    difficulties: vec![Difficulty::Easy, Difficulty::Normal],
                    behind_leader: Some(15.0),
                    light: vec![(AutoBuild, 2), (Petrify, 1), (Minibase, 1)],
                    dark: Some("far_behind".into()),
                    ..default()
                },
                AwardRule {
                    mode: Some(ModeKind::Race),
                    difficulties: vec![Difficulty::Pro],
                    behind_leader: Some(20.0),
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
                    difficulties: vec![Difficulty::Easy, Difficulty::Normal],
                    light: vec![(Ivy, 10), (Undo, 10), (Petrify, 3)],
                    dark_after: vec![
                        (Ivy, "normal".into()),
                        (Undo, "normal".into()),
                        (Petrify, "stone".into()),
                    ],
                    ..default()
                },
                // Pro adds Minibase; after it comes the same set as after Petrify.
                AwardRule {
                    mode: Some(ModeKind::Race),
                    difficulties: vec![Difficulty::Pro],
                    light: vec![(Ivy, 10), (Undo, 10), (Petrify, 3), (Minibase, 3)],
                    dark_after: vec![
                        (Ivy, "normal".into()),
                        (Undo, "normal".into()),
                        (Petrify, "stone".into()),
                        (Minibase, "stone".into()),
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
            minibase_width: 2.0,
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
            auto_rotate_back: (15.0, 0.2),
            auto_rotate_swing: (105.0, 0.45),
            no_rotate_shake: 0.3,
            no_rotate_jump: 0.1,
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
    pub fn wait(&self, players: usize, difficulty: Difficulty) -> f32 {
        if !self.line_wait_extra_on.contains(&difficulty) {
            return self.line_wait;
        }
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

/// The three Mystery shapes, from the MysteryBrick prefabs' box colliders: a 2x4
/// standing block, a ten-cell blob in a 3x4 box (its 45° nub beside the top isn't a
/// cell) and the piano on two feet.
pub fn default_mystery_pieces() -> Vec<PieceDef> {
    vec![
        PieceDef::new(
            "Block",
            [0.80, 0.55, 0.85],
            &[
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1),
                (0, 2),
                (1, 2),
                (0, 3),
                (1, 3),
            ],
        )
        .with_mass(1.5),
        PieceDef::new(
            "Blob",
            [0.55, 0.80, 0.70],
            &[
                (0, 0),
                (1, 0),
                (0, 1),
                (1, 1),
                (2, 1),
                (0, 2),
                (1, 2),
                (2, 2),
                (1, 3),
                (2, 3),
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
            // Called PRO inside the original, but its screens say "Special".
            Difficulty::Pro => "Special",
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

/// Survival Pro's curse waves: every so often a fixed set of curses hits everyone
/// for a while. From the original (docs/research/modes.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct WaveRules {
    /// Whole seconds before the first wave, and from the end of one wave to the
    /// start of the next; the top value is left out, as in the original.
    pub gap: (u32, u32),
    /// Waves before the next level starts (sooner if its list runs out).
    pub waves_per_level: u32,
    /// Each level's waves, drawn without repeats; the last level repeats.
    pub levels: Vec<Vec<Wave>>,
    /// When a wave starts, the brick in hand rises over `lift` seconds (eased in,
    /// collisions off) to `lift_below_top` cells under the top of the view, then
    /// is held `hold` seconds; it shakes the whole time, jumping to a random
    /// offset of up to `shake` cells every step (less when enlarged). From the
    /// original (docs/research/modes.md).
    pub lift: f32,
    pub lift_below_top: f32,
    pub hold: f32,
    pub shake: f32,
}

/// One curse wave: its curses, and how long it lasts in whole seconds (top left out),
/// or in bricks when `bricks` isn't 0 (the brick in hand counts as the first).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct Wave {
    pub curses: Vec<SpellKind>,
    pub seconds: (u32, u32),
    #[serde(default)]
    pub bricks: u32,
}

impl Wave {
    fn new(curses: &[SpellKind], seconds: (u32, u32)) -> Self {
        Self {
            curses: curses.to_vec(),
            seconds,
            bricks: 0,
        }
    }

    /// A wave that lasts `bricks` bricks (Endless Survival) instead of seconds.
    fn bricks(curses: &[SpellKind], bricks: u32) -> Self {
        Self {
            curses: curses.to_vec(),
            seconds: (0, 0),
            bricks,
        }
    }
}

impl Default for WaveRules {
    fn default() -> Self {
        use SpellKind::{Fast, Ghost, Ice, Large, Mist, Mystery, NoRotate as NoTurn};
        let w = Wave::new;
        Self {
            gap: (12, 18),
            waves_per_level: 2,
            lift: 1.0,
            lift_below_top: 4.0,
            hold: 1.0,
            shake: 0.1,
            levels: vec![
                vec![
                    w(&[Large], (6, 8)),
                    w(&[Mist], (14, 14)),
                    w(&[Mystery], (6, 8)),
                    w(&[Fast], (6, 8)),
                    w(&[NoTurn], (6, 8)),
                ],
                vec![
                    w(&[Large], (8, 11)),
                    w(&[Mystery], (8, 11)),
                    w(&[Ice], (6, 8)),
                    w(&[Fast], (8, 11)),
                    w(&[NoTurn], (8, 11)),
                    w(&[Fast, Large], (4, 6)),
                    w(&[Mystery, NoTurn], (4, 6)),
                    w(&[Mist, Large], (6, 8)),
                ],
                vec![
                    w(&[Large], (12, 16)),
                    w(&[Ice], (8, 11)),
                    w(&[NoTurn], (11, 15)),
                    w(&[Ghost], (6, 8)),
                    w(&[Mist, NoTurn], (8, 11)),
                    w(&[Mystery, Ice], (6, 8)),
                    w(&[NoTurn, Fast], (8, 11)),
                    w(&[NoTurn, Large], (8, 11)),
                ],
                vec![
                    w(&[Fast], (12, 16)),
                    w(&[NoTurn], (12, 16)),
                    w(&[Ice], (11, 15)),
                    w(&[Mystery], (12, 16)),
                    w(&[Ghost, Fast], (6, 8)),
                    w(&[Ice, Large], (6, 8)),
                    w(&[Mystery, Ice], (8, 11)),
                    w(&[Large, Large], (6, 8)),
                    w(&[Mist, Mystery], (8, 11)),
                    w(&[Large, Ghost], (7, 10)),
                    w(&[Mystery, Large, NoTurn], (6, 8)),
                    w(&[Large, Fast, NoTurn], (6, 8)),
                ],
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
    /// The dark veil once someone wins: two mirrored strips starting `veil_gap` cells
    /// either side of the winner's lane centre, each `veil_width` cells wide, at this
    /// final opacity. When the computer beats a lone player, one `veil_solo_width`
    /// strip covers the screen instead.
    pub veil_gap: f32,
    pub veil_width: f32,
    pub veil_solo_width: f32,
    pub veil_alpha: f32,
    /// The trophy pops from this share of its size to full over this many seconds.
    pub trophy_pop: (f32, f32),
    /// Degrees a second the sunrays behind WINNER! turn (the original has two ray layers,
    /// 60 and -90; this is the first).
    pub sunray_spin: f32,
    /// Each losing tower's end sequence: seconds before it gives way; every n-th
    /// brick blows up (every brick if the wizard was already out), at most this
    /// many; the gap before each blast is (bricks still to go, at most
    /// `blast_gap_cap`) × a random time in `blast_gap`.
    pub collapse_wait: f32,
    pub blast_every: usize,
    pub blast_limit: usize,
    pub blast_gap: (f32, f32),
    pub blast_gap_cap: usize,
    /// Camera shake per blast and for the rumble after the last one: cells
    /// sideways, cells up, seconds.
    pub blast_shake: (f32, f32, f32),
    pub rumble: (f32, f32, f32),
    /// The losing player's controller rumbles this long when the sequence starts (the
    /// original: 2 s on the big motor after a 1.5 s wait, GameBaskController) and
    /// again with the rumble after the blasts (?), at this strength (0 to 1).
    pub pad_rumble: (f32, f32),
    pub pad_rumble_strength: f32,
    /// The popup opens once every loser's sequence is done, and at least this
    /// many seconds after the match ends.
    pub popup_after: f32,
    /// Seconds the popup takes to slide up into the bottom-right corner.
    pub popup_slide: f32,
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
            veil_gap: 8.0,
            veil_width: 60.0,
            veil_solo_width: 120.0,
            veil_alpha: 0.1,
            trophy_pop: (0.4, 0.6),
            sunray_spin: 60.0,
            collapse_wait: 1.5,
            blast_every: 3,
            blast_limit: 100,
            blast_gap: (0.002, 0.007),
            blast_gap_cap: 10,
            blast_shake: (0.2, 0.26, 0.05),
            rumble: (0.1, 0.13, 2.5),
            pad_rumble: (2.0, 2.5),
            pad_rumble_strength: 1.0,
            popup_after: 0.0,
            popup_slide: 0.25,
        }
    }
}

/// One plain box of a base's collider, in cells: x from the base centre, y from
/// its top (the line bricks rest on).
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq)]
pub struct BaseBox {
    pub x: (f32, f32),
    pub y: (f32, f32),
}

const fn base_box(x: (f32, f32), y: (f32, f32)) -> BaseBox {
    BaseBox { x, y }
}

/// The bases' colliders, from the original (docs/research/modes.md, "Base
/// shapes"). Endless Race uses the Race base. Puzzle builds its floor from
/// columns instead; with the plain floor it gets a `platform_width` box.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Bases {
    /// The house: a 6-wide body, a chimney cell left of the centre and a
    /// ledge on each side under the water.
    pub race: Vec<BaseBox>,
    /// The castle: three merlons with a 1-cell hole between each pair, on a
    /// body that widens and then narrows to its foot.
    pub survival: Vec<BaseBox>,
}

impl Default for Bases {
    fn default() -> Self {
        Self {
            race: vec![
                base_box((-3.0, 3.0), (-9.0, 0.0)),
                base_box((-1.0, 0.0), (0.0, 1.0)),
                base_box((-4.0, -3.0), (-9.0, -5.57)),
                base_box((3.0, 4.0), (-9.0, -5.0)),
            ],
            survival: vec![
                base_box((-3.0, -2.0), (-1.0, 0.0)),
                base_box((-1.0, 1.0), (-1.0, 0.0)),
                base_box((2.0, 3.0), (-1.0, 0.0)),
                base_box((-3.0, 3.0), (-5.0, -1.0)),
                base_box((-3.5, 3.5), (-7.0, -5.0)),
                base_box((-2.5, 3.0), (-11.0, -7.0)),
            ],
        }
    }
}

/// The winner's roof (docs/research/trials-obstacles-roofs.md, "Roofs"): the boxes
/// of `ROOF_01` (Race) and `ROOF_02` (Survival) in cells from the roof's origin,
/// where its art is drawn `art_y` cells below, its fall speed and the seconds it
/// rests before the match moves on.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Roofs {
    pub race: Vec<BaseBox>,
    pub survival: Vec<BaseBox>,
    pub race_art_y: f32,
    pub survival_art_y: f32,
    pub drop_speed: f32,
    pub rest_seconds: f32,
}

impl Default for Roofs {
    fn default() -> Self {
        // Boxes given as centre and size in the original.
        let at = |x: f32, y: f32, w: f32, h: f32| {
            base_box((x - w / 2.0, x + w / 2.0), (y - h / 2.0, y + h / 2.0))
        };
        Self {
            race: vec![
                at(-1.0, -2.911, 1.0, 1.0),
                at(0.0, -0.415, 4.5, 2.0),
                at(1.0, -2.911, 1.0, 1.0),
                at(0.0, -1.915, 3.0, 1.0),
                at(0.0, 1.085, 3.0, 1.0),
                at(0.0, 2.085, 2.0, 1.0),
                at(0.0, 3.085, 1.0, 1.0),
            ],
            survival: vec![
                at(0.0, -5.13, 1.0, 0.5),
                at(0.0, -6.88, 4.0, 3.0),
                at(1.5, -8.88, 1.0, 1.0),
                at(-1.5, -8.88, 1.0, 1.0),
            ],
            race_art_y: 0.0,
            survival_art_y: -7.13,
            drop_speed: 2.0,
            rest_seconds: 0.7,
        }
    }
}

impl GameConfig {
    /// The floor of the trial being played, if it has its own.
    pub fn trial_floor(&self) -> Option<&TrialFloor> {
        let name = self
            .trial
            .and_then(|i| self.trials.get(i))?
            .floor
            .as_ref()?;
        self.trial_floors.get(name)
    }

    /// The boxes of this mode's base, or a plain `platform_width` slab when the
    /// mode has no read shape.
    pub fn base_boxes(&self, mode: ModeKind) -> Vec<BaseBox> {
        let boxes = match mode.look() {
            ModeKind::Race => &self.bases.race,
            ModeKind::Survival => &self.bases.survival,
            _ => return vec![self.plain_base()],
        };
        if boxes.is_empty() {
            vec![self.plain_base()]
        } else {
            boxes.clone()
        }
    }

    fn plain_base(&self) -> BaseBox {
        let half = self.platform_width * 0.5;
        base_box((-half, half), (-1.0, 0.0))
    }

    /// Half the width of the base's top, in cells: the widest box reaching
    /// the top line.
    pub fn base_half_width(&self, mode: ModeKind) -> f32 {
        self.base_boxes(mode)
            .iter()
            .filter(|b| b.y.1 >= 0.0)
            .map(|b| b.x.0.abs().max(b.x.1.abs()))
            .fold(0.0, f32::max)
    }
}

/// A brick that falls below the water line (`kill_depth`) is lost; the water
/// nearly stops it (it keeps `1 - stop` of its speed and `1 - spin_damp` of its
/// spin) and lifts it with `lift` cells/s² against gravity, so it floats. From the
/// original (docs/research/controls-physics.md), except `float_seconds` (?), how
/// long it stays before it is removed.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct WaterRules {
    pub stop: f32,
    pub spin_damp: f32,
    pub lift: f32,
    pub float_seconds: f32,
}

impl Default for WaterRules {
    fn default() -> Self {
        Self {
            stop: 0.9,
            spin_damp: 0.25,
            lift: 17.0,
            float_seconds: 6.0,
        }
    }
}

/// The splash a lost brick makes in the water, from the original
/// (docs/research/controls-physics.md): a front layer of splash pictures that
/// grow in and vanish, and white drops thrown up behind them. Lengths in cells.
/// Skies and weather the original swaps in by mode and difficulty, from the PC readouts
/// (docs/research/modes.md "Race Pro look", docs/research/gap-scan.md 1-2). Colours are
/// 0-255 RGB, gradients top to bottom; lengths in cells, times in seconds; `(?)` marks
/// guesses.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct WeatherLook {
    /// Race Pro ("Special") night sky and water band.
    pub race_pro_sky: Vec<[u8; 3]>,
    /// Race Pro's water band when `water_band_race_pro` isn't staged: gradient top to
    /// bottom, its height in cells and parallax (modes.md, Race Pro look).
    pub race_pro_water: Vec<[u8; 3]>,
    pub race_pro_water_height: f32,
    pub race_pro_water_parallax: f32,
    /// Rain: drops a second, life, speed (cells/s), streak length (cells), RGBA colour;
    /// front then back. Emitters tilt this many degrees with the wind.
    pub rain_front: (f32, f32, f32, f32, [f32; 4]),
    pub rain_back: (f32, f32, f32, f32, [f32; 4]),
    pub rain_tilt: f32,
    /// Lightning: seconds between strikes (random in range), no strikes in the first
    /// seconds, flash opacity and fade, bolt hold and fade, branches.
    pub lightning_every: (f32, f32),
    pub lightning_quiet_start: f32,
    pub lightning_flash: (f32, f32),
    pub lightning_bolt: (f32, f32),
    pub lightning_branches: u32,
    /// Puzzle sandstorm: back and front tint (RGB) and opacity, scroll speed as a
    /// share of the view width per second, and the per-layer offset.
    pub sandstorm_back: ([u8; 3], f32),
    pub sandstorm_front: ([u8; 3], f32),
    pub sandstorm_speed: f32,
    pub sandstorm_offset: f32,
    /// Puzzle sand strips along the screen bottom, as shares of the view height: the back
    /// strip's height, how far its bottom sits above the screen bottom, and its opacity
    /// (drawn behind the front ruins); then the front strip's height (in front of them,
    /// on the screen bottom).
    pub sand_strip: f32,
    pub sand_lift: f32,
    pub sand_alpha: f32,
    pub sand_front: f32,
    /// Puzzle Pro sky, the eclipsed sun's height above the view centre as a share of the
    /// view height (9 units in the original's background camera, ?), its size, and its
    /// corona: rays, width and height ranges (multiples of the ray picture).
    pub puzzle_pro_sky: Vec<[u8; 3]>,
    pub eclipse_height: f32,
    pub eclipse_size: f32,
    pub corona_rays: u32,
    pub corona_width: (f32, f32),
    pub corona_height: (f32, f32),
    /// Survival night sky and the curse-wave sky it shifts to over `wave_shift` seconds.
    pub survival_sky: Vec<[u8; 3]>,
    pub survival_wave_sky: Vec<[u8; 3]>,
    pub wave_shift: f32,
    /// During a curse wave the background's clouds and islands grey to this colour,
    /// and drifting clouds go this many times faster (gap-scan.md 2).
    pub survival_wave_grey: [u8; 3],
    pub survival_wave_drift: f32,
}

impl Default for WeatherLook {
    fn default() -> Self {
        Self {
            race_pro_sky: vec![[49, 21, 138], [13, 119, 154], [37, 25, 126]],
            race_pro_water: vec![[13, 119, 154], [49, 21, 138]],
            race_pro_water_height: 6.0,
            race_pro_water_parallax: 0.3,
            rain_front: (100.0, 1.0, 100.0, 3.0, [0.82, 0.78, 0.96, 0.9]),
            rain_back: (1024.0, 3.0, 40.0, 1.0, [0.43, 0.45, 0.79, 0.9]),
            rain_tilt: 30.0,
            lightning_every: (5.0, 14.0),
            lightning_quiet_start: 3.0,
            lightning_flash: (0.65, 0.1),
            lightning_bolt: (0.4, 0.2),
            lightning_branches: 4,
            sandstorm_back: ([0x9B, 0x86, 0x44], 0.38),
            sandstorm_front: ([0xE9, 0xAA, 0x5D], 0.37),
            sandstorm_speed: 0.035,
            sandstorm_offset: 0.025,
            sand_strip: 0.085,
            sand_lift: 0.027,
            sand_alpha: 0.803,
            sand_front: 0.089,
            puzzle_pro_sky: vec![[0xAB, 0x05, 0x4C], [0xB8, 0x44, 0x42]],
            eclipse_height: 0.3,
            eclipse_size: 0.22,
            corona_rays: 7,
            corona_width: (0.6, 0.8),
            corona_height: (0.3, 2.0),
            survival_sky: vec![[51, 30, 103], [91, 83, 173]],
            survival_wave_sky: vec![[26, 13, 58], [147, 64, 157]],
            wave_shift: 1.0,
            survival_wave_grey: [166, 166, 166],
            survival_wave_drift: 138.0,
        }
    }
}

/// Local cups: a series of matches with points per place, a trophy for the cup
/// winner, and XP (the original gives XP only in cups and online). Built from
/// public descriptions and the trophy names read so far; the series length,
/// points and tie rule are guesses `(?)` until read on the PC.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct CupRules {
    /// Points for 1st, 2nd, 3rd and 4th place in each match (4 − place).
    pub points: Vec<u32>,
    /// The local cups to pick from on Select Mode, smallest first.
    pub sizes: Vec<CupSize>,
}

/// A cup is first to `target` points; a tie on the most points at or above it
/// goes to overtime until one player leads alone (docs/research/match-moments.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct CupSize {
    pub name: String,
    pub target: u32,
    /// The trophy picture shown to the cup winner.
    pub trophy: String,
}

impl Default for CupRules {
    fn default() -> Self {
        let size = |name: &str, target, trophy: &str| CupSize {
            name: name.into(),
            target,
            trophy: trophy.into(),
        };
        Self {
            points: vec![3, 2, 1, 0],
            sizes: vec![
                size("Tiny Cup", 6, "trophy"),
                size("Big Cup", 12, "trophy_big"),
                size("Super Cup", 18, "trophy_super"),
            ],
        }
    }
}

/// Experience, from the original (docs/research/match-moments.md). Only online
/// matches and cups give any; a single local match gives none.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct XpRules {
    /// The winner's XP on Easy, Normal and Pro.
    pub winner: (u32, u32, u32),
    /// Last place's XP.
    pub last: u32,
    /// Second and third to last get these shares of the winner's XP.
    pub second_to_last: f32,
    pub third_to_last: f32,
    /// XP lost for quitting a match.
    pub quit: u32,
    /// XP needed to reach each level, level 1 first.
    pub levels: Vec<u32>,
    /// XP for each level after the last listed one.
    pub level_step: u32,
    /// Level names, level 1 first; past them, `master` with the count of levels
    /// beyond the named ones (?).
    pub names: Vec<String>,
    pub master: String,
}

impl Default for XpRules {
    fn default() -> Self {
        Self {
            winner: (600, 800, 1000),
            last: 100,
            second_to_last: 0.25,
            third_to_last: 0.5,
            quit: 250,
            levels: vec![
                0, 500, 2_500, 10_000, 25_000, 45_000, 75_000, 112_500, 162_500, 225_000, 300_000,
            ],
            level_step: 75_000,
            names: [
                "Rookie Magician",
                "Apprentice Magician",
                "Magician",
                "Rookie Sorcerer",
                "Apprentice Sorcerer",
                "Sorcerer",
                "Rookie Wizard",
                "Apprentice Wizard",
                "Wizard",
                "Expert Wizard",
            ]
            .map(String::from)
            .to_vec(),
            master: "Master Wizard Level".into(),
        }
    }
}

impl XpRules {
    /// XP for finishing in `place` (0 = winner) of `players` on `difficulty`.
    pub fn for_place(&self, place: usize, players: usize, difficulty: Difficulty) -> u32 {
        let winner = match difficulty {
            Difficulty::Easy => self.winner.0,
            Difficulty::Normal => self.winner.1,
            Difficulty::Pro => self.winner.2,
        };
        if place == 0 {
            return winner;
        }
        match players.saturating_sub(place + 1) {
            0 => self.last,
            1 => (winner as f32 * self.second_to_last).round() as u32,
            2 => (winner as f32 * self.third_to_last).round() as u32,
            _ => 0,
        }
    }

    /// The level (1 up) reached with `xp`.
    pub fn level(&self, xp: u32) -> u32 {
        let listed = self
            .levels
            .iter()
            .filter(|&&need| xp >= need)
            .count()
            .max(1) as u32;
        let top = self.levels.last().copied().unwrap_or(0);
        if listed as usize == self.levels.len() && self.level_step > 0 {
            listed + (xp - top) / self.level_step
        } else {
            listed
        }
    }

    /// The name shown for a level.
    pub fn name(&self, level: u32) -> String {
        match self.names.get(level.saturating_sub(1) as usize) {
            Some(name) => name.clone(),
            None => format!("{} {}", self.master, level as usize - self.names.len()),
        }
    }
}

/// How casting looks, from the PC's readout of the original (tools/extract notes,
/// "Cast beams" and "Spell clouds"). Lengths in cells, times in seconds; `(?)` marks
/// guesses.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct CastLook {
    /// Dark cast, beam up from the caster: grows in, then gives up.
    pub beam_up: (f32, f32),
    /// Then a beam down onto each victim's brick: grows in, then gives up.
    pub beam_down: (f32, f32),
    /// Then the overlay over each victim's lane: shown (opening in 1/6 s), closing.
    pub lane_wave: (f32, f32),
    /// A finished beam squeezes to nothing this fast.
    pub beam_squeeze: f32,
    /// Beam width (the white edges sit at ±0.88) and how far above the view top
    /// the beams reach (?).
    pub beam_width: f32,
    pub beam_reach: f32,
    /// Light cast: the lightning from the wizard to the brick lasts this long, and
    /// its wobble peaks at this many cells (a sixth of the way in; clip `Lightning`).
    pub lightning: f32,
    pub lightning_wobble: f32,
    /// Spell clouds: Show, one Floating cycle, Used and Hide clip lengths.
    pub cloud_show: f32,
    pub cloud_float: f32,
    pub cloud_used: f32,
    pub cloud_hide: f32,
    /// Cloud size in cells (the 139 px backing at 52 px a unit, drawn ×1.31), the icon's
    /// share of it (126 px icons at 52 px a unit, unscaled), and how far it bobs while
    /// floating (?).
    pub cloud_size: f32,
    pub cloud_icon: f32,
    pub cloud_bob: f32,
    /// Where each wizard's light and dark clouds sit, from the wizard.
    /// Wizard 4 wasn't read and uses wizard 1's.
    pub cloud_spots: Vec<((f32, f32), (f32, f32))>,
    /// Large: the brick grows over this long, with this camera shake (cells
    /// sideways, cells up, seconds; ?).
    pub large_grow: f32,
    pub large_shake: (f32, f32, f32),
}

impl Default for CastLook {
    fn default() -> Self {
        Self {
            beam_up: (0.33, 1.0),
            beam_down: (0.63, 1.0),
            lane_wave: (1.0, 0.17),
            beam_squeeze: 0.03,
            beam_width: 1.76,
            beam_reach: 30.0,
            lightning: 0.4,
            lightning_wobble: 2.0,
            cloud_show: 0.83,
            cloud_float: 3.0,
            cloud_used: 1.22,
            cloud_hide: 0.38,
            cloud_size: 3.5,
            cloud_icon: 0.69,
            cloud_bob: 0.12,
            cloud_spots: vec![
                ((0.58, 4.8), (2.87, 3.53)),
                ((0.38, 5.37), (2.67, 4.1)),
                ((0.75, 5.0), (3.04, 3.73)),
            ],
            large_grow: 0.3,
            large_shake: (0.1, 0.13, 0.3),
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SplashLook {
    /// The splash sits at the brick's centre, moved down by up to this much.
    pub drop_below: f32,
    /// Front pictures: how many, along a line this wide, each this size across.
    pub fronts: u32,
    pub front_line: f32,
    pub front_size: (f32, f32),
    /// Front picture size over its life: (seconds, share of full size) points.
    pub front_curve: Vec<(f32, f32)>,
    /// Drops: how many, their size, the disc they start in (radius), the cone they
    /// fly up in (degrees either side), their speed and gravity.
    pub drops: u32,
    pub drop_size: (f32, f32),
    pub drop_disc: f32,
    pub drop_cone: f32,
    pub drop_speed: (f32, f32),
    pub drop_gravity: f32,
    /// Drops keep full size until the first time and are gone by the second.
    pub drop_shrink: (f32, f32),
}

impl Default for SplashLook {
    fn default() -> Self {
        Self {
            drop_below: 0.4,
            fronts: 5,
            front_line: 2.86,
            front_size: (1.3, 2.5),
            front_curve: vec![(0.0, 0.0), (0.1, 1.0), (0.42, 0.92), (0.68, 0.0)],
            drops: 12,
            drop_size: (0.3, 0.4),
            drop_disc: 1.5,
            drop_cone: 2.3,
            drop_speed: (10.0, 18.0),
            drop_gravity: 36.0,
            drop_shrink: (0.45, 0.93),
        }
    }
}

/// The start screen's intro, from the original's MoveIn clip (docs/research/
/// menus.md). Offsets are in the logo's units (26 px at 1080 high), y up; keys
/// are (seconds, value).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct StartLook {
    /// The logo's resting centre above the screen centre, in pixels.
    pub logo_y: f32,
    /// The logo dropping in and bouncing.
    pub drop: Vec<(f32, f32)>,
    /// The dip when the brick lands on it.
    pub dip: Vec<(f32, f32)>,
    /// The logo's tilt in degrees, clockwise positive, as the brick lands.
    pub tilt: Vec<(f32, f32)>,
    /// The rubble hopping up as the brick lands.
    pub debris_hop: Vec<(f32, f32)>,
    /// The brick falls from this many units above its spot, from this second,
    /// and lands at this second (the fall's start and height are guesses).
    pub brick_fall: (f32, f32, f32),
    /// Seconds before the prompt shows, and its blink: seconds on, seconds off.
    pub prompt_after: f32,
    pub blink: (f32, f32),
}

impl Default for StartLook {
    fn default() -> Self {
        Self {
            logo_y: 182.0,
            drop: vec![
                (0.0, 22.39),
                (0.38, -2.41),
                (0.466, -1.26),
                (0.552, 0.64),
                (0.638, 1.47),
                (0.724, 0.15),
                (0.81, 0.0),
            ],
            dip: vec![
                (1.63, 0.0),
                (1.92, -2.24),
                (2.1, 0.33),
                (2.3, -0.21),
                (2.48, 0.0),
            ],
            tilt: vec![
                (1.63, 0.0),
                (1.69, 3.05),
                (1.87, -1.87),
                (1.97, 0.82),
                (2.05, 0.32),
                (2.11, -0.18),
                (2.17, 0.0),
            ],
            debris_hop: vec![(1.63, 0.0), (1.8, 2.3), (2.04, 0.0)],
            brick_fall: (20.0, 1.2, 1.63),
            prompt_after: 2.62,
            blink: (0.5, 0.5),
        }
    }
}

/// How the picked title tile moves (docs/research/menus.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct MenuLook {
    /// For each ray layer from the first: degrees it turns, and over how many seconds,
    /// before it starts again. Layers past the list stay still.
    pub ray_turns: Vec<(f32, f32)>,
    /// The corners are this many pixels (of a 1080-high screen) out at the start of
    /// each loop of this many seconds and at rest halfway through.
    pub corner_pulse: (f32, f32),
}

impl Default for MenuLook {
    fn default() -> Self {
        Self {
            ray_turns: vec![(180.0, 2.0), (-90.0, 2.0)],
            corner_pulse: (5.0, 1.0),
        }
    }
}

/// The lanes across the screen, in cells: the screen is `screen_width` wide with
/// `margin` at each side and `gap` between lanes, so each of n lanes is
/// `(screen_width - 2 margin - (n - 1) gap) / n` wide (33 for 2 players, 21.67 for 3,
/// 16 for 4). From the original (docs/research/match-moments.md).
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct Lanes {
    pub screen_width: f32,
    pub margin: f32,
    pub gap: f32,
}

impl Default for Lanes {
    fn default() -> Self {
        Self {
            screen_width: 73.85,
            margin: 3.42,
            gap: 1.0,
        }
    }
}

impl Lanes {
    /// Width of each lane with `count` towers.
    pub fn width(&self, count: usize) -> f32 {
        let n = count.max(1) as f32;
        (self.screen_width - 2.0 * self.margin - (n - 1.0) * self.gap) / n
    }

    /// Distance between neighbouring tower centres with `count` towers.
    pub fn spacing(&self, count: usize) -> f32 {
        self.width(count) + self.gap
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
            lanes: Lanes::default(),
            menu: MenuLook::default(),
            start: StartLook::default(),
            splash: SplashLook::default(),
            casts: CastLook::default(),
            xp: XpRules::default(),
            cups: CupRules::default(),
            weather: WeatherLook::default(),
            water: WaterRules::default(),
            bases: Bases::default(),
            roofs: Roofs::default(),
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
            endless_play: EndlessPlay::default(),
            trials: crate::trial_list::original_trials(),
            trial_floors: crate::trial_list::original_trial_floors(),
            trial_tiers: TrialTiers::default(),
            trial: None,
            skins: Vec::new(),
            magic: MagicRules::default(),
            ai: AiRules::default(),
            wizards: WizardLook::default(),
            moments: MatchMoments::default(),
            hud: HudLook::default(),
            pieces: default_pieces(),
        }
    }
}

impl GameConfig {
    /// The top of the view while a tower's top is at `top` cells: the camera keeps
    /// the spawn point and a little room above it in sight.
    /// How far under the base top the camera may go: the floor's height (the
    /// original's FloorFactory height, set as the tower camera's minBottom).
    pub fn camera_floor(&self) -> f32 {
        let trial_floor = self
            .trial
            .and_then(|i| self.trials.get(i))
            .and_then(|t| t.floor.as_deref())
            .and_then(crate::trial_list::floor_height);
        match (trial_floor, self.mode) {
            (Some(height), _) => height,
            (None, ModeKind::Puzzle) if self.difficulty == Some(Difficulty::Pro) => 13.0,
            (None, ModeKind::Puzzle) => 12.0,
            (None, ModeKind::EndlessPuzzle) => 10.0,
            _ => 12.5,
        }
    }

    pub fn camera_top(&self, top: f32) -> f32 {
        (top + self.spawn_gap + 3.0).max(16.0)
    }

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
            // The Endless modes are played alone: nobody to curse.
            _ => false,
        }
    }

    /// Whether everyone gets the same bricks in `mode`.
    pub fn same_bricks(&self, mode: ModeKind) -> bool {
        match mode {
            ModeKind::Race => self.race.same_bricks,
            ModeKind::Survival => self.survival.same_bricks,
            _ => true,
        }
    }

    /// How far either side of its lane centre a steered brick may go in the current
    /// mode, in cells, for a tower whose widest settled brick reaches `tower_half_width`
    /// from the centre. Puzzle lanes are wider and grow with the tower.
    pub fn lane_reach(&self, tower_half_width: f32) -> f32 {
        match self.mode.rules() {
            ModeKind::Puzzle => (self.lane_half_width * self.puzzle.lane_scale)
                .max(tower_half_width + self.puzzle.lane_margin),
            _ => self.lane_half_width,
        }
    }

    /// Base drop speed of the current mode, in cells/s. Endless Survival and Endless
    /// Puzzle use their battle mode's, set by `EndlessPlay::apply`.
    pub fn drop_speed(&self) -> f32 {
        match self.mode.rules() {
            ModeKind::Survival => self.survival.drop_speed,
            ModeKind::Puzzle => self.puzzle.drop_speed,
            ModeKind::Endless => self.endless.drop_speed,
            _ => self.race.drop_speed,
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
    pub(crate) fn validated(mut self) -> Self {
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
    fn trial_tiers_open_with_medals() {
        let tiers = TrialTiers::default();
        assert_eq!(
            (tiers.tier_of(0), tiers.tier_of(9), tiers.tier_of(10)),
            (0, 0, 1)
        );
        assert_eq!(tiers.tier_of(49), 4);
        assert!(tiers.is_open(9, 0), "Rookie is open from the start");
        assert!(!tiers.is_open(10, 7) && tiers.is_open(10, 8));
        assert!(!tiers.is_open(40, 35) && tiers.is_open(40, 36));
    }

    #[test]
    fn race_pro_spell_rules_differ() {
        let magic = MagicRules::default();
        let race = |d: Difficulty| -> Vec<&AwardRule> {
            magic
                .award_rules
                .iter()
                .filter(|r| {
                    r.mode == Some(ModeKind::Race)
                        && (r.difficulties.is_empty() || r.difficulties.contains(&d))
                })
                .collect()
        };
        let far = |d| {
            race(d)
                .iter()
                .filter_map(|r| r.behind_leader)
                .fold(0.0, f32::max)
        };
        assert_eq!(far(Difficulty::Normal), 15.0);
        assert_eq!(far(Difficulty::Pro), 20.0);
        let minibase = |d| {
            race(d).iter().any(|r| {
                r.behind_leader.is_none()
                    && r.finish_within.is_none()
                    && r.light.contains(&(SpellKind::Minibase, 3))
            })
        };
        assert!(minibase(Difficulty::Pro) && !minibase(Difficulty::Normal));
        // The line wait grows with players on Normal and Pro, not on Easy.
        assert_eq!(magic.wait(4, Difficulty::Easy), magic.line_wait);
        assert_eq!(magic.wait(4, Difficulty::Pro), magic.line_wait * 1.5);
    }

    #[test]
    fn xp_per_place_and_levels() {
        let xp = XpRules::default();
        // Four players on Normal: 800, half, quarter, 100.
        let four: Vec<u32> = (0..4)
            .map(|p| xp.for_place(p, 4, Difficulty::Normal))
            .collect();
        assert_eq!(four, vec![800, 400, 200, 100]);
        // Two players: the winner keeps the full amount, last gets 100.
        assert_eq!(xp.for_place(0, 2, Difficulty::Pro), 1000);
        assert_eq!(xp.for_place(1, 2, Difficulty::Pro), 100);
        assert_eq!(xp.for_place(1, 3, Difficulty::Easy), 150);
        assert_eq!(xp.level(0), 1);
        assert_eq!(xp.level(499), 1);
        assert_eq!(xp.level(500), 2);
        assert_eq!(xp.level(299_999), 10);
        assert_eq!(xp.level(300_000), 11);
        assert_eq!(xp.level(375_000), 12);
        assert_eq!(xp.name(1), "Rookie Magician");
        assert_eq!(xp.name(10), "Expert Wizard");
        assert_eq!(xp.name(11), "Master Wizard Level 1");
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

/// What each lane shows during a match, from gameplay videos of the original
/// (docs/research/video-notes.md). Screen sizes are shares of the view's height
/// (or width, where said), so they hold at any window size; world offsets are cells.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct HudLook {
    /// The next-brick card hanging from the top of each lane: height as a share of
    /// the view height, width over height, its top bar's share of its height, and
    /// how far left of the lane's drop column its centre sits (share of view width).
    pub card_height: f32,
    pub card_aspect: f32,
    pub card_bar: f32,
    pub card_left: f32,
    /// Card and top bar colours (0-255).
    pub card_color: [u8; 3],
    pub card_bar_color: [u8; 3],
    /// The Survival brick count's text size (share of view height).
    pub count_size: f32,
    /// The place label (1ST, 2ND...) under the wizard: text size, cells below the
    /// wizard, and the leader's and the others' colours (0-255).
    pub place_size: f32,
    pub place_below: f32,
    pub place_first: [u8; 3],
    pub place_other: [u8; 3],
    /// Hearts under the wizard: size (the original's 1.12 cells over the 30-cell view) and
    /// cells from the wizard down to the row's centre (1.53 + 0.3).
    pub heart_size: f32,
    pub heart_below: f32,
    /// The drop guide's colour in Race (and Puzzle, Endless) and in Survival (0-1, with alpha).
    pub guide_race: [f32; 4],
    pub guide_survival: [f32; 4],
    /// Spell line diamonds: size and gap in cells, and one colour per line in turn
    /// when the art gives none (0-255).
    pub diamond_size: f32,
    pub diamond_step: f32,
    pub line_colors: Vec<[u8; 3]>,
    /// Losing towers turn this colour (0-1) by this much once the match is over.
    pub loser_tint: [f32; 3],
    pub loser_mix: f32,
    /// Losing wizards shrink away over this many seconds once the match is over.
    pub wizard_vanish: f32,
    /// The tower camera (SingleTowerCamera): at least this many cells tall, its top
    /// this many cells of room above the highest tower minus `camera_tower_room`
    /// (the original's maxTowerInScreen), its bottom this far under the lowest tower
    /// but never under the floor.
    pub camera_height: f32,
    pub camera_tower_room: f32,
    pub camera_below_lowest: f32,
}

impl Default for HudLook {
    fn default() -> Self {
        Self {
            card_height: 65.0 / 720.0,
            card_aspect: 75.0 / 65.0,
            card_bar: 0.2,
            card_left: 78.0 / 1280.0,
            card_color: [255, 255, 255],
            card_bar_color: [36, 52, 110],
            count_size: 34.0 / 720.0,
            place_size: 22.0 / 720.0,
            place_below: 2.2,
            place_first: [255, 190, 60],
            place_other: [205, 190, 240],
            heart_size: 1.12 / 30.0,
            heart_below: 1.83,
            guide_race: [0.65, 0.95, 1.0, 0.3],
            guide_survival: [0.8, 0.7, 1.0, 0.3],
            diamond_size: 0.45,
            diamond_step: 0.6,
            line_colors: vec![
                [80, 220, 255],
                [110, 230, 90],
                [255, 110, 200],
                [200, 235, 80],
            ],
            loser_tint: [0.78, 0.82, 0.88],
            loser_mix: 0.7,
            wizard_vanish: 0.6,
            camera_height: 41.538_46,
            camera_tower_room: 20.0,
            camera_below_lowest: 10.0,
        }
    }
}
