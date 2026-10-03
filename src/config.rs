//! Data-driven game configuration.
//!
//! Everything a modder is likely to want to tweak (physics feel, brick shapes,
//! mode rules) lives here and is loaded from `assets/config/game.ron` at startup.
//! Missing fields fall back to the defaults below, so a mod file only needs the
//! values it changes.

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
    pub mode: ModeConfig,
    pub pieces: Vec<PieceDef>,
}

/// Rules for the active game mode.
#[derive(Deserialize, Clone, Debug, PartialEq)]
pub enum ModeConfig {
    /// Build past a finish line and keep the tower there for `hold_seconds`.
    Race { finish_height: f32, hold_seconds: f32 },
    /// Place `bricks_to_place` bricks while losing fewer than `lives`.
    Survival { lives: u32, bricks_to_place: u32 },
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
            spawn_gap: 8.0,
            kill_depth: 12.0,
            collider_scale: 0.98,
            brick_friction: 0.8,
            brick_restitution: 0.0,
            brick_density: 1.0,
            seed: 0x7457_4552,
            mode: ModeConfig::Race {
                finish_height: 20.0,
                hold_seconds: 3.0,
            },
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
    fn shipped_config_parses() {
        let text = include_str!("../assets/config/game.ron");
        let config = GameConfig::from_ron(text).expect("assets/config/game.ron should parse");
        assert_eq!(config.pieces.len(), 7);
    }

    #[test]
    fn partial_config_keeps_defaults() {
        let config = GameConfig::from_ron("(fall_speed: 9.0)").unwrap();
        assert_eq!(config.fall_speed, 9.0);
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
