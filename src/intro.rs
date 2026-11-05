//! A battle's opening, like the original's INTRO and COUNTDOWN states
//! (docs/research/match-moments.md, "Match intro"): the mode title ribbon
//! ("RACE NORMAL", OVERTIME under it in a tied cup), then the mode's own intro
//! (Race: the camera up to the finish line and back; Survival: the brick count
//! counting up; Puzzle: two waits), then 3-2-1 and play. Rematch and Retry skip the
//! mode's intro; later matches of a cup skip the title unless it's overtime.
//! Timings and layout are `GameConfig::intro`.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{curve, keyed, Difficulty, GameConfig, IntroPlan, ModeKind};
use crate::cups::Cup;
use crate::gameplay::{IntroOn, Match, Phase};
use crate::letters::TextLook;
use crate::popup::{px, sliced};

pub struct IntroPlugin;

impl Plugin for IntroPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IntroOn)
            .init_resource::<Intro>()
            .init_resource::<CameraLift>()
            .init_resource::<IntroCount>()
            .init_resource::<LaserIn>()
            .init_resource::<EclipseIn>()
            .init_resource::<IntroClock>()
            .add_systems(Startup, spawn_intro)
            .add_systems(Update, (run_intro, show_title, show_countdown).chain());
    }
}

/// Cells the intro lifts the camera by (Race's sweep up to the finish line).
#[derive(Resource, Default)]
pub struct CameraLift(pub f32);

/// The Survival brick count shown while it counts up, instead of the real one.
#[derive(Resource, Default)]
pub struct IntroCount(pub Option<u32>);

/// Seconds since Puzzle's laser started coming in (below 0 while it waits); `None`
/// shows the laser as it is.
#[derive(Resource, Default)]
pub struct LaserIn(pub Option<f32>);

/// Seconds into the current match's intro (it keeps counting through the 3-2-1 and on),
/// `None` without one.
#[derive(Resource, Default)]
pub struct IntroClock(pub Option<f32>);

/// Seconds since Endless Puzzle's eclipse started coming on (below 0 while it waits,
/// infinite when a Retry skips it); `None` without an intro.
#[derive(Resource, Default)]
pub struct EclipseIn(pub Option<f32>);

#[derive(Resource, Default)]
struct Intro {
    round: u64,
    clock: f32,
    plan: Option<IntroPlan>,
    /// 0 Easy, 1 Normal, 2 Special.
    difficulty: usize,
    overtime: bool,
    race: bool,
}

/// How much narrower than the band each difficulty's title and OVERTIME labels are (px).
const TITLE_INSETS: [(f32, f32); 3] = [(200.0, 200.0), (400.0, 500.0), (560.0, 700.0)];

const GOLD: Color = Color::srgb(1.0, 0.824, 0.314);

#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    /// One title band per difficulty, and its words.
    Band(usize),
    Words(usize),
    Overtime(usize),
    Count,
    CountBack,
    CountFace,
}

