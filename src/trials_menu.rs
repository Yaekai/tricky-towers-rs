//! The Trials screen, laid out from the PC readout (docs/research/menus.md "Trials
//! screen layout"): one page per tier, side by side and sliding, each with its 10
//! trials in 2 columns of 5; a medal panel at the top right; a locked panel over
//! tiers not yet earned. Up/Down pick in a column, Left/Right change column (and
//! page at the edges), the bumpers turn the page, Enter or A plays, Esc or B goes
//! back to Single Player.
//!
//! Not done: the unlock animation (lock pop, chain sounds, panel sliding up) and
//! the mode stinger when a trial starts.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase};
use crate::menu::MenuScreen;
use crate::select::{CORNER_SMALL, SCREEN, corners, label, place, read_taps, shown, spawn_page};
use crate::trials::TrialProgress;

/// Layout from the readout, in 1920×1080 pixels from the screen centre (y up).
const BUTTON: (f32, f32) = (423.0, 101.0);
const ROW_STEP: f32 = 125.0;
const COLUMN_STEP: f32 = 440.0;
/// The top row's centre (?: the readout gives the spacing, not the place).
const TOP_ROW: f32 = 230.0;
const ROWS: usize = 5;
const NUMBER_SIZE: f32 = 46.0;
const FACE: ((f32, f32), f32) = ((84.0, 84.0), -119.0);
const MEDAL: ((f32, f32), f32) = ((70.0, 96.0), 145.0);
const CORNER_FRAME: (f32, f32) = (449.0, 134.0);
/// The medal panel, its centre 120 px in from the right edge and 100 px down from
/// the top (read as "120 right of, 100 above the corner", taken inward, ?).
const INFO: ((f32, f32), (f32, f32)) = (
    (382.0, 183.0),
    (SCREEN.0 / 2.0 - 120.0, SCREEN.1 / 2.0 - 100.0),
);
const LOCKED: (f32, f32) = (783.0, 215.0);
const LOCK: (f32, f32) = (68.0, 63.0);
const LINK: (f32, f32) = (48.0, 35.0);
const PAGE_ARROW: ((f32, f32), f32) = ((94.0, 55.0), 500.0);
const HEADER_Y: f32 = (SCREEN.1 - 90.0) / 2.0;
/// Seconds a page slide takes (?).
const SLIDE_SECONDS: f32 = 0.3;

#[derive(Resource, Debug, Default)]
pub struct TrialsMenu {
    pub selected: usize,
    /// Where the pages are, sliding towards the picked trial's tier.
    slide: f32,
}

#[derive(Component)]
pub(crate) struct TrialsRoot;

/// A Trials node: what it is and which tier or trial it belongs to.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) enum TrialPart {
    Page(usize),
    Corner(usize),
    Medal(usize),
    Locked(usize),
    LockedText(usize),
    Title,
    MedalCount,
    PageArrow(bool),
}

/// Trial `index`'s place on its page: column-major, 1-5 down the left (?).
fn slot(index: usize, per_tier: usize) -> (f32, f32) {
    let local = index % per_tier.max(1);
    let (column, row) = (local / ROWS, local % ROWS);
    (
        (column as f32 - 0.5) * COLUMN_STEP,
        TOP_ROW - ROW_STEP * row as f32,
    )
}

fn button_art(mode: ModeKind) -> (&'static str, (f32, f32), usize) {
    // Picture, its size, and the wizard face shown for the mode.
    match mode.rules() {
        ModeKind::Survival => ("trial_button_survival", (399.0, 90.0), 2),
        ModeKind::Puzzle => ("trial_button_puzzle", (423.0, 101.0), 4),
        _ => ("trial_button_race", (405.0, 101.0), 3),
    }
}

fn vh(px: f32) -> f32 {
    px / SCREEN.1 * 100.0
}

