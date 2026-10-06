//! Weather and sky changes by mode, from the PC readouts (docs/research/modes.md "Race
//! Pro look", docs/research/gap-scan.md 1-2):
//! - Race Pro ("Special"): rain from the top, slanting with the wind, front streaks and a
//!   dense darker back layer, and lightning: a full-screen flash and a bolt with branches.
//! - Every Puzzle: a sand strip along the bottom and a two-layer sandstorm scrolling
//!   across (the back one behind the play, the front one over it).
//! - Puzzle Pro: the eclipsed sun with its corona rays.
//! - Survival: the night sky shifts to the curse-wave sky over 1 s while a wave lasts.
//!
//! The Race Pro and Puzzle Pro sky gradients are set up in `visuals::spawn_backgrounds`.
//! Numbers live in `GameConfig::weather`. Looks only.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{Match, Phase, TowerState};
use crate::visuals::{Sky, ViewSize, gradient_image, pro_look};

pub struct WeatherPlugin;

impl Plugin for WeatherPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<WeatherState>()
            .init_resource::<Flash>()
            .init_resource::<WaveMix>()
            .add_systems(
                Update,
                (
                    set_up_weather,
                    rain,
                    lightning,
                    fit_weather,
                    survival_wave_sky,
                )
                    .chain(),
            );
    }
}

/// Everything this module spawns, so a new look clears the old one.
#[derive(Component)]
struct Weather;

/// How bright the lightning flash is now, 0 to 1, for layers lit by it.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct Flash(pub f32);

/// How far Survival's background has shifted to the curse-wave look, 0 to 1.
#[derive(Resource, Default, Debug, Clone, Copy)]
pub struct WaveMix(pub f32);

/// The full-screen flash of a strike (not a bolt piece).
#[derive(Component)]
struct FlashStrike;

#[derive(Component)]
struct RainDrop {
    velocity: Vec2,
    life: f32,
}

/// A lightning flash (full screen) or bolt piece: seconds since the strike, and how it
/// fades.
#[derive(Component)]
struct Strike {
    time: f32,
    hold: f32,
    fade: f32,
    alpha: f32,
}

/// A scrolling sandstorm copy: front or back layer, layer number, and which of the two
/// side-by-side copies.
#[derive(Component)]
struct SandLayer {
    front: bool,
    layer: u32,
    copy: u32,
}

/// A Puzzle sand strip along the screen bottom; `front` is the one before the front ruins.
#[derive(Component)]
struct SandStrip {
    front: bool,
}

#[derive(Component)]
struct EclipseSun;

/// One corona ray around the eclipsed sun: its angle and stretch.
#[derive(Component)]
struct CoronaRay {
    angle: f32,
    stretch: Vec2,
}

#[derive(Resource, Default)]
struct WeatherState {
    look: Option<(ModeKind, Option<&'static str>, bool)>,
    rain_debt: (f32, f32),
    next_strike: f32,
    seed: u32,
    /// 0 = Survival night sky, 1 = the curse-wave sky.
    wave_mix: f32,
    wave_drawn: f32,
}

fn roll(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 10_000) as f32 / 10_000.0
}

fn rgba(c: [u8; 3], a: f32) -> Color {
    Color::srgba_u8(c[0], c[1], c[2], (a * 255.0) as u8)
}

/// The camera's position and the size of what it sees, in world units.
fn view(camera: &Query<(&Transform, &Projection), With<Camera2d>>, size: Vec2) -> (Vec2, Vec2) {
    match camera.single() {
        Ok((transform, Projection::Orthographic(ortho))) => {
            (transform.translation.truncate(), size * ortho.scale)
        }
        _ => (Vec2::ZERO, size),
    }
}