fn spawn_intro(mut commands: Commands, config: Res<GameConfig>, art: Option<Res<Art>>) {
    let look = &config.intro;
    let image = |name: &str| art.as_deref().and_then(|a| a.image(name));
    let outlined = TextLook {
    low: None,
    outline_edge: 0.0,
    bevel: false,
        body: false,
        face: None,
        outline: TextLook::OUTLINED.outline,
    };
    // ModeTitleMessage*'s Label (EnzoBlackGold46) and OvertimeLabel (EnzoBlackGold26), spaced
    // -1, auto-sized from their largest size (160 and 46 px) to fit the band less 400 and 500
    // px (Normal's; `TITLE_INSETS`), kept fitted to the band as it opens.
    let words = |size: f32, look: TextLook| {
        (
            Text::new(""),
            crate::letters::LetterSpacing(-1.0),
            TextFont {
                font_size: FontSize::Vh(size / 10.8),
                ..default()
            },
            TextColor(GOLD),
            TextLayout::justify(Justify::Center),
            look,
            crate::letters::AutoSize { min: 18.0 / 10.8, width: f32::MAX },
        )
    };
    for d in 0..3 {
        let art_name = &look.title_art[d];
        let (side_w, side_h) = look.title_side;
        let (star_w, star_h) = look.title_star;
        let h = look.title_height;
        let mut band = commands.spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                height: px(h),
                ..default()
            },
            UiTransform::default(),
            GlobalZIndex(19),
            Visibility::Hidden,
            Part::Band(d),
        ));
        band.with_children(|band| {
            // The ribbon ends hang off each end, the right one mirrored.
            let side = image(&format!("menu_mode_title_side_{art_name}"));
            let (dx, dy) = look.title_side_at;
            let top = px(h / 2.0 - dy - side_h / 2.0);
            for flip in [false, true] {
                let outside = px(-dx.abs() - side_w / 2.0);
                let mut node = Node {
                    position_type: PositionType::Absolute,
                    top,
                    width: px(side_w),
                    height: px(side_h),
                    ..default()
                };
                if flip {
                    node.right = outside;
                } else {
                    node.left = outside;
                }
                let mut end = band.spawn(node);
                if let Some(picture) = side.clone() {
                    end.insert(ImageNode {
                        image: picture,
                        flip_x: flip,
                        ..default()
                    });
                }
            }
            // The band over its ends (ModeTitleMessage*: BackgroundLeft/Right come before
            // BackgroundSprite), so the ends show only below and beside it.
            let (left, right) = look.title_slice[d];
            let mut fill = band.spawn(Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            });
            if let Some(picture) = image(&format!("menu_mode_title_background_{art_name}")) {
                fill.insert(sliced(picture, [left, right, 0.0, 0.0]));
            }
            let star = image(&format!("menu_mode_title_star_{art_name}"));
            for &from_end in &look.title_stars[d] {
                for flip in [false, true] {
                    let inset = px(from_end - star_w / 2.0);
                    let mut node = Node {
                        position_type: PositionType::Absolute,
                        top: px((h - star_h) / 2.0),
                        width: px(star_w),
                        height: px(star_h),
                        ..default()
                    };
                    if flip {
                        node.right = inset;
                    } else {
                        node.left = inset;
                    }
                    let mut dot = band.spawn(node);
                    if let Some(picture) = star.clone() {
                        dot.insert(ImageNode::new(picture));
                    }
                }
            }
            // The words' bottom sits `title_bottom` below the band's centre.
            for (part, size, bottom, face) in [
                (Part::Words(d), 160.0, look.title_bottom, TextLook::GOLD),
                (Part::Overtime(d), 46.0, look.overtime_bottom, TextLook::GOLD_75),
            ] {
                band.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: px(-400.0),
                        right: px(-400.0),
                        bottom: px(h / 2.0 - bottom),
                        justify_content: JustifyContent::Center,
                        ..default()
                    },
                    children![(words(size, face), part)],
                ));
            }
        });
    }
    // The 3-2-1: a darker copy behind, stretched 10% taller, and the face.
    let shadow = config.moments.shadow_color;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                top: px(540.0 - look.count_y - 100.0),
                height: px(200.0),
                display: Display::Grid,
                justify_items: JustifyItems::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            UiTransform::default(),
            GlobalZIndex(19),
            Visibility::Hidden,
            Part::Count,
        ))
        .with_children(|count| {
            let layer = |color: Color| {
                (
                    Text::new(""),
                    TextFont {
                        font_size: FontSize::Vh(look.count_size),
                        ..default()
                    },
                    TextColor(color),
                    TextLayout::justify(Justify::Center),
                    outlined,
                    Node {
                        grid_row: GridPlacement::start(1),
                        grid_column: GridPlacement::start(1),
                        ..default()
                    },
                )
            };
            count.spawn((
                layer(Color::srgb_u8(shadow[0], shadow[1], shadow[2])),
                UiTransform::from_scale(Vec2::new(1.0, 1.1)),
                Part::CountBack,
            ));
            count.spawn((layer(GOLD), Part::CountFace));
        });
}

fn difficulty_index(d: Difficulty) -> usize {
    match d {
        Difficulty::Easy => 0,
        Difficulty::Normal => 1,
        Difficulty::Pro => 2,
    }
}

