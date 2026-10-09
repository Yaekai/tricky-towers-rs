//! Local cups (Max's vanilla recording plays a local Tiny Cup), from the PC readout (docs/research/match-moments.md): matches
//! picked on Select Mode, each giving points by place ([`CupRules::points`]),
//! until one player alone has the most points at or above the cup's target
//! (ties go to overtime). The winner gets the cup's trophy. Local cups give no XP.
//!
//! Headless-safe: no rendering or keyboard here.

use std::path::PathBuf;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{CupRules, CupSize, Difficulty, GameConfig, ModeKind};
use crate::gameplay::{GameplaySystems, Match, Phase, Tower, TowerState, placings};

pub struct CupsPlugin;

impl Plugin for CupsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Cup>()
            .init_resource::<XpBook>()
            .add_systems(FixedUpdate, score_cup.after(GameplaySystems));
    }
}

/// The cup being played, if any, and the one picked for the next start.
#[derive(Resource, Debug, Default, Clone)]
pub struct Cup {
    /// The cup picked on Select Mode, as an index into `CupRules::sizes`; `None`
    /// plays single matches.
    pub pick: Option<usize>,
    /// The cup being played, once started.
    pub size: Option<usize>,
    pub played: u32,
    /// Points per tower index.
    pub points: Vec<u32>,
    /// Per tower index, the place (0 = first) that earned each point, in order:
    /// the standings panel draws one medal per point.
    pub medals: Vec<Vec<usize>>,
    /// The final standings (tower indexes, first first) once the cup is over.
    pub standings: Option<Vec<usize>>,
    /// The mode and difficulty pairs the cup plays (more than one when Select Mode
    /// picked Random), the one being played, and those left to pick from next.
    pub options: Vec<(ModeKind, Difficulty)>,
    pub current: Option<(ModeKind, Difficulty)>,
    pool: Vec<(ModeKind, Difficulty)>,
    /// Whether the next match opens with its mode title: the first match, and a
    /// match whose mode or difficulty changed.
    pub show_title: bool,
    counted: u64,
}

impl Cup {
    /// Starts the picked cup (or none) from scratch, at the next match.
    pub fn begin(&mut self) {
        let (options, current) = (std::mem::take(&mut self.options), self.current);
        *self = Self {
            pick: self.pick,
            size: self.pick,
            show_title: true,
            ..default()
        };
        self.play(options, current);
    }

    /// Sets the pairs a cup plays and the first one (CupMatchFlowController).
    pub fn play(&mut self, options: Vec<(ModeKind, Difficulty)>, first: Option<(ModeKind, Difficulty)>) {
        self.pool = options.iter().copied().filter(|&o| Some(o) != first).collect();
        self.options = options;
        self.current = first;
    }

    /// The next match's mode and difficulty, from `roll`. With more than one pair,
    /// one not just played; with Random mode and Random difficulty both, another
    /// mode altogether. Sets `show_title` when it changes.
    pub fn next_match(&mut self, roll: usize) -> Option<(ModeKind, Difficulty)> {
        let current = self.current?;
        if self.options.len() < 2 {
            self.show_title = false;
            return Some(current);
        }
        let modes = |o: &[(ModeKind, Difficulty)]| {
            let mut m: Vec<ModeKind> = o.iter().map(|p| p.0).collect();
            m.dedup();
            m.len()
        };
        let any = modes(&self.options) > 1 && self.options.len() > modes(&self.options);
        let mut set_aside = Vec::new();
        let put_aside = |pool: &mut Vec<(ModeKind, Difficulty)>, aside: &mut Vec<_>| {
            if any {
                aside.extend(pool.iter().copied().filter(|p: &(ModeKind, Difficulty)| p.0 == current.0));
                pool.retain(|p| p.0 != current.0);
            } else {
                pool.retain(|&p| p != current);
            }
        };
        put_aside(&mut self.pool, &mut set_aside);
        if self.pool.is_empty() {
            self.pool = self.options.clone();
            put_aside(&mut self.pool, &mut set_aside);
        }
        let pick = self.pool[roll % self.pool.len()];
        self.show_title = pick != current;
        self.pool.extend(set_aside);
        self.pool.retain(|&p| p != current);
        self.current = Some(pick);
        Some(pick)
    }

    pub fn rules<'a>(&self, rules: &'a CupRules) -> Option<&'a CupSize> {
        self.size.and_then(|s| rules.sizes.get(s))
    }

    /// True while a cup has matches left to play.
    pub fn in_progress(&self) -> bool {
        self.size.is_some() && self.standings.is_none()
    }

    /// Adds one match's result. Returns true when it decided the cup.
    pub fn add_match(&mut self, rules: &CupRules, order: &[usize]) -> bool {
        let Some(size) = self.rules(rules).cloned() else {
            return false;
        };
        for (place, &tower) in order.iter().enumerate() {
            let earned = rules.points.get(place).copied().unwrap_or(0);
            if self.points.len() <= tower {
                self.points.resize(tower + 1, 0);
                self.medals.resize(tower + 1, Vec::new());
            }
            self.points[tower] += earned;
            let medals = &mut self.medals[tower];
            medals.resize(medals.len() + earned as usize, place);
        }
        self.played += 1;
        let best = self.points.iter().copied().max().unwrap_or(0);
        let leaders = self.points.iter().filter(|&&p| p == best).count();
        if best < size.target || leaders > 1 {
            return false;
        }
        // Most points first; the original sorts by points alone and keeps ties in
        // seat order, which this does too (CupMatchFlowController).
        let mut standings: Vec<usize> = (0..self.points.len()).collect();
        let last = |t: usize| order.iter().position(|&o| o == t).unwrap_or(usize::MAX);
        standings.sort_by(|&a, &b| {
            self.points[b]
                .cmp(&self.points[a])
                .then(last(a).cmp(&last(b)))
        });
        self.standings = Some(standings);
        true
    }

    /// True when the leaders are tied at or above the target, playing on.
    pub fn overtime(&self, rules: &CupRules) -> bool {
        let Some(size) = self.rules(rules) else {
            return false;
        };
        let best = self.points.iter().copied().max().unwrap_or(0);
        best >= size.target && self.points.iter().filter(|&&p| p == best).count() > 1
    }
}