#[allow(clippy::too_many_arguments)]
fn set_up_weather(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    mut state: ResMut<WeatherState>,
    camera: Query<Entity, With<Camera2d>>,
    old: Query<Entity, With<Weather>>,
) {
    let in_match = game.phase != Phase::Menu;
    let look = (
        game.mode.rules(),
        pro_look(game.mode, config.difficulty),
        in_match,
    );
    if state.look == Some(look) && !art.is_changed() {
        return;
    }
    state.look = Some(look);
    state.seed = (game.round as u32).wrapping_mul(2_654_435_761) | 1;
    state.next_strike = config.weather.lightning_quiet_start;
    for entity in &old {
        commands.entity(entity).despawn();
    }
    let Ok(camera) = camera.single() else {
        return;
    };
    if !in_match {
        return;
    }
    let weather = &config.weather;
    if look.0 == ModeKind::Puzzle {
        // Back strip between the middle and front ruins (screen layers sit at -900 + z),
        // front strip just in front of the front ruins.
        for (front, name, z) in [
            (false, "puzzle_sand", -895.5),
            (true, "puzzle_sand_front", -893.5),
        ] {
            if let Some(sand) = art.image(name) {
                let alpha = if front { 1.0 } else { weather.sand_alpha };
                commands.entity(camera).with_child((
                    Weather,
                    SandStrip { front },
                    Sprite {
                        image: sand,
                        color: Color::WHITE.with_alpha(alpha),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, z),
                ));
            }
        }
        for (front, names, (tint, alpha), z) in [
            (
                false,
                ["sandstorm_back_1", "sandstorm_back_2"],
                weather.sandstorm_back,
                -600.0,
            ),
            (
                true,
                ["sandstorm_front_1", "sandstorm_front_2"],
                weather.sandstorm_front,
                400.0,
            ),
        ] {
            for (layer, name) in names.iter().enumerate() {
                let Some(image) = art.image(name) else {
                    continue;
                };
                for copy in 0..2 {
                    commands.entity(camera).with_child((
                        Weather,
                        SandLayer {
                            front,
                            layer: layer as u32,
                            copy,
                        },
                        Sprite {
                            image: image.clone(),
                            color: rgba(tint, alpha),
                            ..default()
                        },
                        Transform::from_xyz(0.0, 0.0, z + layer as f32),
                    ));
                }
            }
        }
    }
    if matches!(look.1, Some("puzzle_pro" | "endless_puzzle")) {
        if let Some(sun) = art.image("puzzle_pro_sun") {
            commands.entity(camera).with_child((
                Weather,
                EclipseSun,
                Sprite::from_image(sun),
                Transform::from_xyz(0.0, 0.0, -930.0),
            ));
        }
        if let Some(ray) = art.image("puzzle_pro_corona") {
            let mut seed = state.seed;
            for i in 0..weather.corona_rays {
                let (w, h) = (weather.corona_width, weather.corona_height);
                let stretch = Vec2::new(
                    w.0 + (w.1 - w.0) * roll(&mut seed),
                    h.0 + (h.1 - h.0) * roll(&mut seed),
                );
                commands.entity(camera).with_child((
                    Weather,
                    CoronaRay {
                        angle: i as f32 / weather.corona_rays.max(1) as f32 * std::f32::consts::TAU,
                        stretch,
                    },
                    Sprite::from_image(ray.clone()),
                    Transform::from_xyz(0.0, 0.0, -935.0),
                ));
            }
            state.seed = seed;
        }
    }
}

