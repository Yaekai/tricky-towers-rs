//! The Trials screen, laid out from the PC readout (docs/research/menus.md "Trials
//! screen layout"): one page per tier, side by side and sliding, each with its 10
//! trials in 2 columns of 5; a medal panel at the top right; a locked panel over
//! tiers not yet earned. Up/Down pick in a column, Left/Right change column (and
//! page at the edges), the bumpers turn the page, Enter or A plays, Esc or B goes
//! back to Single Player.
//!
//! When a tier opens, the screen turns to it and plays the original's unlock: the
//! chain breaks link by link from the far end, the padlock pops and fades, the
//! locked panel slides up and the trial buttons slide in (docs/research/
//! trials-obstacles-roofs.md, section 3). Each tier's unlock plays once
//! (`TrialProgress::tiers_shown`).
//!
//! Starting a trial plays its mode's stinger (art.rs `play_music`).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig, ModeKind};
use crate::gameplay::{Match, MatchRequest, Phase};
use crate::menu::MenuScreen;
use crate::select::{
    CORNER_SMALL, SCREEN, corners, label, label_with, place, read_taps, shown, spawn_page,
};
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
/// The medal panel in the top-right corner: its picture at 1.1x, "MEDALS:" and the count
/// on the left, the medal on the right (measured from Max's screenshots of the original, ?).
const INFO: ((f32, f32), (f32, f32)) = (
    (251.0, 91.0),
    (SCREEN.0 / 2.0 - 135.0, SCREEN.1 / 2.0 - 50.0),
);
const LOCKED: (f32, f32) = (783.0, 215.0);
const LOCK: (f32, f32) = (68.0, 63.0);
const LINK: (f32, f32) = (48.0, 35.0);
const SHORT_LINK: (f32, f32) = (24.0, 17.0);
/// The padlock on the locked panel: centre (px from the panel's centre, y up) and turn.
const PADLOCK: ((f32, f32), f32) = ((-323.0, 28.3), 8.5);
/// The unlock, from the readout: wait after a finished trial; each link shrinks over
/// `LINK_SHRINK`, the far ones first, spread over `LINKS_OVER`; a chain sound at most
/// every `CHAIN_SOUND_GAP`; the padlock pops `LOCK_AFTER` after the nearest link has
/// gone, over `LOCK_POP` to `LOCK_SCALE`, fading over `LOCK_FADE` from `LOCK_FADE_AT`;
/// `PANEL_AFTER` later the panel slides `SLIDE_PX` up over `PANEL_SLIDE` and the
/// buttons come up from `SLIDE_PX` low, `BUTTON_GAP` apart.
const AFTER_TRIAL: f32 = 0.8;
const LINK_SHRINK: f32 = 0.1;
const LINKS_OVER: f32 = 0.6;
const CHAIN_SOUND_GAP: f32 = 0.2;
const LOCK_AFTER: f32 = 0.3;
const LOCK_POP: f32 = 0.45;
const LOCK_SCALE: f32 = 1.3;
const LOCK_FADE_AT: f32 = 0.5;
const LOCK_FADE: f32 = 0.25;
/// When the panel starts after the padlock's fade ends (read as "then, after 0.2 s", ?).
const PANEL_AFTER: f32 = 0.2;
const PANEL_SLIDE: f32 = 0.5;
const BUTTON_GAP: f32 = 0.05;
const SLIDE_PX: f32 = 1000.0;
const PAGE_ARROW: ((f32, f32), f32) = ((94.0, 55.0), 500.0);
const HEADER_Y: f32 = (SCREEN.1 - 90.0) / 2.0;
/// Seconds a page slide takes (LevelDisplay: DOLocalMoveX over 0.3 s).
const SLIDE_SECONDS: f32 = 0.3;

#[derive(Resource, Debug, Default)]
pub struct TrialsMenu {
    pub selected: usize,
    /// Where the pages are, sliding towards the picked trial's tier.
    slide: f32,
    /// The tier unlock playing, if any: the tier and seconds since its chain started
    /// breaking (below 0 while the screen waits and turns to it).
    pub unlock: Option<(usize, f32)>,
    /// A trial was started from here, so the next unlock waits for the screen to settle.
    from_trial: bool,
    /// When the last chain sound played, in unlock seconds.
    chain_sound: f32,
}

/// One piece of the chain on the locked panel.
#[derive(Clone, Copy, Debug, PartialEq)]
enum ChainPiece {
    Long,
    Short,
    End,
}

