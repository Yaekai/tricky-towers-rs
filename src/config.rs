//! Data-driven game configuration.
//!
//! Everything a modder is likely to want to tweak (physics feel, brick shapes,
//! mode rules, spells, opponents) lives here and is loaded from
//! `assets/config/game.ron` at startup. Missing fields fall back to the defaults
//! below, so a mod file only needs the values it changes.

use std::path::Path;

use bevy::prelude::*;
use serde::Deserialize;

/// Default location of the config file, relative to the working directory.
pub const DEFAULT_CONFIG_PATH: &str = "assets/config/game.ron";

#[derive(Resource, Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct GameConfig {
    /// World units per brick cell. All other lengths are given in cells.
    pub cell_size: f32,
    /// Downward gravity in cells per second squared.
    pub gravity: f32,
    /// How fast the controlled brick falls, in cells per second.
    pub fall_speed: f32,
    /// Multiplier applied to `fall_speed` while the player holds "down".
    pub fast_fall_multiplier: f32,
    /// Horizontal step per nudge, in cells. The original moves in half cells.
    pub nudge_step: f32,
    /// Seconds a direction must be held before the nudge starts repeating.
    pub nudge_repeat_delay: f32,
    /// Seconds between repeated nudges while a direction is held.
    pub nudge_repeat_interval: f32,
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
    pub brick_density: f32,
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

#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq, Default)]
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

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct RaceRules {
    /// Finish line height above the platform, in cells.
    pub finish_height: f32,
    /// Seconds the tower must stay past the line to win.
    pub hold_seconds: f32,
}

impl Default for RaceRules {
    fn default() -> Self {
        Self {
            finish_height: 20.0,
            hold_seconds: 3.0,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct SurvivalRules {
    pub lives: u32,
    pub bricks_to_place: u32,
}

impl Default for SurvivalRules {
    fn default() -> Self {
        Self {
            lives: 3,
            bricks_to_place: 30,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct PuzzleRules {
    /// Bricks to place.
    pub bricks: u32,
    /// Height of the laser line, in cells. Any settled brick above it fails the puzzle.
    pub line_height: f32,
    /// Seconds everything must rest below the line after the last brick lands.
    pub settle_seconds: f32,
}

impl Default for PuzzleRules {
    fn default() -> Self {
        Self {
            bricks: 10,
            line_height: 6.0,
            settle_seconds: 2.0,
        }
    }
}

/// Spells that can be granted. Light spells help the caster; dark spells hit
/// the opponent with the tallest tower.
#[derive(Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SpellKind {
    /// Light: vines bind every brick of your tower to the bricks it touches.
    Vines,
    /// Light: every resting brick of your tower turns to immovable stone.
    Stone,
    /// Light: your next brick is half size.
    Shrink,
    /// Dark: the target's next brick is oversized.
    Enlarge,
    /// Dark: the target's next brick is slippery as ice.
    Slippery,
}

impl SpellKind {
    pub fn is_dark(self) -> bool {
        matches!(self, SpellKind::Enlarge | SpellKind::Slippery)
    }

    pub fn name(self) -> &'static str {
        match self {
            SpellKind::Vines => "Vines",
            SpellKind::Stone => "Stone",
            SpellKind::Shrink => "Shrink",
            SpellKind::Enlarge => "Enlarge",
            SpellKind::Slippery => "Slippery",
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
#[serde(default)]
pub struct MagicRules {
    pub enabled: bool,
    /// A spell is granted every this many bricks placed (if the slot is empty).
    pub bricks_per_spell: u32,
    pub light: Vec<SpellKind>,
    /// Only granted when there is someone to cast them on.
    pub dark: Vec<SpellKind>,
    /// Size multiplier for Shrink and Enlarge.
    pub shrink_scale: f32,
    pub enlarge_scale: f32,
    /// Friction of a Slippery brick.
    pub slippery_friction: f32,
}

impl Default for MagicRules {
    fn default() -> Self {
        Self {
            enabled: true,
            bricks_per_spell: 5,
            light: vec![SpellKind::Vines, SpellKind::Stone, SpellKind::Shrink],
            dark: vec![SpellKind::Enlarge, SpellKind::Slippery],
            shrink_scale: 0.5,
            enlarge_scale: 1.5,
            slippery_friction: 0.02,
        }
    }
}

#[derive(Deserialize, Clone, Debug, PartialEq)]
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
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub struct PieceDef {
    pub name: String,
    /// sRGB colour, each channel 0.0 to 1.0.
    pub color: [f32; 3],
    /// Cell coordinates (x right, y up). Any polyomino works, not just tetrominoes.
    pub cells: Vec<(i32, i32)>,
}

impl PieceDef {
    pub fn new(name: &str, color: [f32; 3], cells: &[(i32, i32)]) -> Self {
        Self {
            name: name.to_string(),
            color,
            cells: cells.to_vec(),
        }
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

/// The seven classic tetrominoes.
pub fn default_pieces() -> Vec<PieceDef> {
    vec![
        PieceDef::new("I", [0.30, 0.80, 0.95], &[(0, 0), (1, 0), (2, 0), (3, 0)]),
        PieceDef::new("O", [0.95, 0.85, 0.30], &[(0, 0), (1, 0), (0, 1), (1, 1)]),
        PieceDef::new("T", [0.70, 0.40, 0.90], &[(0, 0), (1, 0), (2, 0), (1, 1)]),
        PieceDef::new("S", [0.45, 0.85, 0.40], &[(0, 0), (1, 0), (1, 1), (2, 1)]),
        PieceDef::new("Z", [0.95, 0.40, 0.40], &[(0, 1), (1, 1), (1, 0), (2, 0)]),
        PieceDef::new("J", [0.35, 0.50, 0.95], &[(0, 0), (1, 0), (2, 0), (0, 1)]),
        PieceDef::new("L", [0.95, 0.60, 0.25], &[(0, 0), (1, 0), (2, 0), (2, 1)]),
    ]
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            cell_size: 32.0,
            gravity: 30.0,
            fall_speed: 2.5,
            fast_fall_multiplier: 6.0,
            nudge_step: 0.5,
            nudge_repeat_delay: 0.18,
            nudge_repeat_interval: 0.05,
            platform_width: 9.0,
            tower_spacing: 20.0,
            spawn_gap: 8.0,
            kill_depth: 12.0,
            collider_scale: 0.98,
            brick_friction: 0.8,
            brick_restitution: 0.0,
            brick_density: 1.0,
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
        self.magic.bricks_per_spell = self.magic.bricks_per_spell.max(1);
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

    #[test]
    fn partial_config_keeps_defaults() {
        let config = GameConfig::from_ron("(fall_speed: 9.0, puzzle: (bricks: 4))").unwrap();
        assert_eq!(config.fall_speed, 9.0);
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