/// Starts each battle's intro, keeps its clock (stopped while paused or behind
/// the explanation popup), plays its sounds, and hands over to play.
#[allow(clippy::too_many_arguments)]
fn run_intro(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    cup: Res<Cup>,
    art: Option<Res<Art>>,
    mut game: ResMut<Match>,
    mut intro: ResMut<Intro>,
    mut lift: ResMut<CameraLift>,
    mut count: ResMut<IntroCount>,
    mut laser: ResMut<LaserIn>,
    mut eclipse: ResMut<EclipseIn>,
    mut clock: ResMut<IntroClock>,
) {
    let look = &config.intro;
    if game.round != intro.round {
        intro.round = game.round;
        intro.clock = 0.0;
        intro.plan = None;
        if game.phase == Phase::Intro || game.phase == Phase::Menu {
            let mode = game.mode.rules();
            intro.overtime = cup.overtime(&config.cups);
            let title = cup.size.is_none() || cup.show_title || intro.overtime;
            let single = config.trial.is_some() || game.mode.is_endless();
            intro.plan = Some(if single {
                look.plan_single(game.mode, game.retry, config.endless.intro_seconds)
            } else {
                look.plan(mode, title, game.retry, config.survival.bricks_to_place)
            });
            intro.difficulty = difficulty_index(config.difficulty.unwrap_or(Difficulty::Normal));
            // RaceGameModeCountDownController (Race battles and trials, Endless Race) ends
            // on the RACE word.
            intro.race = matches!(game.mode, ModeKind::Race | ModeKind::Endless);
        }
    }
    let Some(plan) = intro.plan else {
        lift.0 = 0.0;
        count.0 = None;
        laser.0 = None;
        eclipse.0 = None;
        clock.0 = None;
        return;
    };
    let before = intro.clock;
    let running = game.phase == Phase::Intro
        || (game.phase == Phase::Playing && before < plan.countdown_at + look.count_end);
    if running {
        intro.clock += time.delta_secs();
    }
    let now = intro.clock;
    let crossed = |at: f32| before < at && now >= at;
    let sound = |commands: &mut Commands, name: &str, speed: f32| {
        if let Some(handle) = art.as_deref().and_then(|a| a.sounds.get(name)) {
            commands.spawn((
                AudioPlayer(handle.clone()),
                PlaybackSettings::DESPAWN.with_speed(speed),
            ));
        }
    };
    if plan.title_at.is_some_and(crossed) {
        sound(&mut commands, "sfx_countdown", 1.0);
    }
    for (k, pitch) in look.count_pitch.iter().enumerate().filter(|_| plan.count) {
        if crossed(plan.countdown_at + look.count_step * k as f32) {
            sound(&mut commands, "sfx_count_v3", *pitch);
        }
    }
    laser.0 = plan
        .laser_at
        .map(|at| now - at)
        .filter(|&t| t < look.laser_sweep + look.laser_fade);
    if plan.laser_at.is_some_and(crossed) {
        sound(&mut commands, "sfx_spawn_laser", 1.0);
    }
    eclipse.0 = plan.eclipse_at.map(|at| now - at);
    clock.0 = Some(now);
    let go = if plan.count { look.count_go } else { 0.0 };
    if game.phase == Phase::Intro && now >= plan.countdown_at + go {
        game.phase = Phase::Playing;
    }
    // Race: up to the finish line (less `race_room`), a hold, back down; in-out quad.
    lift.0 = plan.sweep_at.map_or(0.0, |at| {
        let height = (game.finish_height - look.race_room).max(0.0);
        let t = now - at;
        let quad = |x: f32| {
            let x = x.clamp(0.0, 1.0);
            if x < 0.5 {
                2.0 * x * x
            } else {
                1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
            }
        };
        let (up, hold, down) = plan.sweep;
        if t < up + hold {
            height * quad(t / up.max(0.01))
        } else {
            height * (1.0 - quad((t - up - hold) / down.max(0.01)))
        }
    });
    // Survival: 0 until the count-up, then up to the bricks to place.
    count.0 = match plan.count_up {
        Some((at, long)) if game.phase == Phase::Intro && now < at + long => {
            if crossed(at) {
                let name = if long < look.survival_count {
                    "sfx_count_up_short"
                } else {
                    "sfx_count_up_long"
                };
                sound(&mut commands, name, 1.0);
            }
            let share = ((now - at) / long.max(0.01)).clamp(0.0, 1.0);
            Some((config.survival.bricks_to_place as f32 * share) as u32)
        }
        _ => None,
    };
}