impl ChainPiece {
    fn art(self) -> &'static str {
        match self {
            ChainPiece::Long => "trial_chain_link",
            ChainPiece::Short => "trial_chain_link_short",
            ChainPiece::End => "trial_chain_link_end",
        }
    }

    fn size(self) -> (f32, f32) {
        match self {
            ChainPiece::Short => SHORT_LINK,
            _ => LINK,
        }
    }
}

/// The chain's links from the readout: kind, centre (px from the panel centre, y up)
/// and turn in degrees. A long run to the right, an end to the left, three hanging
/// down and two going up from the padlock.
fn chain() -> Vec<(ChainPiece, (f32, f32), f32)> {
    let mut links: Vec<_> = (0..13)
        .map(|i| {
            let i = i as f32;
            (
                ChainPiece::Long,
                (-278.0 + 34.6 * i, 35.1 + 5.15 * i),
                188.5,
            )
        })
        .collect();
    links.extend([
        (ChainPiece::Short, (161.3, 101.8), 188.5),
        (ChainPiece::End, (-372.9, 20.9), -351.6),
        (ChainPiece::Long, (-318.4, -16.3), -261.4),
        (ChainPiece::Long, (-313.2, -50.9), -261.4),
        (ChainPiece::End, (-307.9, -85.5), -261.4),
        (ChainPiece::Long, (-332.2, 74.6), 278.7),
        (ChainPiece::Short, (-338.0, 102.2), 278.7),
    ]);
    links
}

/// When link `i` starts to shrink: the far links first, spread over `LINKS_OVER`.
fn link_start(i: usize) -> f32 {
    let links = chain();
    let distance = |(x, y): (f32, f32)| (x - PADLOCK.0.0).hypot(y - PADLOCK.0.1);
    let far = links.iter().map(|l| distance(l.1)).fold(0.0, f32::max);
    links
        .get(i)
        .map_or(0.0, |l| (1.0 - distance(l.1) / far.max(1.0)) * LINKS_OVER)
}

/// When the padlock pops: `LOCK_AFTER` after the nearest link has gone.
fn padlock_start() -> f32 {
    let last = (0..chain().len()).map(link_start).fold(0.0, f32::max);
    last + LINK_SHRINK + LOCK_AFTER
}

/// When the panel starts sliding up.
fn panel_start() -> f32 {
    padlock_start() + LOCK_FADE_AT + LOCK_FADE + PANEL_AFTER
}

/// When the whole unlock is over: the last button is in place.
fn unlock_end(per_tier: usize) -> f32 {
    panel_start() + BUTTON_GAP * per_tier.saturating_sub(1) as f32 + PANEL_SLIDE
}

fn ease_out_back(x: f32) -> f32 {
    let (c1, c3) = (1.70158, 2.70158);
    1.0 + c3 * (x - 1.0).powi(3) + c1 * (x - 1.0).powi(2)
}

/// A rotation in the original's terms (degrees, anticlockwise) for a UI node, whose
/// y axis points down (?: checked by eye on the PC).
fn turn(degrees: f32) -> Rot2 {
    Rot2::degrees(-degrees)
}

#[derive(Component)]
pub(crate) struct TrialsRoot;

