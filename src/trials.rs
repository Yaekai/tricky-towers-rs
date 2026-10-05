//! Trials: single-player challenges from [`GameConfig::trials`]. The trial's rules
//! are put into the config when its match starts (`TrialDef::apply`, called from
//! `start_match`); this module runs the parts a normal match doesn't have: the race
//! clock and the enemy wizard's script, and remembers which trials are done.
//!
//! Headless-safe: no rendering or keyboard here.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{CastWhen, Curse, GameConfig};
use crate::gameplay::{GameplaySystems, Match, Phase, TowerState, TowerStatus};
use crate::spells::EnemyCasts;

pub struct TrialsPlugin;

impl Plugin for TrialsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<TrialRun>()
            .init_resource::<TrialProgress>()
            .add_systems(
                FixedUpdate,
                (run_trial, record_trial).chain().after(GameplaySystems),
            );
    }
}

/// The running trial's script state, reset each match.
#[derive(Resource, Debug, Default)]
pub struct TrialRun {
    round: u64,
    /// For each line of the enemy script: whether it has fired (once-only lines),
    /// or when it fires next (`Every` lines).
    fired: Vec<bool>,
    next: Vec<f32>,
}

/// Which trials have been beaten, by name. Kept between runs in `file` when one is
/// set (the game sets it; tests and offscreen pictures don't).
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct TrialProgress {
    pub done: Vec<String>,
    #[serde(skip)]
    pub file: Option<PathBuf>,
}

impl TrialProgress {
    pub fn is_done(&self, name: &str) -> bool {
        self.done.iter().any(|d| d == name)
    }

    /// Medals earned so far (per trial beaten, ?).
    pub fn medals(&self, tiers: &crate::config::TrialTiers) -> u32 {
        self.done.len() as u32 * tiers.medals_per_trial
    }

    /// The progress saved in `file`, or none yet, saving there.
    pub fn load(file: PathBuf) -> Self {
        let mut progress: Self = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        progress.file = Some(file);
        progress
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
            warn!("could not save trial progress to {}: {err}", file.display());
        }
    }
}

/// Runs the race clock and the enemy wizard's script of the trial being played.
fn run_trial(
    config: Res<GameConfig>,
    game: Res<Match>,
    mut run: ResMut<TrialRun>,
    mut enemy: ResMut<EnemyCasts>,
    mut towers: Query<&mut TowerState>,
) {
    let Some(trial) = config.trial.and_then(|i| config.trials.get(i)) else {
        return;
    };
    if run.round != game.round {
        run.round = game.round;
        run.fired = vec![false; trial.enemy.len()];
        run.next = trial
            .enemy
            .iter()
            .map(|cast| match cast.when {
                CastWhen::Every(period) => period,
                _ => 0.0,
            })
            .collect();
    }
    if game.phase != Phase::Playing {
        return;
    }
    let used = towers.iter().map(|s| s.bricks_used()).max().unwrap_or(0);
    let in_hand = towers.iter().any(|s| s.active.is_some());
    let height = towers.iter().map(|s| s.height).fold(0.0, f32::max);
    let target = trial.brick_target(&config);
    for (i, cast) in trial.enemy.iter().enumerate() {
        let due = match cast.when {
            // The original's two kinds: when the tower reaches a height, or on the
            // Nth brick (the brick in hand is cursed).
            CastWhen::Height(cells) => !run.fired[i] && height >= cells,
            CastWhen::Brick(n) => !run.fired[i] && in_hand && used + 1 >= n.max(1),
            CastWhen::Every(period) => {
                let due = game.elapsed >= run.next[i];
                if due {
                    run.next[i] += period.max(0.1);
                }
                due
            }
            CastWhen::At(seconds) => !run.fired[i] && game.elapsed >= seconds,
            CastWhen::BricksUsed(share) => {
                !run.fired[i] && target.is_some_and(|n| used as f32 >= (share * n as f32).ceil())
            }
        };
        if due {
            run.fired[i] = true;
            enemy.0.push(Curse::new(&cast.spells, 1, None));
        }
    }
    // A race trial is lost when its clock runs out.
    if let Some(seconds) = trial.seconds
        && game.elapsed >= seconds
    {
        for mut state in &mut towers {
            if state.status == TowerStatus::Building {
                state.status = TowerStatus::Out;
                state.out_at = Some(game.elapsed);
            }
        }
    }
}

/// Marks the trial done when it is won.
fn record_trial(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<&TowerState>,
    mut progress: ResMut<TrialProgress>,
) {
    let Some(trial) = config.trial.and_then(|i| config.trials.get(i)) else {
        return;
    };
    let won = game.phase == Phase::Over && towers.iter().any(|s| s.status == TowerStatus::Won);
    if won && !progress.is_done(&trial.name) {
        progress.done.push(trial.name.clone());
        progress.save();
    }
}
