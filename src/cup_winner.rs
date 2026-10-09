//! The cup winner's popup (the original's TournamentWinnerPopup), shown when a cup is
//! decided, after the standings panel: "<CUP> WINNER!" with the cup's decorations, the
//! winner's name, how many medals of each place they took (counted up 4th, 3rd, 2nd,
//! 1st, with rising beeps), then the cup's trophy pops in over turning rays, the drum
//! roll gives way to the win music, and the winner's wizard appears with confetti and
//! fireworks. 12 s after it opened it fades to black over 3 s and Select Mode follows.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::cups::Cup;
use crate::gameplay::{Match, Phase, Tower, TowerSkin};
use crate::letters::TextLook;
use crate::menu::MenuScreen;
use crate::popup::Popup;
use crate::select::{SCREEN, label_with, place, seat_picture, sliced};

pub struct CupWinnerPlugin;

impl Plugin for CupWinnerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CupWinner>()
            .add_systems(Startup, spawn_winner)
            .add_systems(Update, (show_winner, finish_winner));
    }
}

/// The popup's panel (PlayerInfo): 950 x 580, its middle 75 px above the screen's.
const PANEL: ((f32, f32), (f32, f32)) = ((950.0, 580.0), (0.0, 75.0));
/// The name panel (PlayerPanel): 800 x 125, its top 100 px under the panel's, 9-sliced
/// 142 / 152 px at the sides.
const NAME_PANEL: (f32, f32) = (800.0, 125.0);
const NAME_SLICE: [f32; 4] = [142.0, 152.0, 0.0, 0.0];
/// The four medal counters' middles from the name panel's bottom-left (Fourth, Bronze,
/// Silver, Gold), 35 px up; each a 100 x 50 box with the medal at its right and the
/// number to its left.
const COUNTERS: [f32; 4] = [215.0, 340.0, 460.0, 585.0];
/// The medal pictures and their sizes, in the same order.
const COUNTER_MEDALS: [(&str, (f32, f32)); 4] = [
    ("menu_online_score_medal_last", (48.0, 48.0)),
    ("menu_leaderboard_medal_3", (44.0, 54.0)),
    ("menu_leaderboard_medal_2", (40.0, 49.0)),
    ("menu_leaderboard_medal_1", (44.0, 52.0)),
];
/// The trophy's background (475 x 195, its bottom 10 px over the panel's and 7 px left
/// of the middle), and each cup's trophy: picture size, its middle's offset from the
/// container's bottom middle, and the rays behind it.
const TROPHY_BACK: ((f32, f32), (f32, f32)) = ((475.0, 195.0), (-7.0, 10.0));
const TROPHIES: [((f32, f32), f32, f32); 3] = [
    ((111.0, 132.0), 50.0 - 17.33, 200.0),
    ((203.0, 203.0), 50.0, 300.0),
    ((307.0, 247.0), 50.0, 350.0),
];
/// Where the wizard appears (WizardSpawnPosition), from the screen's middle.
const WIZARD_AT: (f32, f32) = (650.0, -380.0);
const WIZARD_SIZE: (f32, f32) = (200.0, 180.0);

/// Timings (TournamentWinnerPopup, AnimateMedalAmountEffect, MoveInTrophyEffect).
const OPEN: f32 = 0.4;
const FIRST_DELAY: f32 = 0.5;
const COUNT_SCALE: f32 = 0.05;
const COUNT_TIME: f32 = 0.3;
const COUNT_FINISH: f32 = 0.089;
const TROPHY_DELAY: f32 = 0.3;
const TROPHY_TIME: f32 = 0.6;
const WIZARD_DELAY: f32 = 0.92;
const AUTO_CLOSE: f32 = 12.0;
const FADE_OUT: f32 = 3.0;
/// Each counter's first beep pitch (4th, 3rd, 2nd, 1st); it rises by 0.5 over the count.
const PITCH: [f32; 4] = [0.8, 1.0, 1.5, 2.0];

#[derive(Component, Clone, Copy, PartialEq)]
enum Part {
    Root,
    Panel,
    Title,
    Deco(bool),
    Name,
    Counter(usize),
    Count(usize),
    TrophyRays(usize),
    Trophy,
    Wizard,
    Confetti(usize),
    Spark(usize),
    Fade,
}