pub(crate) fn spawn_trials_screen(
    mut commands: Commands,
    art: Option<Res<Art>>,
    config: Res<GameConfig>,
) {
    let art = art.as_deref();
    let image = |name: &str| art.and_then(|a| a.image(name));
    let tiers = &config.trial_tiers;
    let per_tier = tiers.size.max(1);
    let pages = config.trials.len().div_ceil(per_tier);
    let root = spawn_page(&mut commands, art, "");
    commands
        .entity(root)
        .insert(TrialsRoot)
        .with_children(|root| {
            root.spawn(place((800.0, 90.0), (0.0, HEADER_Y)))
                .with_children(|h| {
                    h.spawn((label(4.5, ""), TrialPart::Title));
                });
            for tier in 0..pages {
                root.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        width: Val::Percent(100.0),
                        height: Val::Percent(100.0),
                        ..default()
                    },
                    TrialPart::Page(tier),
                ))
                .with_children(|page| {
                    let first = tier * per_tier;
                    for index in first..(first + per_tier).min(config.trials.len()) {
                        spawn_button(page, &image, index, config.trials[index].mode, per_tier);
                    }
                    spawn_locked(page, &image, tier);
                });
            }
            // The medal panel.
            let (size, at) = INFO;
            let mut info = root.spawn((
                place(size, at),
                BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.5)),
            ));
            if let Some(back) = image("menu_info_background") {
                info.insert((ImageNode::new(back), BackgroundColor(Color::NONE)));
            }
            let left = at.0 - size.0 / 2.0;
            if let Some(medal) = image("medal_large") {
                root.spawn((ImageNode::new(medal), place(MEDAL.0, (left + 70.0, at.1))));
            }
            root.spawn(place((160.0, 40.0), (left + 200.0, at.1 + 35.0)))
                .with_children(|t| {
                    t.spawn(label(vh(28.0), "MEDALS:"));
                });
            root.spawn(place((160.0, 60.0), (left + 200.0, at.1 - 20.0)))
                .with_children(|t| {
                    t.spawn((label(vh(NUMBER_SIZE), ""), TrialPart::MedalCount));
                });
            for right in [false, true] {
                let x = if right { PAGE_ARROW.1 } else { -PAGE_ARROW.1 };
                let mut arrow = root.spawn((
                    place(PAGE_ARROW.0, (x, TOP_ROW - ROW_STEP * 2.0)),
                    TrialPart::PageArrow(right),
                ));
                match image("menu_arrow") {
                    Some(art) => {
                        arrow.insert(ImageNode {
                            image: art,
                            flip_x: !right,
                            ..default()
                        });
                    }
                    None => {
                        arrow.with_children(|a| {
                            a.spawn(label(4.0, if right { ">" } else { "<" }));
                        });
                    }
                }
            }
        });
}

fn spawn_button(
    page: &mut ChildSpawnerCommands,
    image: &impl Fn(&str) -> Option<Handle<Image>>,
    index: usize,
    mode: ModeKind,
    per_tier: usize,
) {
    let (x, y) = slot(index, per_tier);
    let (picture, size, face) = button_art(mode);
    let mut button = page.spawn((
        place(size, (x, y)),
        BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.8)),
    ));
    if let Some(art) = image(picture) {
        button.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
    }
    if let Some(art) = image(&format!("menu_challenge_wizard{face}")) {
        page.spawn((ImageNode::new(art), place(FACE.0, (x + FACE.1, y))));
    }
    page.spawn(place(BUTTON, (x, y))).with_children(|n| {
        n.spawn(label(vh(NUMBER_SIZE), &(index % per_tier + 1).to_string()));
    });
    let mut medal = page.spawn((place(MEDAL.0, (x + MEDAL.1, y)), TrialPart::Medal(index)));
    match image("medal_large") {
        Some(art) => {
            medal.insert(ImageNode::new(art));
        }
        None => {
            medal.insert(BackgroundColor(Color::srgb(0.95, 0.75, 0.2)));
        }
    }
    match image("menu_corner_small") {
        Some(corner) => corners(
            page,
            &corner,
            CORNER_FRAME,
            (x, y),
            CORNER_SMALL,
            TrialPart::Corner(index),
        ),
        None => {
            page.spawn((
                place(CORNER_FRAME, (x, y)),
                Outline::new(Val::Px(3.0), Val::ZERO, Color::WHITE),
                TrialPart::Corner(index),
            ));
        }
    }
}

