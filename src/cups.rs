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
    /// Per tower index, how many medals it had before the latest match: the panel
    /// draws those at once and animates only the new ones (TournamentResultsItem).
    pub before: Vec<usize>,
    /// Per tower index, how many matches it finished fourth (no points, so no medals): the
    /// cup winner popup counts them.
    pub fourths: Vec<u32>,
    /// The tower whose fourth place in the latest match still shows one empty medal (1 in
    /// 7, TournamentResultsItem's joke). It takes a slot only in that match's fill.
    pub stray: Option<usize>,
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

    /// The most medals any player earned in the latest match.
    pub fn fresh(&self) -> usize {
        (0..self.medals.len())
            .map(|t| self.shown(t).saturating_sub(self.before.get(t).copied().unwrap_or(0)))
            .max()
            .unwrap_or(0)
    }

    /// The standings' rows, top first: the final standings, or most points first.
    pub fn panel_order(&self) -> Vec<usize> {
        self.standings.clone().unwrap_or_else(|| {
            let mut order: Vec<usize> = (0..self.points.len()).collect();
            order.sort_by(|&a, &b| self.points[b].cmp(&self.points[a]).then(a.cmp(&b)));
            order
        })
    }

    /// When each of the latest match's medals starts to pop in, per tower, and when the
    /// fill is over (CONTINUE): row by row down the standings, `step` apart within a row,
    /// `row_gap` after each row, from `start`.
    pub fn fill_times(&self, start: f32, step: f32, row_gap: f32) -> (Vec<Vec<f32>>, f32) {
        let mut times = vec![Vec::new(); self.medals.len()];
        let mut at = start;
        for t in self.panel_order() {
            let fresh = self.shown(t).saturating_sub(self.before.get(t).copied().unwrap_or(0));
            for _ in 0..fresh {
                times[t].push(at);
                at += step;
            }
            at += row_gap;
        }
        (times, at)
    }

    /// Medals tower `t` shows on the standings, the stray empty one included.
    pub fn shown(&self, t: usize) -> usize {
        self.medals.get(t).map_or(0, Vec::len) + usize::from(self.stray == Some(t))
    }

    /// The place (0 = first, 3 = the empty medal) of tower `t`'s `k`th medal shown.
    pub fn medal(&self, t: usize, k: usize) -> Option<usize> {
        let medals = self.medals.get(t)?;
        medals.get(k).copied().or((k == medals.len() && self.stray == Some(t)).then_some(3))
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
        self.stray = None;
        for (place, &tower) in order.iter().enumerate() {
            let earned = rules.points.get(place).copied().unwrap_or(0);
            if self.points.len() <= tower {
                self.points.resize(tower + 1, 0);
                self.medals.resize(tower + 1, Vec::new());
            }
            if self.before.len() <= tower {
                self.before.resize(tower + 1, 0);
            }
            if self.fourths.len() <= tower {
                self.fourths.resize(tower + 1, 0);
            }
            if place == 3 {
                self.fourths[tower] += 1;
            }
            self.points[tower] += earned;
            self.before[tower] = self.medals[tower].len();
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
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    if nanos % 7 == 0 {
        cup.stray = order.get(3).copied();
    }
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
    fn medals_fill_row_by_row_down_the_standings() {
        let (rules, mut cup) = cup(12);
        cup.add_match(&rules, &[1, 0]);
        let (times, end) = cup.fill_times(1.0, 0.25, 0.75);
        let gold = rules.points[0] as usize;
        let silver = rules.points[1] as usize;
        assert_eq!(times[1].len(), gold);
        assert_eq!(times[1][0], 1.0, "the leader's row starts after the wait");
        assert_eq!(times[0][0], 1.0 + 0.25 * gold as f32 + 0.75, "the next row after the leader's");
        assert_eq!(end, 1.0 + 0.25 * (gold + silver) as f32 + 1.5);
    }

    #[test]
    fn a_stray_empty_medal_shows_only_in_its_match() {
        let (rules, mut cup) = cup(6);
        cup.add_match(&rules, &[0, 1, 2, 3]);
        cup.stray = Some(3);
        assert_eq!((cup.shown(3), cup.medal(3, 0)), (1, Some(3)));
        assert_eq!(cup.fresh(), 3, "the winner's three still lead the fill");
        cup.add_match(&rules, &[0, 1, 2, 3]);
        assert_eq!((cup.shown(3), cup.medal(3, 0)), (0, None));
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
        assert_eq!(cup.before, vec![4, 4, 4]);
        assert_eq!(cup.fresh(), 3);
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
    fn a_tie_below_the_target_is_not_overtime() {
        let (rules, mut cup) = cup(6);
        // 5 each after two matches: tied, but nobody has reached 6 yet.
        cup.add_match(&rules, &[0, 1]);
        cup.add_match(&rules, &[1, 0]);
        assert_eq!(cup.points, vec![5, 5]);
        assert!(!cup.overtime(&rules));
        assert!(cup.in_progress());
    }

    #[test]
    fn a_lone_leader_at_the_target_wins_even_with_a_close_second() {
        let (rules, mut cup) = cup(6);
        cup.add_match(&rules, &[0, 1]); // 3 : 2
        cup.add_match(&rules, &[0, 1]); // 6 : 4
        assert!(!cup.in_progress());
        assert_eq!(cup.standings.as_ref().map(|s| s[0]), Some(0));
        assert!(!cup.overtime(&rules));
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