/// The popup's clock and what it has done, so each sound plays once.
#[derive(Resource, Default)]
pub struct CupWinner {
    /// Seconds since it opened, while it's up.
    pub time: Option<f32>,
    beeps: [usize; 4],
    drum: Option<Entity>,
    music: Option<Entity>,
    trophy_sound: bool,
}

impl CupWinner {
    pub fn up(&self) -> bool {
        self.time.is_some()
    }
}

/// When each counter starts and how long it takes, by the medal amounts (4th first).
fn counter_times(amounts: [usize; 4]) -> ([f32; 4], f32) {
    let mut at = OPEN + FIRST_DELAY;
    let mut starts = [0.0; 4];
    for (k, &amount) in amounts.iter().enumerate() {
        starts[k] = at;
        if amount > 0 {
            let step = COUNT_TIME / amount as f32;
            at += COUNT_SCALE + step * (amount + 1) as f32 + COUNT_FINISH + COUNT_SCALE;
        }
    }
    (starts, at)
}

fn spawn_winner(mut commands: Commands, config: Res<GameConfig>, art: Option<Res<Art>>) {
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let p = &config.cup_panel;
    let (size, at) = PANEL;
    let top = at.1 + size.1 / 2.0;
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            BackgroundColor(
                Color::srgb_u8(p.dim_colour.0, p.dim_colour.1, p.dim_colour.2).with_alpha(p.dim),
            ),
            GlobalZIndex(24),
            Visibility::Hidden,
            Part::Root,
        ))
        .with_children(|root| {
            let mut panel = root.spawn((
                place(size, at),
                BackgroundColor(Color::srgb(0.86, 0.89, 0.99)),
                Part::Panel,
            ));
            if let Some(art) = image("menu_leaderboard_panel") {
                panel.insert((
                    sliced(art, [0.0, 0.0, 37.0, 51.0]),
                    BackgroundColor(Color::NONE),
                ));
            }
            // The title straddles the bar, 14.81 px above the panel's top.
            root.spawn(place(
                (size.0 - 200.0, 60.0),
                (at.0, top + 14.81 - p.title_lift),
            ))
            .with_children(|t| {
                t.spawn((
                    label_with(
                        p.title_size,
                        "",
                        TextLook {
                            face: Some(Color::srgb_u8(
                                p.title_colour.0,
                                p.title_colour.1,
                                p.title_colour.2,
                            )),
                            ..TextLook::OUTLINED
                        },
                    ),
                    Part::Title,
                ));
            });
            for right in [false, true] {
                root.spawn((
                    place((0.0, 0.0), (0.0, 0.0)),
                    ImageNode::default(),
                    Part::Deco(right),
                ));
            }
            let name_top = top - 100.0;
            let name_mid = name_top - NAME_PANEL.1 / 2.0;
            let mut name = root.spawn(place(NAME_PANEL, (at.0, name_mid)));
            if let Some(art) = image("menu_cup_mode_winner_popup_name_background") {
                name.insert(sliced(art, NAME_SLICE));
            }
            // The winner's name hangs 10 px from the name panel's top.
            root.spawn(place(
                (NAME_PANEL.0 - 100.0, 40.0),
                (at.0, name_top - 10.0 - 20.0),
            ))
            .with_children(|n| {
                n.spawn((label_with(3.0, "", TextLook::BODY), Part::Name));
            });
            let bottom = name_top - NAME_PANEL.1;
            for (k, x) in COUNTERS.iter().enumerate() {
                let centre = (at.0 - NAME_PANEL.0 / 2.0 + x, bottom + 35.0);
                let (medal, medal_size) = COUNTER_MEDALS[k];
                root.spawn((
                    place((100.0, 50.0), centre),
                    UiTransform::default(),
                    Part::Counter(k),
                ))
                .with_children(|c| {
                    let mut pic = c.spawn(Node {
                        position_type: PositionType::Absolute,
                        right: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                        width: Val::Px(medal_size.0),
                        height: Val::Px(medal_size.1),
                        ..default()
                    });
                    if let Some(art) = image(medal) {
                        pic.insert(ImageNode::new(art));
                    }
                    c.spawn(Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(0.0),
                        top: Val::Px(0.0),
                        width: Val::Px(50.0),
                        height: Val::Px(50.0),
                        justify_content: JustifyContent::Center,
                        align_items: AlignItems::Center,
                        ..default()
                    })
                    .with_children(|l| {
                        l.spawn((label_with(3.4, "0", TextLook::BODY), Part::Count(k)));
                    });
                });
            }
            let (back_size, back_at) = TROPHY_BACK;
            let panel_bottom = at.1 - size.1 / 2.0;
            let back_mid = (
                at.0 + back_at.0,
                panel_bottom + back_at.1 + back_size.1 / 2.0,
            );
            let mut back = root.spawn(place(back_size, back_mid));
            if let Some(art) = image("menu_cup_mode_winner_popup_trophy_background") {
                back.insert(ImageNode::new(art));
            }
            for layer in 0..2 {
                let mut rays = root.spawn((
                    place((0.0, 0.0), (0.0, 0.0)),
                    UiTransform::default(),
                    Visibility::Hidden,
                    Part::TrophyRays(layer),
                ));
                if let Some(art) = image("cast_sunrays") {
                    rays.insert(
                        ImageNode::new(art).with_color(Color::srgba(0.929, 0.969, 1.0, 0.235)),
                    );
                }
            }
            root.spawn((
                place((0.0, 0.0), (0.0, 0.0)),
                ImageNode::default(),
                UiTransform::default(),
                Visibility::Hidden,
                Part::Trophy,
            ));
            root.spawn((
                place(WIZARD_SIZE, WIZARD_AT),
                ImageNode::default(),
                UiTransform::default(),
                Visibility::Hidden,
                Part::Wizard,
            ));
            // ponytail: confetti and fireworks are simple shapes; the original's particle
            // systems (CONFETTI_1/2, Firework) aren't read yet.
            for k in 0..60 {
                root.spawn((
                    place((10.0, 16.0), (0.0, 0.0)),
                    BackgroundColor(Color::NONE),
                    UiTransform::default(),
                    Part::Confetti(k),
                ));
            }
            for k in 0..48 {
                root.spawn((
                    Node {
                        border_radius: BorderRadius::MAX,
                        ..place((8.0, 8.0), (0.0, 0.0))
                    },
                    BackgroundColor(Color::NONE),
                    Part::Spark(k),
                ));
            }
            root.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    ..default()
                },
                BackgroundColor(Color::NONE),
                Part::Fade,
            ));
        });
}