/// The mode title: opens to full height, its width bouncing to size, stays, then
/// rises a little, drops and fades.
#[allow(clippy::type_complexity)]
fn show_title(
    config: Res<GameConfig>,
    game: Res<Match>,
    intro: Res<Intro>,
    mut bands: Query<(
        &Part,
        &mut Node,
        &mut UiTransform,
        &mut Visibility,
        &Children,
    )>,
    mut images: Query<&mut ImageNode>,
    kids: Query<&Children>,
    mut words: Query<(&Part, &mut Text, &mut TextColor, &mut crate::letters::AutoSize)>,
) {
    let look = &config.intro;
    let d = intro.difficulty;
    let t = intro
        .plan
        .and_then(|p| p.title_at)
        .map(|at| intro.clock - at);
    let out_at = look.title_in + look.title_hold;
    let on = t.is_some_and(|t| t >= 0.0 && t < out_at + look.title_out);
    let t = t.unwrap_or(0.0);
    let (from, to) = look.title_fade;
    let alpha = 1.0 - ((t - out_at - from) / (to - from).max(0.01)).clamp(0.0, 1.0);
    for (part, mut node, mut transform, mut visibility, children) in &mut bands {
        let Part::Band(band) = *part else {
            continue;
        };
        let show = on && band == d;
        visibility.set_if_neq(if show {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        if !show {
            continue;
        }
        let width = curve(&look.title_widths[d], t);
        let rise = if t >= out_at {
            curve(&look.title_out_y, t - out_at)
        } else {
            0.0
        };
        let centre = look.title_y[d] + rise;
        let (left, top, w) = (
            px(-width / 2.0),
            px(540.0 - centre - look.title_height / 2.0),
            px(width),
        );
        if node.margin.left != left || node.top != top || node.width != w {
            node.margin.left = left;
            node.top = top;
            node.width = w;
        }
        let open = (t / look.title_open.max(0.01)).clamp(0.0, 1.0);
        transform.scale = Vec2::new(1.0, open);
        // Pictures fade with the words.
        let mut stack: Vec<Entity> = children.iter().collect();
        while let Some(child) = stack.pop() {
            if let Ok(mut image) = images.get_mut(child)
                && image.color.alpha() != alpha
            {
                image.color.set_alpha(alpha);
            }
            if let Ok(more) = kids.get(child) {
                stack.extend(more.iter());
            }
        }
    }
    let mode = match game.mode.rules() {
        ModeKind::Race => "RACE",
        ModeKind::Survival => "SURVIVAL",
        _ => "PUZZLE",
    };
    let difficulty = ["EASY", "NORMAL", "SPECIAL"][d];
    let band_width = curve(&look.title_widths[d], t);
    for (part, mut text, mut color, mut fit) in &mut words {
        // Each ModeTitleMessage's Label and OvertimeLabel are narrower than the band by their
        // own amounts (Easy 200 and 200, Normal 400 and 500, Special 560 and 700 px).
        let (words_in, overtime_in) = TITLE_INSETS[d];
        let room = match *part {
            Part::Words(_) => band_width - words_in,
            _ => band_width - overtime_in,
        } / 10.8;
        if fit.width != room {
            fit.width = room.max(0.0);
        }
        let want = match *part {
            Part::Words(band) if band == d => format!("{mode} {difficulty}"),
            Part::Overtime(band) if band == d && intro.overtime => "OVERTIME".to_string(),
            Part::Words(_) | Part::Overtime(_) => String::new(),
            _ => continue,
        };
        if text.0 != want {
            text.0 = want;
        }
        if color.0.alpha() != alpha {
            color.0.set_alpha(alpha);
        }
    }
}

/// 3, 2, 1 a number every `count_step`, each popping in; Race shows its word as
/// play starts.
fn show_countdown(
    config: Res<GameConfig>,
    intro: Res<Intro>,
    mut root: Query<(&Part, &mut UiTransform, &mut Visibility), Without<Text>>,
    mut layers: Query<(&Part, &mut Text)>,
) {
    let look = &config.intro;
    let t = intro.plan.map(|p| intro.clock - p.countdown_at);
    let end = if intro.race {
        look.count_end
    } else {
        look.count_go
    };
    let on = intro.plan.is_some_and(|p| p.count) && t.is_some_and(|t| t >= 0.0 && t < end);
    let t = t.unwrap_or(0.0);
    let words = match (t / look.count_step.max(0.01)) as usize {
        0 => "3",
        1 => "2",
        2 => "1",
        _ => "RACE",
    };
    for (part, mut transform, mut visibility) in &mut root {
        if *part != Part::Count {
            continue;
        }
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
        transform.scale = Vec2::splat(curve(&look.count_scale, t));
        // Unity turns counter-clockwise for a positive z; UI rotation runs the other way.
        transform.rotation = Rot2::degrees(-keyed(&look.count_turn, t));
    }
    for (part, mut text) in &mut layers {
        if matches!(part, Part::CountBack | Part::CountFace) && text.0 != words {
            text.0 = words.to_string();
        }
    }
}