/// Race Pro rain: drops fall from above the view, slanting with the wind.
#[allow(clippy::too_many_arguments)]
fn rain(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    view_size: Res<ViewSize>,
    mut state: ResMut<WeatherState>,
    camera: Query<(&Transform, &Projection), With<Camera2d>>,
    mut drops: Query<(Entity, &mut RainDrop, &mut Transform), Without<Camera2d>>,
) {
    let dt = time.delta_secs();
    for (entity, mut drop, mut transform) in &mut drops {
        drop.life -= dt;
        if drop.life <= 0.0 {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += (drop.velocity * dt).extend(0.0);
    }
    let raining = state.look.is_some_and(|l| l.1 == Some("race_pro") && l.2);
    if !raining || game.phase == Phase::Menu {
        return;
    }
    let weather = &config.weather;
    let c = config.cell_size;
    let (centre, size) = view(&camera, view_size.0);
    // The rain slants away from the side the wind blows from.
    let side = if game.wind < 0.0 { -1.0 } else { 1.0 };
    let tilt = weather.rain_tilt.to_radians() * side;
    let dir = Vec2::new(tilt.sin(), -tilt.cos());
    let image = art.image("rain_drop");
    let mut seed = state.seed;
    for (front, (rate, life, speed, length, colour)) in
        [(true, weather.rain_front), (false, weather.rain_back)]
    {
        let debt = if front {
            &mut state.rain_debt.0
        } else {
            &mut state.rain_debt.1
        };
        *debt += rate * dt;
        while *debt >= 1.0 {
            *debt -= 1.0;
            // From a line above the view, wide enough that slanted drops still cover it.
            let reach = size.x * 0.5 + size.y * tilt.abs().tan() + c * 4.0;
            let x = centre.x - side * size.y * 0.5 * tilt.tan().abs()
                + (roll(&mut seed) * 2.0 - 1.0) * reach;
            let y = centre.y + size.y * 0.5 + c * (2.0 + roll(&mut seed) * 6.0);
            let sprite = match &image {
                Some(image) => Sprite {
                    image: image.clone(),
                    color: Color::srgba(colour[0], colour[1], colour[2], colour[3]),
                    custom_size: Some(Vec2::new(
                        c * 0.12 * length.max(1.0) / 3.0 + c * 0.05,
                        c * length,
                    )),
                    ..default()
                },
                None => Sprite::from_color(
                    Color::srgba(colour[0], colour[1], colour[2], colour[3] * 0.6),
                    Vec2::new(c * 0.06, c * length),
                ),
            };
            commands.spawn((
                Weather,
                RainDrop {
                    velocity: dir * speed * c,
                    life,
                },
                sprite,
                Transform::from_xyz(x, y, if front { 25.0 } else { -500.0 })
                    .with_rotation(Quat::from_rotation_z(tilt)),
            ));
        }
    }
    state.seed = seed;
}

/// Race Pro lightning: a white flash over the screen and a bolt top to bottom with a few
/// branches, every 5 to 14 s.
#[allow(clippy::too_many_arguments)]
fn lightning(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    view_size: Res<ViewSize>,
    mut state: ResMut<WeatherState>,
    mut flash: ResMut<Flash>,
    camera: Query<(Entity, &Transform, &Projection), With<Camera2d>>,
    mut strikes: Query<(Entity, &mut Strike, &mut Sprite, Has<FlashStrike>)>,
) {
    let dt = time.delta_secs();
    let mut lit = 0.0f32;
    for (entity, mut strike, mut sprite, is_flash) in &mut strikes {
        strike.time += dt;
        let fading = (strike.time - strike.hold).max(0.0) / strike.fade.max(0.001);
        if fading >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        sprite.color.set_alpha(strike.alpha * (1.0 - fading));
        if is_flash {
            lit = lit.max(1.0 - fading);
        }
    }
    flash.0 = lit;
    let stormy = state.look.is_some_and(|l| l.1 == Some("race_pro") && l.2);
    if !stormy || game.phase != Phase::Playing {
        return;
    }
    state.next_strike -= dt;
    if state.next_strike > 0.0 {
        return;
    }
    let weather = &config.weather;
    let mut seed = state.seed;
    let (lo, hi) = weather.lightning_every;
    state.next_strike = lo + (hi - lo) * roll(&mut seed);
    let Ok((camera, transform, Projection::Orthographic(ortho))) = camera.single() else {
        state.seed = seed;
        return;
    };
    let size = view_size.0 * ortho.scale;
    let c = config.cell_size;
    let (flash_alpha, flash_fade) = weather.lightning_flash;
    commands.entity(camera).with_child((
        Weather,
        FlashStrike,
        Strike {
            time: 0.0,
            hold: 0.0,
            fade: flash_fade,
            alpha: flash_alpha,
        },
        Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, flash_alpha), size),
        // Behind the islands, like the original's flash on its background landscape layer
        // (screen layers sit at -900 + z; the clouds are 0 and 1).
        Transform::from_xyz(0.0, 0.0, -898.5),
    ));
    // The bolt: from a random point on the top edge to the bottom edge, shifted up to
    // ±5/48 of the width, jagged, with a few shorter branches.
    let centre = transform.translation.truncate();
    let top = Vec2::new(
        centre.x + (roll(&mut seed) - 0.5) * size.x,
        centre.y + size.y * 0.5,
    );
    let bottom = Vec2::new(
        top.x + (roll(&mut seed) * 2.0 - 1.0) * size.x * 5.0 / 48.0,
        centre.y - size.y * 0.5,
    );
    let (hold, fade) = weather.lightning_bolt;
    let image = art.image("lightning_thick");
    let segment = |commands: &mut Commands, a: Vec2, b: Vec2, width: f32| {
        let along = b - a;
        let sprite = match &image {
            Some(image) => Sprite {
                image: image.clone(),
                custom_size: Some(Vec2::new(along.length() + 2.0, width)),
                ..default()
            },
            None => Sprite::from_color(Color::WHITE, Vec2::new(along.length() + 2.0, width * 0.4)),
        };
        commands.spawn((
            Weather,
            Strike {
                time: 0.0,
                hold,
                fade,
                alpha: 1.0,
            },
            sprite,
            Transform::from_translation(((a + b) * 0.5).extend(-898.2))
                .with_rotation(Quat::from_rotation_z(along.to_angle())),
        ));
    };
    let pieces = 14;
    let mut points = vec![top];
    for i in 1..pieces {
        let base = top.lerp(bottom, i as f32 / pieces as f32);
        points.push(base + Vec2::X * (roll(&mut seed) - 0.5) * c * 3.0);
    }
    points.push(bottom);
    for pair in points.windows(2) {
        segment(&mut commands, pair[0], pair[1], c * 0.8);
    }
    let branches = (roll(&mut seed) * (weather.lightning_branches + 1) as f32) as usize;
    for _ in 0..branches {
        let from = points[1 + (roll(&mut seed) * (points.len() - 2) as f32) as usize];
        let mut at = from;
        let lean = if roll(&mut seed) < 0.5 { -1.0 } else { 1.0 };
        for _ in 0..4 {
            let next = at
                + Vec2::new(
                    lean * c * (1.0 + roll(&mut seed) * 2.0),
                    -c * (1.5 + roll(&mut seed) * 2.0),
                );
            segment(&mut commands, at, next, c * 0.4);
            at = next;
        }
    }
    state.seed = seed;
}