fn hash(n: u32) -> f32 {
    let mut x = n.wrapping_mul(0x9E37_79B9) ^ 0x85EB_CA6B;
    x ^= x >> 15;
    x = x.wrapping_mul(0x2C1B_3C6D);
    x ^= x >> 12;
    (x % 10_000) as f32 / 10_000.0
}

fn out_elastic(x: f32) -> f32 {
    if x <= 0.0 {
        0.0
    } else if x >= 1.0 {
        1.0
    } else {
        2f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * std::f32::consts::TAU / 3.0).sin() + 1.0
    }
}

/// Opens the popup once the decided cup's standings panel has been up its while, and
/// runs its clock and sounds.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn show_winner(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    cup: Res<Cup>,
    popup: Res<Popup>,
    art: Option<Res<Art>>,
    mut winner: ResMut<CupWinner>,
    towers: Query<(&Tower, Option<&TowerSkin>)>,
    mut nodes: Query<(
        &Part,
        &mut Node,
        &mut Visibility,
        Option<&mut ImageNode>,
        Option<&mut BackgroundColor>,
        Option<&mut UiTransform>,
    )>,
    mut texts: Query<(&Part, &mut Text)>,
) {
    let p = &config.cup_panel;
    let decided = cup.size.is_some() && cup.standings.is_some();
    let due = game.phase == Phase::Over
        && popup.shown
        && decided
        && popup.since_shown() >= p.decided_hold;
    if !due {
        if winner.up() {
            for audio in [winner.drum.take(), winner.music.take()]
                .into_iter()
                .flatten()
            {
                commands.entity(audio).despawn();
            }
            *winner = CupWinner::default();
        }
        for (part, _, mut v, ..) in &mut nodes {
            if *part == Part::Root {
                v.set_if_neq(Visibility::Hidden);
            }
        }
        return;
    }
    let t = winner.time.map_or(0.0, |t| t + time.delta_secs());
    winner.time = Some(t);
    let image = |name: &str| art.as_ref().and_then(|a| a.image(name));
    let sound = |commands: &mut Commands, name: &str, settings: PlaybackSettings| {
        art.as_ref()
            .and_then(|a| a.sounds.get(name).cloned())
            .map(|s| commands.spawn((AudioPlayer(s), settings)).id())
    };
    let Some(&champion) = cup.standings.as_ref().and_then(|s| s.first()) else {
        return;
    };
    let size = cup.size.unwrap_or(0);
    // Medals of each place (4th, 3rd, 2nd, 1st): one per point, as on the standings.
    let medals = cup.medals.get(champion).cloned().unwrap_or_default();
    let fourths = cup
        .places
        .get(champion)
        .map_or(0, |v| v.iter().filter(|&&p| p >= 3).count());
    let amounts = [
        fourths,
        medals.iter().filter(|&&p| p == 2).count(),
        medals.iter().filter(|&&p| p == 1).count(),
        medals.iter().filter(|&&p| p == 0).count(),
    ];
    let (starts, counted) = counter_times(amounts);
    let trophy_at = counted + TROPHY_DELAY;
    let trophy_in = trophy_at + TROPHY_TIME;
    let wizard_at = trophy_in + WIZARD_DELAY;
    // Sounds: the drum roll from the start until the trophy is in, then the win music.
    if winner.drum.is_none() && t < trophy_in && winner.music.is_none() {
        winner.drum = sound(&mut commands, "sfx_drum_roll", PlaybackSettings::LOOP);
    }
    for k in 0..4 {
        let amount = amounts[k];
        if amount == 0 {
            continue;
        }
        let step = COUNT_TIME / amount as f32;
        let shown = (((t - starts[k] - COUNT_SCALE) / step).floor().max(0.0) as usize).min(amount);
        while winner.beeps[k] < shown {
            winner.beeps[k] += 1;
            let pitch = PITCH[k] + 0.5 / amount as f32 * (winner.beeps[k] - 1) as f32;
            sound(
                &mut commands,
                "sfx_countdown",
                PlaybackSettings::DESPAWN.with_speed(pitch),
            );
        }
    }
    if !winner.trophy_sound && t >= trophy_at {
        winner.trophy_sound = true;
        sound(&mut commands, "sfx_trophy", PlaybackSettings::DESPAWN);
    }
    if t >= trophy_in && winner.music.is_none() {
        if let Some(drum) = winner.drum.take() {
            commands.entity(drum).despawn();
        }
        winner.music = sound(&mut commands, "music_win_game_loop", PlaybackSettings::LOOP);
    }

    let (panel_size, panel_at) = PANEL;
    // Slides up from below as it opens, easing out.
    let open = (t / OPEN).clamp(0.0, 1.0);
    let rise = (1.0 - (1.0 - open) * (1.0 - open) - 1.0) * SCREEN.1;
    let closing = (t - OPEN - AUTO_CLOSE).max(0.0) / FADE_OUT;
    let deco = p.decos.get(size);
    let champion_tower = towers.iter().find(|(tower, _)| tower.index == champion);
    for (part, mut node, mut v, image_node, colour, transform) in &mut nodes {
        let show = |v: &mut Visibility, on: bool| {
            let want = if on { Visibility::Inherited } else { Visibility::Hidden };
            if *v != want {
                *v = want;
            }
        };
        match *part {
            Part::Root => {
                show(&mut v, true);
                node.top = Val::Percent(-rise / SCREEN.1 * 100.0);
            }
            Part::Deco(right) => {
                if let (Some(d), Some(mut img)) = (deco, image_node) {
                    let corner = if right {
                        panel_at.0 + panel_size.0 / 2.0
                    } else {
                        panel_at.0 - panel_size.0 / 2.0
                    };
                    let x = if right {
                        corner + d.cup_at.0
                    } else {
                        corner - d.cup_at.0
                    };
                    *node = place(
                        d.cup_size,
                        (x, panel_at.1 + panel_size.1 / 2.0 + d.cup_at.1),
                    );
                    if let Some(cup_art) = image(&d.cup) {
                        img.image = cup_art;
                    }
                    img.flip_x = !right;
                }
            }
            Part::Counter(k) => {
                // Grows to 1.3x while its number counts.
                let amount = amounts[k];
                let end = starts[k]
                    + if amount > 0 {
                        COUNT_SCALE
                            + COUNT_TIME / amount as f32 * (amount + 1) as f32
                            + COUNT_FINISH
                            + COUNT_SCALE
                    } else {
                        0.0
                    };
                let local = t - starts[k];
                let scale = if amount == 0 || local < 0.0 || t >= end {
                    1.0
                } else if local < COUNT_SCALE {
                    1.0 + 0.3 * local / COUNT_SCALE
                } else if t > end - COUNT_SCALE {
                    1.0 + 0.3 * (end - t) / COUNT_SCALE
                } else {
                    1.3
                };
                if let Some(mut tr) = transform {
                    tr.scale = Vec2::splat(scale);
                }
            }
            Part::TrophyRays(_) | Part::Trophy => {
                let layer = if let Part::TrophyRays(n) = *part { n } else { 0 };
                let on = t >= trophy_at;
                show(&mut v, on);
                let ((w, h), lift, rays) = TROPHIES[size.min(2)];
                let (back_size, back_at) = TROPHY_BACK;
                let base = (
                    panel_at.0 + back_at.0,
                    panel_at.1 - panel_size.1 / 2.0 + back_at.1,
                );
                let scale = 0.4 + 0.6 * out_elastic((t - trophy_at) / TROPHY_TIME);
                let _ = back_size;
                if *part == Part::Trophy {
                    *node = place((w, h), (base.0, base.1 + lift));
                    if let Some(mut img) = image_node
                        && let Some(cup_art) =
                            cup.rules(&config.cups).and_then(|s| image(&s.trophy))
                    {
                        img.image = cup_art;
                    }
                    if let Some(mut tr) = transform {
                        tr.scale = Vec2::splat(scale);
                    }
                } else {
                    *node = place((rays, rays), (base.0, base.1 + 50.0));
                    if let Some(mut tr) = transform {
                        tr.scale = Vec2::splat(scale);
                        // Sunrays01 turns -80 degrees a second, Sunrays02 45 (Big's stays).
                        let speed = if layer == 0 {
                            -80.0
                        } else if size == 1 {
                            0.0
                        } else {
                            45.0
                        };
                        tr.rotation = Rot2::degrees(speed * (t - trophy_at));
                    }
                }
            }
            Part::Wizard => {
                let on = t >= wizard_at;
                show(&mut v, on);
                if let (Some(mut img), Some((tower, skin))) = (image_node, champion_tower) {
                    let wizard = skin.map_or(tower.index, |s| s.wizard);
                    if let Some(pic) = image(&seat_picture(wizard, "ready")) {
                        img.image = pic;
                    }
                }
                // ponytail: the original lets the winner fly this wizard around; it bobs here.
                if let Some(mut tr) = transform {
                    let pop = ((t - wizard_at) / 0.2).clamp(0.0, 1.0);
                    tr.scale = Vec2::splat(pop);
                    tr.translation = Val2::px(0.0, (t * 2.0).sin() * 8.0);
                }
            }
            Part::Confetti(k) => {
                let since = t - wizard_at;
                let on = since >= 0.0;
                let fall = 260.0 + 160.0 * hash(k as u32);
                let x = (hash(k as u32 + 100) - 0.5) * SCREEN.0;
                let y = SCREEN.1 / 2.0 + 20.0
                    - ((since * fall + hash(k as u32 + 200) * SCREEN.1) % (SCREEN.1 + 40.0));
                *node = place((10.0, 16.0), (x + (since * 2.0 + k as f32).sin() * 30.0, y));
                if let Some(mut c) = colour {
                    let hues = [
                        (1.0, 0.3, 0.4),
                        (0.3, 0.8, 1.0),
                        (1.0, 0.85, 0.2),
                        (0.5, 1.0, 0.4),
                        (0.9, 0.4, 1.0),
                    ];
                    let (r, g, b) = hues[k % hues.len()];
                    c.0 = if on {
                        Color::srgb(r, g, b)
                    } else {
                        Color::NONE
                    };
                }
                if let Some(mut tr) = transform {
                    tr.rotation = Rot2::radians(since * (2.0 + hash(k as u32 + 300) * 4.0));
                }
            }
            Part::Spark(k) => {
                // A burst every 0.3 to 0.6 s: 16 sparks fly out and fade over 0.8 s.
                let since = t - wizard_at;
                let burst = k / 16;
                let period = 0.3 + 0.3 * hash(burst as u32 + 7);
                let local = if since >= 0.0 {
                    (since + burst as f32 * 0.4) % (period * 3.0)
                } else {
                    -1.0
                };
                let round = if since >= 0.0 {
                    ((since + burst as f32 * 0.4) / (period * 3.0)) as u32
                } else {
                    0
                };
                let centre = (
                    (hash(round * 31 + burst as u32) - 0.5) * SCREEN.0 * 0.8,
                    hash(round * 17 + burst as u32 + 5) * SCREEN.1 * 0.35,
                );
                let angle = (k % 16) as f32 / 16.0 * std::f32::consts::TAU;
                let life = local / 0.8;
                let out = 140.0 * (1.0 - (1.0 - life.clamp(0.0, 1.0)).powi(2));
                *node = Node {
                    border_radius: BorderRadius::MAX,
                    ..place(
                        (8.0, 8.0),
                        (centre.0 + angle.cos() * out, centre.1 + angle.sin() * out),
                    )
                };
                if let Some(mut c) = colour {
                    c.0 = if (0.0..1.0).contains(&life) {
                        Color::srgb(1.0, 0.9, 0.5).with_alpha(1.0 - life)
                    } else {
                        Color::NONE
                    };
                }
            }
            Part::Fade => {
                if let Some(mut c) = colour {
                    c.0 = Color::BLACK.with_alpha(closing.min(1.0));
                }
            }
            _ => {}
        }
    }
    for (part, mut text) in &mut texts {
        let words = match *part {
            Part::Title => cup
                .rules(&config.cups)
                .map(|s| format!("{} WINNER!", s.name.to_uppercase())),
            Part::Name => champion_tower.map(|(t, _)| match t.player {
                Some(n) => format!("PLAYER {}", n + 1),
                None => format!("WIZARD {}", t.index + 1),
            }),
            Part::Count(k) => {
                let amount = amounts[k];
                let shown = if amount == 0 {
                    0
                } else {
                    let step = COUNT_TIME / amount as f32;
                    (((t - starts[k] - COUNT_SCALE) / step).floor().max(0.0) as usize).min(amount)
                };
                Some(shown.to_string())
            }
            _ => None,
        };
        if let Some(words) = words
            && text.0 != words
        {
            text.0 = words;
        }
    }
}

/// After the fade, the cup is over: Select Mode, as the original's local cups.
fn finish_winner(
    mut commands: Commands,
    mut winner: ResMut<CupWinner>,
    mut game: ResMut<Match>,
    mut screen: ResMut<MenuScreen>,
    mut cup: ResMut<Cup>,
) {
    if winner
        .time
        .is_some_and(|t| t >= OPEN + AUTO_CLOSE + FADE_OUT)
    {
        for audio in [winner.drum.take(), winner.music.take()]
            .into_iter()
            .flatten()
        {
            commands.entity(audio).despawn();
        }
        *winner = CupWinner::default();
        cup.size = None;
        game.phase = Phase::Menu;
        *screen = MenuScreen::Modes;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counters_take_turns_and_skip_empty_places() {
        let (starts, end) = counter_times([0, 1, 2, 9]);
        assert_eq!(
            starts[0], starts[1],
            "no 4th places: the 3rd counts at once"
        );
        // One bronze: scale up, two steps of 0.3, the finish delay, scale down.
        let bronze = COUNT_SCALE + 0.3 * 2.0 + COUNT_FINISH + COUNT_SCALE;
        assert!((starts[2] - starts[1] - bronze).abs() < 1e-4);
        assert!(end > starts[3]);
    }
}