/// A Trials node: what it is and which tier or trial it belongs to.
#[derive(Component, Clone, Copy, Debug, PartialEq)]
pub(crate) enum TrialPart {
    Page(usize),
    /// A trial's button and everything on it, to slide in when its tier opens.
    Button(usize),
    Corner(usize),
    Medal(usize),
    /// A tier's locked panel with its chain, to slide away when it opens.
    Locked(usize),
    LockedText(usize),
    Link(usize, usize),
    Padlock(usize),
    Title,
    MedalCount,
    PageArrow(bool),
    /// The mode's wizard beside a trial's button, shown on the picked one only.
    Face(usize),
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
                    h.spawn((
                        label_with(4.5, "", crate::letters::TextLook::TITLE),
                        TrialPart::Title,
                    ));
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
                let medal_size = (MEDAL.0.0 * 0.6, MEDAL.0.1 * 0.6);
                root.spawn((
                    ImageNode::new(medal),
                    place(medal_size, (left + size.0 - 40.0, at.1)),
                ));
            }
            root.spawn(place((160.0, 40.0), (left + 95.0, at.1 + 18.0)))
                .with_children(|t| {
                    t.spawn((
                        label_with(vh(22.0), "MEDALS:", crate::letters::TextLook::OUTLINED),
                        crate::letters::LetterSpacing(-1.0),
                    ));
                });
            root.spawn(place((160.0, 60.0), (left + 95.0, at.1 - 18.0)))
                .with_children(|t| {
                    t.spawn((
                        label_with(vh(36.0), "", crate::letters::TextLook::OUTLINED),
                        TrialPart::MedalCount,
                    ));
                });
            for right in [false, true] {
                let x = if right { PAGE_ARROW.1 } else { -PAGE_ARROW.1 };
                let mut arrow = root.spawn((
                    place(PAGE_ARROW.0, (x, TOP_ROW - ROW_STEP * 2.0)),
                    TrialPart::PageArrow(right),
                ));
                match image("menu_arrow") {
                    Some(art) => {
                        // The staged arrow points up; the original's point out sideways.
                        arrow.insert((
                            ImageNode::new(art),
                            UiTransform::from_rotation(Rot2::degrees(if right {
                                90.0
                            } else {
                                -90.0
                            })),
                        ));
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
    page.spawn((full_page(), TrialPart::Button(index)))
        .with_children(|page| {
            button_parts(page, image, index, per_tier, (x, y), (picture, size, face))
        });
}

fn full_page() -> Node {
    Node {
        position_type: PositionType::Absolute,
        width: Val::Percent(100.0),
        height: Val::Percent(100.0),
        ..default()
    }
}

fn button_parts(
    page: &mut ChildSpawnerCommands,
    image: &impl Fn(&str) -> Option<Handle<Image>>,
    index: usize,
    per_tier: usize,
    (x, y): (f32, f32),
    (picture, size, face): (&str, (f32, f32), usize),
) {
    let mut button = page.spawn((
        place(size, (x, y)),
        BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.8)),
    ));
    if let Some(art) = image(picture) {
        button.insert((ImageNode::new(art), BackgroundColor(Color::NONE)));
    }
    if let Some(art) = image(&format!("menu_challenge_wizard{face}")) {
        page.spawn((
            ImageNode::new(art),
            place(FACE.0, (x + FACE.1, y)),
            TrialPart::Face(index),
        ));
    }
    page.spawn(place(BUTTON, (x, y))).with_children(|n| {
        n.spawn(label_with(
            vh(NUMBER_SIZE),
            &(index % per_tier + 1).to_string(),
            crate::letters::TextLook::OUTLINED,
        ));
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
    page.spawn((full_page(), TrialPart::Locked(tier)))
        .with_children(|page| {
            let mut panel = page.spawn((
                place(LOCKED, (0.0, y)),
                BackgroundColor(Color::srgba(0.1, 0.08, 0.2, 0.92)),
            ));
            if let Some(art) = image("trial_locked_background") {
                // A 9-slice picture (the sprite's borders: left 20, right 24, top 46,
                // bottom 10).
                panel.insert((
                    crate::select::sliced(art, [20.0, 24.0, 46.0, 10.0]),
                    BackgroundColor(Color::NONE),
                ));
            }
            // The original's WorldSelectListItem: LOCKED in pink across the panel's top
            // edge left of centre, "EARN 8 / TO UNLOCK" right of centre and the medal at
            // the right end.
            let locked = Color::srgb(1.0, 0.31, 0.55);
            page.spawn(place((393.0, 80.0), (-105.0, y + LOCKED.1 / 2.0)))
                .with_children(|t| {
                    t.spawn(label_with(
                        vh(60.0),
                        "LOCKED",
                        crate::letters::TextLook::OUTLINED,
                    ))
                    .insert((TextColor(locked), crate::letters::LetterSpacing(-1.0)));
                });
            page.spawn(place((348.0, 150.0), (82.5, y)))
                .with_children(|t| {
                    t.spawn((
                        label_with(vh(46.0), "", crate::letters::TextLook::OUTLINED),
                        TrialPart::LockedText(tier),
                    ));
                });
            if let Some(art) = image("medal_large") {
                page.spawn((
                    ImageNode::new(art),
                    place((70.0, 96.0), (LOCKED.0 / 2.0 - 83.0, y + 0.8)),
                ));
            }
            for (i, (link, (x, dy), degrees)) in chain().into_iter().enumerate() {
                if let Some(art) = image(link.art()) {
                    page.spawn((
                        ImageNode::new(art),
                        place(link.size(), (x, y + dy)),
                        UiTransform {
                            rotation: turn(degrees),
                            ..default()
                        },
                        TrialPart::Link(tier, i),
                    ));
                }
            }
            let ((x, dy), degrees) = PADLOCK;
            let mut lock = page.spawn((
                place(LOCK, (x, y + dy)),
                UiTransform {
                    rotation: turn(degrees),
                    ..default()
                },
                TrialPart::Padlock(tier),
            ));
            if let Some(art) = image("trial_lock") {
                lock.insert(ImageNode::new(art));
            }
        });
}

/// Tiers shown as open so far: the saved count, and at least every tier that needs
/// no medals (those never play an unlock).
fn tiers_shown(progress: &TrialProgress, tiers: &crate::config::TrialTiers) -> usize {
    let free = tiers
        .tiers
        .iter()
        .take_while(|(_, need)| *need == 0)
        .count();
    progress.tiers_shown.max(free)
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
    // Nothing moves while a tier unlocks.
    if menu.unlock.is_some() {
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
        menu.from_trial = true;
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
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Match>,
    screen: Res<MenuScreen>,
    mut menu: ResMut<TrialsMenu>,
    config: Res<GameConfig>,
    mut progress: ResMut<TrialProgress>,
    art: Option<Res<Art>>,
    mut root: Query<&mut Visibility, With<TrialsRoot>>,
    mut parts: Query<(&TrialPart, &mut Node, &mut Visibility), Without<TrialsRoot>>,
    mut turns: Query<(&TrialPart, &mut UiTransform, Option<&mut ImageNode>)>,
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
    let pages = config.trials.len().div_ceil(per_tier);
    let medals = progress.medals(tiers);
    let shown_tiers = tiers_shown(&progress, tiers);
    // A tier that has opened since the screen last showed it plays its unlock:
    // the screen turns to it first.
    if menu.unlock.is_none() {
        if shown_tiers < pages && tiers.is_open(shown_tiers * per_tier, medals) {
            let wait = if menu.from_trial { AFTER_TRIAL } else { 0.0 };
            menu.unlock = Some((shown_tiers, -(wait + SLIDE_SECONDS)));
            menu.selected = shown_tiers * per_tier;
            menu.chain_sound = f32::MIN;
        }
        menu.from_trial = false;
    }
    let before = menu.unlock.map(|(_, t)| t);
    if let Some((opening, t)) = menu.unlock.as_mut() {
        *t += time.delta_secs();
        if *t >= unlock_end(per_tier) {
            progress.tiers_shown = *opening + 1;
            progress.save();
            menu.unlock = None;
        }
    }
    play_unlock_sounds(&mut commands, art.as_deref(), &mut menu, before);
    let tier = menu.selected / per_tier;
    let target = tier as f32;
    let step = time.delta_secs() / SLIDE_SECONDS;
    menu.slide = if (menu.slide - target).abs() <= step {
        target
    } else {
        menu.slide + step * (target - menu.slide).signum()
    };
    // How far a part has slid: 0 in place, 1 a whole `SLIDE_PX` away.
    let unlocking = |t: usize| menu.unlock.filter(|(o, _)| *o == t).map(|(_, time)| time);
    let slid = |time: f32, start: f32| ((time - start) / PANEL_SLIDE).clamp(0.0, 1.0);
    let shown_tiers = tiers_shown(&progress, tiers);
    let moved = |node: &mut Node, share: f32| {
        let top = Val::Percent(share * SLIDE_PX / SCREEN.1 * 100.0);
        if node.top != top {
            node.top = top;
        }
    };
    for (part, mut node, mut visibility) in &mut parts {
        let visible = match *part {
            TrialPart::Page(p) => {
                let left = Val::Percent((p as f32 - menu.slide) * 100.0);
                if node.left != left {
                    node.left = left;
                }
                true
            }
            // A tier's buttons wait below the screen until its unlock brings them up.
            TrialPart::Button(i) => {
                let t = i / per_tier;
                let low = match unlocking(t) {
                    Some(time) => {
                        let start = panel_start() + BUTTON_GAP * (i % per_tier) as f32;
                        1.0 - slid(time, start)
                    }
                    None if t < shown_tiers => 0.0,
                    None => 1.0,
                };
                moved(&mut node, low);
                true
            }
            TrialPart::Locked(t) => match unlocking(t) {
                Some(time) => {
                    moved(&mut node, -slid(time, panel_start()));
                    true
                }
                None => {
                    moved(&mut node, 0.0);
                    t >= shown_tiers
                }
            },
            TrialPart::Corner(i) | TrialPart::Face(i) => i == menu.selected,
            TrialPart::Medal(i) => config
                .trials
                .get(i)
                .is_some_and(|t| progress.is_done(&t.name)),
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
    // The chain breaks and the padlock pops on the tier opening.
    for (part, mut transform, image) in &mut turns {
        match *part {
            TrialPart::Link(t, i) => {
                let gone = unlocking(t).map_or(0.0, |time| {
                    ((time - link_start(i)) / LINK_SHRINK).clamp(0.0, 1.0)
                });
                transform.scale = Vec2::splat(1.0 - gone);
            }
            TrialPart::Padlock(t) => {
                let since = unlocking(t).map_or(f32::MIN, |time| time - padlock_start());
                let pop = (since / LOCK_POP).clamp(0.0, 1.0);
                let fade = ((since - LOCK_FADE_AT) / LOCK_FADE).clamp(0.0, 1.0);
                transform.scale = Vec2::splat(1.0 + (LOCK_SCALE - 1.0) * ease_out_back(pop));
                transform.rotation = turn(PADLOCK.1 * (1.0 - ease_out_back(pop)));
                if let Some(mut image) = image {
                    let alpha = 1.0 - fade;
                    if image.color.alpha() != alpha {
                        image.color.set_alpha(alpha);
                    }
                }
            }
            _ => {}
        }
    }
    let tier_name = |t: usize| tiers.tiers.get(tiers.tier_of(t * per_tier));
    for (part, mut words) in &mut texts {
        let line = match *part {
            TrialPart::Title => {
                // "ROOKIE TRIALS", as the original's tier titles read.
                tier_name(tier).map_or(String::new(), |(name, _)| {
                    format!("{} TRIALS", name.to_uppercase())
                })
            }
            // The count against all the medals there are (?).
            TrialPart::MedalCount => format!(
                "{medals}/{}",
                config.trials.len() as u32 * tiers.medals_per_trial
            ),
            TrialPart::LockedText(t) => {
                let need = tier_name(t).map_or(0, |(_, need)| *need);
                // The original's SINGLE_PLAYER_LEVEL_UNLOCK_TARGET, on two lines.
                format!("EARN {need}\nTO UNLOCK")
            }
            _ => continue,
        };
        if words.0 != line {
            words.0 = line;
        }
    }
}

/// The unlock's sounds, for the stretch of time from `before` to now: a chain sound
/// as links start to go (one of 4, at most every `CHAIN_SOUND_GAP`), then the brick
/// spawn and roof placed sounds as the padlock pops.
fn play_unlock_sounds(
    commands: &mut Commands,
    art: Option<&Art>,
    menu: &mut TrialsMenu,
    before: Option<f32>,
) {
    let (Some(before), Some(now)) = (before, menu.unlock.map(|(_, t)| t).or(before)) else {
        return;
    };
    let mut play = |name: &str| {
        if let Some(handle) = art.and_then(|a| a.sounds.get(name)) {
            commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
        }
    };
    let crossed = |at: f32| before < at && at <= now;
    for i in 0..chain().len() {
        let at = link_start(i);
        if crossed(at) && at - menu.chain_sound >= CHAIN_SOUND_GAP {
            menu.chain_sound = at;
            // One of the 4 chain sounds, varied by link.
            play(&format!("sfx_chain_{}", i * 3 % 4 + 1));
        }
    }
    if crossed(padlock_start()) {
        play("sfx_brick_spawn");
        play("sfx_roof_placed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_chain_breaks_from_the_far_end_then_the_padlock_pops() {
        let links = chain();
        assert_eq!(links.len(), 20);
        // The short link at the long run's far end goes first, the links beside the
        // padlock last.
        let far = 13;
        let near = links.len() - 2;
        assert!(link_start(far) < 0.05, "{}", link_start(far));
        assert!(link_start(near) > 0.5);
        assert!((0..links.len()).all(|i| (0.0..=LINKS_OVER).contains(&link_start(i))));
        let last = (0..links.len()).map(link_start).fold(0.0, f32::max);
        assert!((padlock_start() - (last + LINK_SHRINK + LOCK_AFTER)).abs() < 1e-5);
        assert!(panel_start() > padlock_start() + LOCK_FADE_AT);
        assert!((unlock_end(10) - (panel_start() + 0.45 + PANEL_SLIDE)).abs() < 1e-5);
        assert!((ease_out_back(1.0) - 1.0).abs() < 1e-5 && ease_out_back(0.0).abs() < 1e-5);
    }

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