/// XP earned so far by each player seat (player 1 first), kept between runs in
/// `file` when one is set (the game sets it; tests and offscreen pictures don't).
/// There are no profiles yet, so XP belongs to the seat (?).
#[derive(Resource, Debug, Default, Serialize, Deserialize)]
pub struct XpBook {
    pub players: Vec<u32>,
    #[serde(skip)]
    pub file: Option<PathBuf>,
}

impl XpBook {
    pub fn load(file: PathBuf) -> Self {
        let mut book: Self = std::fs::read_to_string(&file)
            .ok()
            .and_then(|text| ron::from_str(&text).ok())
            .unwrap_or_default();
        book.file = Some(file);
        book
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
            warn!("could not save XP to {}: {err}", file.display());
        }
    }

    pub fn add(&mut self, player: usize, xp: u32) {
        if self.players.len() <= player {
            self.players.resize(player + 1, 0);
        }
        self.players[player] += xp;
    }
}

/// Adds each finished match to the cup.
fn score_cup(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<(&Tower, &TowerState)>,
    mut cup: ResMut<Cup>,
) {
    let local = config.trial.is_none() && !config.mode.is_endless();
    if game.phase != Phase::Over || cup.counted == game.round || !local || !cup.in_progress() {
        return;
    }
    cup.counted = game.round;
    let all: Vec<(usize, &TowerState)> = towers.iter().map(|(t, s)| (t.index, s)).collect();
    let order = placings(&game, &all);
    cup.add_match(&config.cups, &order);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cup(target: u32) -> (CupRules, Cup) {
        let rules = CupRules {
            sizes: vec![CupSize {
                name: "test".into(),
                target,
                trophy: "trophy".into(),
            }],
            ..default()
        };
        let mut cup = Cup {
            pick: Some(0),
            ..default()
        };
        cup.begin();
        (rules, cup)
    }

    #[test]
    fn a_cup_is_first_to_its_target() {
        let (rules, mut cup) = cup(6);
        assert!(cup.in_progress());
        assert!(!cup.add_match(&rules, &[0, 1, 2]));
        assert!(!cup.add_match(&rules, &[2, 1, 0]));
        // 0 reaches 7, alone at the top.
        assert!(cup.add_match(&rules, &[0, 2, 1]));
        assert_eq!(cup.points, vec![7, 5, 6]);
        assert_eq!(cup.medals[0], vec![0, 0, 0, 2, 0, 0, 0]);
        assert_eq!(cup.standings, Some(vec![0, 2, 1]));
        assert!(!cup.in_progress());
        // Starting again keeps the pick and clears the score.
        cup.begin();
        assert_eq!((cup.size, cup.played, cup.points.len()), (Some(0), 0, 0));
    }

    #[test]
    fn a_tie_at_the_target_goes_to_overtime() {
        let (rules, mut cup) = cup(5);
        cup.add_match(&rules, &[0, 1]);
        cup.add_match(&rules, &[1, 0]);
        // 5 each: both at the target, tied.
        assert_eq!(cup.points, vec![5, 5]);
        assert!(cup.overtime(&rules));
        assert!(cup.in_progress());
        assert!(cup.add_match(&rules, &[1, 0]));
        assert_eq!(cup.standings, Some(vec![1, 0]));
    }

    #[test]
    fn a_random_cup_plays_a_new_mode_each_match() {
        use ModeKind::*;
        let (_, mut cup) = cup(6);
        let all: Vec<_> = [Race, Survival, Puzzle]
            .iter()
            .flat_map(|&m| Difficulty::ALL.map(|d| (m, d)))
            .collect();
        cup.play(all, Some((Race, Difficulty::Normal)));
        let mut last = Race;
        for roll in 0..12 {
            let (mode, _) = cup.next_match(roll * 7).unwrap();
            // Random mode and difficulty: always another mode, with its title.
            assert_ne!(mode, last);
            assert!(cup.show_title);
            last = mode;
        }
        // One pair: the same match again, no title.
        cup.play(vec![(Puzzle, Difficulty::Easy)], Some((Puzzle, Difficulty::Easy)));
        assert_eq!(cup.next_match(3), Some((Puzzle, Difficulty::Easy)));
        assert!(!cup.show_title);
    }

    #[test]
    fn the_local_cups_follow_the_original() {
        let rules = CupRules::default();
        let targets: Vec<u32> = rules.sizes.iter().map(|s| s.target).collect();
        assert_eq!(targets, [6, 12, 18]);
        assert_eq!(rules.points, [3, 2, 1, 0]);
    }
}