/// Keeps the screen-fixed pieces sized to the view and scrolls the sandstorm.
#[allow(clippy::type_complexity)]
fn fit_weather(
    time: Res<Time>,
    config: Res<GameConfig>,
    view_size: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), With<Camera2d>>,
    mut parts: ParamSet<(
        Query<(&SandStrip, &mut Sprite, &mut Transform), Without<Camera2d>>,
        Query<(&SandLayer, &mut Sprite, &mut Transform), Without<Camera2d>>,
        Query<(&mut Sprite, &mut Transform), (With<EclipseSun>, Without<Camera2d>)>,
        Query<(&CoronaRay, &mut Sprite, &mut Transform), Without<Camera2d>>,
    )>,
) {
    let weather = &config.weather;
    let (_, size) = view(&camera, view_size.0);
    let t = time.elapsed_secs();
    for (strip, mut sprite, mut transform) in &mut parts.p0() {
        let (h, lift) = if strip.front {
            (weather.sand_front, 0.0)
        } else {
            (weather.sand_strip, weather.sand_lift)
        };
        let h = size.y * h;
        sprite.custom_size = Some(Vec2::new(size.x, h));
        transform.translation.y = -size.y * (0.5 - lift) + h * 0.5;
    }
    for (layer, mut sprite, mut transform) in &mut parts.p1() {
        sprite.custom_size = Some(size);
        // Two copies side by side slide across and wrap.
        let speed = weather.sandstorm_speed * if layer.front { 1.0 } else { 0.7 };
        let shift = (t * speed + layer.layer as f32 * weather.sandstorm_offset).fract();
        transform.translation.x = (layer.copy as f32 - shift) * size.x;
    }
    let sun_size = size.y * weather.eclipse_size;
    let sun_y = size.y * weather.eclipse_height;
    for (mut sprite, mut transform) in &mut parts.p2() {
        sprite.custom_size = Some(Vec2::splat(sun_size));
        transform.translation.y = sun_y;
    }
    for (ray, mut sprite, mut transform) in &mut parts.p3() {
        let len = sun_size * 0.5 * ray.stretch.y;
        sprite.custom_size = Some(Vec2::new(sun_size * 0.25 * ray.stretch.x, len));
        let out = Vec2::from_angle(ray.angle + t * 0.05);
        transform.translation.x = out.x * (sun_size * 0.5 + len * 0.5);
        transform.translation.y = sun_y + out.y * (sun_size * 0.5 + len * 0.5);
        transform.rotation = Quat::from_rotation_z(out.to_angle() - std::f32::consts::FRAC_PI_2);
    }
}

/// Survival: while a curse wave lasts, the sky shifts to the wave colours over 1 s, and
/// back when it ends.
#[allow(clippy::too_many_arguments)]
fn survival_wave_sky(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut state: ResMut<WeatherState>,
    mut mix: ResMut<WaveMix>,
    towers: Query<&TowerState>,
    sky: Query<&Sprite, With<Sky>>,
    mut images: ResMut<Assets<Image>>,
) {
    if game.mode.rules() != ModeKind::Survival || game.phase == Phase::Menu {
        state.wave_mix = 0.0;
        mix.0 = 0.0;
        return;
    }
    let weather = &config.weather;
    // Timed waves (Survival Pro) and waves counted in bricks (Endless Survival).
    let wave = towers
        .iter()
        .any(|s| s.wave_left > 0.0 || s.wave_bricks > 0);
    let step = time.delta_secs() / weather.wave_shift.max(0.01);
    state.wave_mix = if wave {
        (state.wave_mix + step).min(1.0)
    } else {
        (state.wave_mix - step).max(0.0)
    };
    mix.0 = state.wave_mix;
    // Redraw only while the colours are moving.
    if state.wave_mix == state.wave_drawn {
        return;
    }
    state.wave_drawn = state.wave_mix;
    let colors: Vec<Color> = weather
        .survival_sky
        .iter()
        .zip(&weather.survival_wave_sky)
        .map(|(&[r, g, b], &[wr, wg, wb])| {
            Color::srgb_u8(r, g, b).mix(&Color::srgb_u8(wr, wg, wb), state.wave_mix)
        })
        .collect();
    if colors.is_empty() {
        return;
    }
    for sprite in &sky {
        if let Some(mut image) = images.get_mut(&sprite.image) {
            *image = gradient_image(&colors);
        }
    }
}