/// The panel over a locked tier: LOCKED, what it takes, and a padlock on a chain.
fn spawn_locked(
    page: &mut ChildSpawnerCommands,
    image: &impl Fn(&str) -> Option<Handle<Image>>,
    tier: usize,
) {
    let y = TOP_ROW - ROW_STEP * 2.0;
    let mut panel = page.spawn((
        place(LOCKED, (0.0, y)),
        BackgroundColor(Color::srgba(0.1, 0.08, 0.2, 0.92)),
        TrialPart::Locked(tier),
    ));
    if let Some(art) = image("trial_locked_background") {
        panel.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
    }
    panel.with_children(|p| {
        p.spawn(Node {
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            ..default()
        })
        .with_children(|c| {
            c.spawn(label(vh(60.0), "LOCKED"));
            c.spawn((label(vh(NUMBER_SIZE), ""), TrialPart::LockedText(tier)));
        });
    });
    // The chain runs across the panel's foot with the padlock in the middle (?).
    let chain_y = y - LOCKED.1 / 2.0;
    let links = (LOCKED.0 / LINK.0) as i32;
    for i in 0..links {
        let x = -LOCKED.0 / 2.0 + LINK.0 * (i as f32 + 0.5);
        let name = if i == 0 || i == links - 1 {
            "trial_chain_link_end"
        } else {
            "trial_chain_link"
        };
        if let Some(art) = image(name) {
            page.spawn((
                ImageNode {
                    image: art,
                    flip_x: i == links - 1,
                    ..default()
                },
                place(LINK, (x, chain_y)),
                TrialPart::Locked(tier),
            ));
        }
    }
    let mut lock = page.spawn((place(LOCK, (0.0, chain_y)), TrialPart::Locked(tier)));
    if let Some(art) = image("trial_lock") {
        lock.insert(ImageNode::new(art));
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn trials_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<TrialsMenu>,
    mut config: ResMut<GameConfig>,
    mut request: ResMut<MatchRequest>,
    progress: Res<TrialProgress>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Trials {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Single;
        return;
    }
    let count = config.trials.len();
    if count == 0 {
        return;
    }
    let bumper = |button| pads.iter().any(|(_, pad)| pad.just_pressed(button));
    let turn =
        bumper(GamepadButton::RightTrigger) as i32 - bumper(GamepadButton::LeftTrigger) as i32;
    let dir = (
        taps.right as i32 - taps.left as i32,
        taps.down as i32 - taps.up as i32,
    );
    menu.selected = step(
        menu.selected.min(count - 1),
        dir,
        turn,
        config.trial_tiers.size,
        count,
    );
    if taps.key_pick || !taps.pad_pick.is_empty() {
        // A tier opens once enough medals are earned.
        let medals = progress.medals(&config.trial_tiers);
        if !config.trial_tiers.is_open(menu.selected, medals) {
            return;
        }
        config.trial = Some(menu.selected);
        // A trial's own values go over Normal's (?).
        config.difficulty = Some(Difficulty::Normal);
        request.pending = true;
    }
}

/// Moves the pick: `dir` is (columns right, rows down), `turn` pages. Rows wrap
/// in a column; Left/Right past the edge column turns the page.
fn step(selected: usize, dir: (i32, i32), turn: i32, per_tier: usize, count: usize) -> usize {
    let per_tier = per_tier.max(1);
    let pages = count.div_ceil(per_tier) as i32;
    let mut tier = (selected / per_tier) as i32;
    let local = selected % per_tier;
    let rows = ROWS.min(per_tier) as i32;
    let columns = per_tier.div_ceil(ROWS) as i32;
    let (mut column, mut row) = ((local / ROWS) as i32, (local % ROWS) as i32);
    row = (row + dir.1).rem_euclid(rows);
    column += dir.0;
    if column < 0 && tier > 0 {
        tier -= 1;
        column = columns - 1;
    } else if column >= columns && tier + 1 < pages {
        tier += 1;
        column = 0;
    }
    column = column.clamp(0, columns - 1);
    tier = (tier + turn).clamp(0, pages - 1);
    let index = tier as usize * per_tier + (column * ROWS as i32 + row) as usize;
    index.min(count - 1)
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_trials_screen(
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut menu: ResMut<TrialsMenu>,
    config: Res<GameConfig>,
    progress: Res<TrialProgress>,
    mut root: Query<&mut Visibility, With<TrialsRoot>>,
    mut parts: Query<(&TrialPart, &mut Node, &mut Visibility), Without<TrialsRoot>>,
    mut texts: Query<(&TrialPart, &mut Text)>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Trials;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    let tiers = &config.trial_tiers;
    let per_tier = tiers.size.max(1);
    let tier = menu.selected / per_tier;
    let pages = config.trials.len().div_ceil(per_tier);
    let target = tier as f32;
    let step = time.delta_secs() / SLIDE_SECONDS;
    menu.slide = if (menu.slide - target).abs() <= step {
        target
    } else {
        menu.slide + step * (target - menu.slide).signum()
    };
    let medals = progress.medals(tiers);
    for (part, mut node, mut visibility) in &mut parts {
        let visible = match *part {
            TrialPart::Page(p) => {
                let left = Val::Percent((p as f32 - menu.slide) * 100.0);
                if node.left != left {
                    node.left = left;
                }
                true
            }
            TrialPart::Corner(i) => i == menu.selected,
            TrialPart::Medal(i) => config
                .trials
                .get(i)
                .is_some_and(|t| progress.is_done(&t.name)),
            TrialPart::Locked(t) => !tiers.is_open(t * per_tier, medals),
            TrialPart::PageArrow(right) => {
                if right {
                    tier + 1 < pages
                } else {
                    tier > 0
                }
            }
            _ => true,
        };
        visibility.set_if_neq(shown(visible));
    }
    let tier_name = |t: usize| tiers.tiers.get(tiers.tier_of(t * per_tier));
    for (part, mut words) in &mut texts {
        let line = match *part {
            TrialPart::Title => {
                tier_name(tier).map_or(String::new(), |(name, _)| name.to_uppercase())
            }
            // The count against all the medals there are (?).
            TrialPart::MedalCount => format!(
                "{medals}/{}",
                config.trials.len() as u32 * tiers.medals_per_trial
            ),
            TrialPart::LockedText(t) => {
                let need = tier_name(t).map_or(0, |(_, need)| *need);
                format!("Earn {need} medals to unlock")
            }
            _ => continue,
        };
        if words.0 != line {
            words.0 = line;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trials_sit_in_two_columns_of_five() {
        assert_eq!(slot(0, 10), (-220.0, TOP_ROW));
        assert_eq!(slot(4, 10), (-220.0, TOP_ROW - 500.0));
        assert_eq!(slot(5, 10), (220.0, TOP_ROW));
        assert_eq!(slot(19, 10), (220.0, TOP_ROW - 500.0));
    }

    #[test]
    fn arrows_move_in_columns_and_turn_pages_at_the_edges() {
        // Down and up wrap in a column.
        assert_eq!(step(0, (0, 1), 0, 10, 50), 1);
        assert_eq!(step(0, (0, -1), 0, 10, 50), 4);
        // Right moves to the next column, then the next page.
        assert_eq!(step(1, (1, 0), 0, 10, 50), 6);
        assert_eq!(step(6, (1, 0), 0, 10, 50), 11);
        assert_eq!(step(11, (-1, 0), 0, 10, 50), 6);
        // No page before the first or after the last.
        assert_eq!(step(2, (-1, 0), 0, 10, 50), 2);
        assert_eq!(step(47, (1, 0), 0, 10, 50), 47);
        // The bumpers turn the page and keep the place.
        assert_eq!(step(3, (0, 0), 1, 10, 50), 13);
        assert_eq!(step(13, (0, 0), -1, 10, 50), 3);
    }
}
