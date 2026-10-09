//! Checks the 50 original trials against the PC's readout (docs/research/trials.md):
//! the counts, the mode from each id's prefix, and a few rows copied from the table.

use std::collections::HashSet;

use tricky_towers::config::ModeKind;
use tricky_towers::trial_list::{MAX_TRIAL_LIVES, original_trials};

#[test]
fn there_are_fifty_trials_with_unique_ids() {
    let trials = original_trials();
    assert_eq!(trials.len(), 50);
    let names: HashSet<_> = trials.iter().map(|t| t.name.as_str()).collect();
    assert_eq!(names.len(), 50, "duplicate trial ids");
}

#[test]
fn each_id_prefix_names_its_mode() {
    for t in original_trials() {
        let expected = match &t.name[..2] {
            "SR" => ModeKind::Race,
            "SS" => ModeKind::Survival,
            "SP" => ModeKind::Puzzle,
            other => panic!("{}: unknown prefix {other}", t.name),
        };
        assert_eq!(t.mode, expected, "{}", t.name);
    }
}

#[test]
fn each_mode_has_its_goal_set() {
    for t in original_trials() {
        match t.mode {
            // Race: a height within a time limit.
            ModeKind::Race => {
                assert!(t.finish_height.is_some(), "{}: no height", t.name);
                assert!(t.seconds.is_some(), "{}: no time limit", t.name);
            }
            // Survival: a brick count and the lives.
            ModeKind::Survival => {
                assert!(t.bricks.is_some(), "{}: no brick count", t.name);
                assert!(t.lives.is_some(), "{}: no lives", t.name);
            }
            // Puzzle: a brick count under a laser, with one life.
            ModeKind::Puzzle => {
                assert!(t.laser.is_some(), "{}: no laser", t.name);
                assert!(t.lives.unwrap_or(1) == 1, "{}: puzzle trials have 1 life", t.name);
                assert_eq!(
                    t.order.len(),
                    t.bricks.unwrap_or(0) as usize,
                    "{}: fixed order must have one entry per brick",
                    t.name
                );
            }
            _ => panic!("{}: not a trial mode", t.name),
        }
    }
}

#[test]
fn survival_trials_use_three_lives_except_the_last_master() {
    for t in original_trials() {
        if t.mode != ModeKind::Survival {
            continue;
        }
        let expected = if t.name == "SS_20" {
            MAX_TRIAL_LIVES
        } else {
            3
        };
        assert_eq!(t.lives, Some(expected), "{}", t.name);
    }
}

#[test]
fn rows_copied_from_the_readout_match() {
    let trials = original_trials();
    let find = |name: &str| trials.iter().find(|t| t.name == name).unwrap();

    // Rookie 1: reach 14 cells in 60 s.
    let t = find("SR_10");
    assert_eq!(t.finish_height, Some(14.0));
    assert_eq!(t.seconds, Some(60.0));

    // Rookie 2: place 15 bricks, 3 lives, drop 3, large curses on bricks 5, 10, 15.
    let t = find("SS_17");
    assert_eq!(t.bricks, Some(15));
    assert_eq!(t.lives, Some(3));
    assert_eq!(t.drop_speed, Some(3.0));
    assert_eq!(t.enemy.len(), 3);

    // Rookie 3: place 9 bricks under a laser at 2.2, fixed order O Z S T T O T T I.
    let t = find("SP_20");
    assert_eq!(t.bricks, Some(9));
    assert_eq!(t.laser, Some(2.2));
    assert_eq!(t.order.join(" "), "O Z S T T O T T I");

    // Master 10: place 99 bricks with the game's maximum lives.
    let t = find("SS_20");
    assert_eq!(t.bricks, Some(99));
    assert_eq!(t.lives, Some(MAX_TRIAL_LIVES));

    // Master 8: reach 75 cells in 145 s.
    let t = find("SR_23");
    assert_eq!(t.finish_height, Some(75.0));
    assert_eq!(t.seconds, Some(145.0));
}
