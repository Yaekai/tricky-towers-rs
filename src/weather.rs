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
                    show_weather_in_menus,
                    rain,
                    lightning,
                    fit_weather,
                    run_endless_sun,
                    wave_mist_edges,
                    survival_wave_sky,
                )
                    .chain(),
            );
    }
}

/// Endless Puzzle's sun quad (DesertEndlessSun's SunGraphic: 20 x 20 units, 9.35 over the
/// view's middle).
#[derive(Component)]
struct EndlessSunQuad;

/// Sizes Survival's waving mist edges like the old edge sprites and speeds their wave up
/// with a curse wave (MistLayer.ChangeSpeed, eased with the sky).
fn wave_mist_edges(
    config: Res<GameConfig>,
    view_size: Res<ViewSize>,
    mix: Res<WaveMix>,
    camera: Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>,
    mut edges: Query<(&SurvivalMist, &MeshMaterial2d<crate::scene_material::FlagWave>, &mut Transform), Without<crate::visuals::MainCamera>>,
    mut waves: Option<ResMut<Assets<crate::scene_material::FlagWave>>>,
) {
    let _ = &config;
    let (_, size) = view(&camera, view_size.0);
    let unit = size.y / 41.54;
    let speed = MIST_SPEED.0 + (MIST_SPEED.1 - MIST_SPEED.0) * mix.0;
    for (mist, material, mut transform) in &mut edges {
        let top = -size.y * 0.5 + size.y * mist.top;
        let h = MIST_EDGE * mist.squash * unit;
        transform.scale = Vec3::new(size.x, h, 1.0);
        transform.translation.x = 0.0;
        transform.translation.y = top + h * 0.25;
        if let Some(mut wave) = waves.as_mut().and_then(|w| w.get_mut(&material.0))
            && wave.look.params.y != speed
        {
            wave.look.params.y = speed;
        }
    }
}

/// Places Endless Puzzle's sun and runs its eclipse: the amount and the gradient colour
/// ease out-expo over the eclipse's 2 s (SunEndlessLayer.Eclipse).
fn run_endless_sun(
    game: Res<Match>,
    intro: Option<Res<crate::intro::EclipseIn>>,
    view_size: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>,
    mut quads: Query<(&MeshMaterial2d<crate::scene_material::EndlessSun>, &mut Transform), (With<EndlessSunQuad>, Without<crate::visuals::MainCamera>)>,
    mut suns: Option<ResMut<Assets<crate::scene_material::EndlessSun>>>,
) {
    let (_, size) = view(&camera, view_size.0);
    let unit = size.y / 41.54;
    let t = crate::visuals::eclipse_progress(&game, intro.as_deref());
    let amount = if t >= 1.0 { 1.0 } else { 1.0 - 2f32.powf(-10.0 * t) };
    for (material, mut transform) in &mut quads {
        transform.translation.x = 0.0;
        transform.translation.y = 9.35 * unit;
        transform.scale = Vec3::new(20.0 * unit, 20.0 * unit, 1.0);
        if let Some(mut sun) = suns.as_mut().and_then(|s| s.get_mut(&material.0)) {
            sun.look = crate::scene_material::EndlessSunLook::at(amount);
        }
    }
}

/// Puts the weather on both render layers, so it shows behind the menus too.
fn show_weather_in_menus(
    mut commands: Commands,
    added: Query<Entity, (Added<Weather>, Without<bevy::camera::visibility::RenderLayers>)>,
) {
    for entity in &added {
        commands
            .entity(entity)
            .insert(bevy::camera::visibility::RenderLayers::from_layers(&[0, 1]));
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

/// A scrolling sandstorm copy: its band (`WeatherLook::sandstorm_bands`), the band's
/// place among its prefab's bands and how many there are, and which of the two
/// side-by-side copies.
#[derive(Component)]
struct SandLayer {
    band: usize,
    index: u32,
    count: u32,
    copy: u32,
}

/// A Puzzle sand strip along the screen bottom; `front` is the one before the front ruins.
#[derive(Component)]
struct SandStrip {
    front: bool,
}

/// One of Survival's three mist bands along the bottom (SurvivalBackground's
/// MistLayers): its top as a share of the view's height, and whether this is the
/// cloudy edge on top (squashed by `squash`) or the solid fill under it.
#[derive(Component)]
struct SurvivalMist {
    top: f32,
    edge: bool,
    squash: f32,
}

/// The bands: colour, top (14%, 19%, 24% of the view), edge squash, depth (in front
/// of the front islands, between the islands, behind them), and the edge wave's sideways
/// offset (MistLayer's xOffset).
const SURVIVAL_MISTS: [([u8; 3], f32, f32, f32, f32); 4] = [
    ([192, 163, 225], 0.14, 0.8, -895.5, 0.1),
    ([207, 163, 222], 0.19, 0.6, -897.5, 0.5),
    ([151, 125, 216], 0.24, 0.4, -899.55, 0.3),
    // MistForeground: its own camera over the towers, the bottom 10% of its view (0.08 of
    // ours matches Max's Survival clip, as ours draws its edge a little higher).
    // ponytail: it doesn't sink to 8 cells under the lowest tower when that is lower.
    ([227, 214, 246], 0.08, 1.0, 450.0, 0.0),
];
/// The mist edge's wave (MistWave, Custom/FlagWave): amplitude, offset up, and its speed
/// out of a curse wave and in one (SurvivalBackground: 1, then 5).
const MIST_WAVE: (f32, f32) = (0.05, 0.05);
const MIST_SPEED: (f32, f32) = (1.0, 5.0);
/// The mist edge's height in background units before its squash: gph_cursedLineMist is
/// 95.9 px at 26 px a unit, and the Mist prefab's Clouds child doubles it (y scale 2).
const MIST_EDGE: f32 = 7.38;

#[derive(Component)]
struct EclipseSun;

/// Puzzle Easy and Normal's plain sun (DesertBackgroundSun's SunLayer).
#[derive(Component)]
struct DesertSun;

/// One corona bar of the eclipsed sun: its place in `WeatherLook::corona`.
#[derive(Component)]
struct CoronaRay(usize);

#[derive(Resource, Default)]
struct WeatherState {
    look: Option<(ModeKind, Option<&'static str>, bool)>,
    /// How many pictures were staged when the weather was last built: it is built again
    /// once more have loaded (Art changes every frame, which kept clearing the rain).
    art_count: usize,
    rain_debt: (f32, f32),
    next_strike: f32,
    seed: u32,
    /// 0 = Survival night sky, 1 = the curse-wave sky.
    wave_mix: f32,
    wave_drawn: f32,
    /// Seconds since the last strike, for the islands' light (None before the first).
    since_strike: Option<f32>,
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
fn view(camera: &Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>, size: Vec2) -> (Vec2, Vec2) {
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
    menu_look: Option<Res<crate::visuals::MenuLook>>,
    menu_scene: Option<Res<crate::visuals::MenuScene>>,
    mut state: ResMut<WeatherState>,
    camera: Query<Entity, With<crate::visuals::MainCamera>>,
    old: Query<Entity, With<Weather>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut suns: Option<ResMut<Assets<crate::scene_material::EndlessSun>>>,
    mut waves: Option<ResMut<Assets<crate::scene_material::FlagWave>>>,
) {
    // Behind the menus the picked mode's weather shows with its background (Select Mode,
    // Endless); the plain Race scene has none. Over a match (and its pause) the match's.
    let behind_menus = menu_scene.is_some_and(|s| s.0);
    let picked = menu_look.and_then(|l| l.0).filter(|_| behind_menus);
    let in_match = !behind_menus || picked.is_some();
    let look = match picked {
        Some((mode, special)) => (
            mode.rules(),
            pro_look(mode, special.then_some(crate::config::Difficulty::Pro)),
            in_match,
        ),
        None => (
            game.mode.rules(),
            pro_look(game.mode, config.difficulty),
            in_match,
        ),
    };
    if state.look == Some(look) && state.art_count == art.images.len() {
        return;
    }
    state.look = Some(look);
    state.art_count = art.images.len();
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
    if look.0 == ModeKind::Survival {
        let edge = art.image("mist");
        let solid = art.image("misc_cursed_line_solid");
        for (color, top, squash, z, x_offset) in SURVIVAL_MISTS {
            let color = Color::srgb_u8(color[0], color[1], color[2]);
            // The edge waves along its top (a FlagWave quad, sized by `fit_weather`).
            if let (Some(image), Some(meshes), Some(waves)) = (edge.clone(), meshes.as_mut(), waves.as_mut()) {
                commands.entity(camera).with_child((
                    Weather,
                    SurvivalMist {
                        top,
                        edge: true,
                        squash,
                    },
                    Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                    MeshMaterial2d(waves.add(crate::scene_material::FlagWave {
                        look: crate::scene_material::WaterLook {
                            params: Vec4::new(MIST_WAVE.0, MIST_SPEED.0, MIST_WAVE.1, 1.0),
                            tint: Vec4::from_array(color.to_linear().to_f32_array()),
                            extra: Vec4::new(x_offset, 0.0, 0.0, 0.0),
                        },
                        picture: image,
                    })),
                    Transform::from_xyz(0.0, 0.0, z + 0.01),
                ));
            }
            for (is_edge, image) in [(false, solid.clone())] {
                let sprite = match image {
                    Some(image) => Sprite {
                        image,
                        color,
                        ..default()
                    },
                    None if !is_edge => Sprite::from_color(color, Vec2::ONE),
                    None => continue,
                };
                commands.entity(camera).with_child((
                    Weather,
                    SurvivalMist {
                        top,
                        edge: is_edge,
                        squash,
                    },
                    sprite,
                    Transform::from_xyz(0.0, 0.0, z + if is_edge { 0.01 } else { 0.0 }),
                ));
            }
        }
    }
    if look.0 == ModeKind::Puzzle {
        // Back strip between the middle and front ruins (screen layers sit at -900 + z); the
        // front strip is DesertForeground's sand, drawn by its own camera over the towers.
        for (front, name, z) in [
            (false, "puzzle_sand", -895.5),
            (true, "puzzle_sand_front", 400.0),
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
        for (band, &(front, picture, ..)) in weather.sandstorm_bands.iter().enumerate() {
            let name = format!("sandstorm_{}_{picture}", if front { "front" } else { "back" });
            let Some(image) = art.image(&name) else {
                continue;
            };
            let (tint, alpha) = if front {
                weather.sandstorm_front
            } else {
                weather.sandstorm_back
            };
            let mine = |b: &&(bool, u32, f32, f32)| b.0 == front;
            let index = weather.sandstorm_bands[..band].iter().filter(mine).count() as u32;
            let count = weather.sandstorm_bands.iter().filter(mine).count() as u32;
            // The front bands are DesertForeground's too: over the towers and bricks (they
            // sit low, so they only cover the floor's foot).
            let z = if front { 401.0 } else { -600.0 } + index as f32;
            for copy in 0..2 {
                commands.entity(camera).with_child((
                    Weather,
                    SandLayer {
                        band,
                        index,
                        count,
                        copy,
                    },
                    Sprite {
                        image: image.clone(),
                        color: rgba(tint, alpha),
                        ..default()
                    },
                    Transform::from_xyz(0.0, 0.0, z),
                ));
            }
        }
    }
    if look.0 == ModeKind::Puzzle
        && look.1.is_none()
        && let Some(sun) = art.image("puzzle_sun")
    {
        commands.spawn((
            Weather,
            DesertSun,
            Sprite::from_image(sun),
            Transform::from_xyz(0.0, 0.0, -930.0),
            ChildOf(camera),
        ));
    }
    // Endless Puzzle has its own sun (DesertEndlessSun): a moon slides over it during the
    // intro, leaving an orange crescent; no corona.
    if look.1 == Some("endless_puzzle")
        && let (Some(ramp), Some(meshes), Some(suns)) = (art.image("eclipse_gradient"), meshes.as_mut(), suns.as_mut())
    {
        commands.spawn((
            Weather,
            EndlessSunQuad,
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(suns.add(crate::scene_material::EndlessSun {
                look: crate::scene_material::EndlessSunLook::at(0.0),
                ramp,
            })),
            Transform::from_xyz(0.0, 0.0, -930.0),
            ChildOf(camera),
        ));
    }
    if look.1 == Some("puzzle_pro") {
        if let Some(sun) = art.image("puzzle_pro_sun") {
            commands.spawn((
                Weather,
                EclipseSun,
                Sprite::from_image(sun),
                Transform::from_xyz(0.0, 0.0, -930.0),
                ChildOf(camera),
            ));
        }
        if let Some(ray) = art.image("puzzle_pro_corona") {
            for i in 0..weather.corona.len() {
                commands.spawn((
                    Weather,
                    CoronaRay(i),
                    Sprite::from_image(ray.clone()),
                    Transform::from_xyz(0.0, 0.0, -935.0 + i as f32 * 0.01),
                    ChildOf(camera),
                ));
            }
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
    camera: Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>,
    mut drops: Query<(Entity, &mut RainDrop, &mut Transform), Without<crate::visuals::MainCamera>>,
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
            // Rain and RainBackground draw as stretched billboards (length scale 5 times the
            // particle size): gph_rainDrop is a thin streak along its width, so it lies along
            // the fall, its bright end leading.
            let (sprite, turn) = match &image {
                Some(image) => (
                    Sprite {
                        image: image.clone(),
                        color: Color::srgba(colour[0], colour[1], colour[2], colour[3]),
                        custom_size: Some(Vec2::new(c * length * weather.rain_stretch, c * length)),
                        ..default()
                    },
                    dir.y.atan2(dir.x),
                ),
                None => (
                    Sprite::from_color(
                        Color::srgba(colour[0], colour[1], colour[2], colour[3] * 0.6),
                        Vec2::new(c * 0.06, c * length),
                    ),
                    tilt,
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
                    .with_rotation(Quat::from_rotation_z(turn)),
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
    camera: Query<(Entity, &Transform, &Projection), With<crate::visuals::MainCamera>>,
    mut strikes: Query<(Entity, &mut Strike, &mut Sprite, Has<FlashStrike>)>,
) {
    let dt = time.delta_secs();
    for (entity, mut strike, mut sprite, _) in &mut strikes {
        strike.time += dt;
        let fading = (strike.time - strike.hold).max(0.0) / strike.fade.max(0.001);
        if fading >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        sprite.color.set_alpha(strike.alpha * (1.0 - fading));
    }
    // The islands' lit pictures fade on their own clock (IslandsProLayer: 1 to 0 over
    // 0.2 s, DOTween's default out-quad), longer than the screen flash's 0.1 s.
    let since = state.since_strike.map(|t| t + dt);
    state.since_strike = since;
    let k = since.map_or(1.0, |t| (t / config.weather.lightning_islands.max(0.001)).min(1.0));
    flash.0 = (1.0 - k) * (1.0 - k);
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
    state.since_strike = Some(0.0);
    // One of the three thunder strikes (LIGHTNING_STRIKE_01..03).
    if let Some(thunder) = art.sounds.get(&format!("sfx_thunder_{}", 1 + (roll(&mut seed) * 3.0) as usize % 3)) {
        commands.spawn((AudioPlayer(thunder.clone()), PlaybackSettings::DESPAWN));
    }
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
    // LightningLayer._CreateLightningBranch: 10 points from top to bottom, all but the
    // first nudged up to a unit sideways; at the fourth point, while the strike has fewer
    // than its most bolts, 4 in 5 times a branch runs to the bottom up to 20 units aside.
    let mut bolts = 0;
    let mut stack = vec![(top, bottom)];
    while let Some((from, to)) = stack.pop() {
        bolts += 1;
        let mut points = Vec::with_capacity(10);
        for i in 0..10 {
            let mut p = from.lerp(to, i as f32 / 9.0);
            if i > 0 {
                p.x += (roll(&mut seed) * 2.0 - 1.0) * c;
            }
            points.push(p);
            if i == 3 && bolts + stack.len() < weather.lightning_branches as usize && roll(&mut seed) > 0.2 {
                stack.push((p, to + Vec2::X * (roll(&mut seed) * 40.0 - 20.0) * c));
            }
        }
        for pair in points.windows(2) {
            segment(&mut commands, pair[0], pair[1], c * 0.8);
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
    camera: Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>,
    mut parts: ParamSet<(
        Query<(&SandStrip, &mut Sprite, &mut Transform), Without<crate::visuals::MainCamera>>,
        Query<(&SandLayer, &mut Sprite, &mut Transform), Without<crate::visuals::MainCamera>>,
        Query<
            (&mut Sprite, &mut Transform, Has<DesertSun>),
            (Or<(With<EclipseSun>, With<DesertSun>)>, Without<crate::visuals::MainCamera>),
        >,
        Query<(&CoronaRay, &mut Sprite, &mut Transform), Without<crate::visuals::MainCamera>>,
        Query<(&SurvivalMist, &mut Sprite, &mut Transform), Without<crate::visuals::MainCamera>>,
    )>,
    climb: Option<Res<crate::visuals::ViewClimb>>,
) {
    let weather = &config.weather;
    let (_, size) = view(&camera, view_size.0);
    let t = time.elapsed_secs();
    // The sand and sandstorm layers (parallax 1) sink as the camera climbs.
    let sink = climb.as_deref().map_or(0.0, |c| c.0) * size.y / 41.54;
    for (strip, mut sprite, mut transform) in &mut parts.p0() {
        let (h, lift) = if strip.front {
            (weather.sand_front, 0.0)
        } else {
            (weather.sand_strip, weather.sand_lift)
        };
        let h = size.y * h;
        sprite.custom_size = Some(Vec2::new(size.x, h));
        transform.translation.y = -size.y * (0.5 - lift) + h * 0.5 - sink;
    }
    let unit = size.y / 41.54;
    for (layer, mut sprite, mut transform) in &mut parts.p1() {
        let Some(&(_, _, centre, height)) = weather.sandstorm_bands.get(layer.band) else {
            continue;
        };
        // A band across the view near its bottom; two copies side by side slide
        // across and wrap, each band starting a share further on and running faster.
        sprite.custom_size = Some(Vec2::new(size.x, height * unit));
        transform.translation.y = -size.y * 0.5 + centre * unit - sink;
        let i = layer.index as f32;
        let speed = weather.sandstorm_speed + i * weather.sandstorm_offset;
        let shift = (t * speed + i / layer.count.max(1) as f32).fract();
        transform.translation.x = (layer.copy as f32 - shift) * size.x;
    }
    for (mist, mut sprite, mut transform) in &mut parts.p4() {
        let top = -size.y * 0.5 + size.y * mist.top;
        if mist.edge {
            let h = MIST_EDGE * mist.squash * unit;
            sprite.custom_size = Some(Vec2::new(size.x, h));
            transform.translation.y = top + h * 0.25;
        } else {
            let h = size.y * mist.top;
            sprite.custom_size = Some(Vec2::new(size.x, h));
            transform.translation.y = top - h * 0.5;
        }
    }
    let sun_size = size.y * weather.eclipse_size;
    let sun_y = size.y * weather.eclipse_height;
    // The corona fades in with the sun (Endless Puzzle's intro).
    let mut sun_alpha = 1.0;
    for (mut sprite, mut transform, plain) in &mut parts.p2() {
        transform.translation.y = sun_y;
        if plain {
            // gph_sun, 746x413 px at 26 px a unit, 0.1 units right of centre.
            sprite.custom_size = Some(Vec2::new(746.0, 413.0) / 26.0 * unit);
            transform.translation.x = 0.1 * unit;
            continue;
        }
        sprite.custom_size = Some(Vec2::splat(sun_size));
        sun_alpha = sprite.color.alpha();
    }
    for (ray, mut sprite, mut transform) in &mut parts.p3() {
        let Some(part) = weather.corona.get(ray.0) else {
            continue;
        };
        sprite.custom_size = Some(Vec2::new(683.0, 97.0) / 26.0 * unit * Vec2::from(part.scale));
        transform.translation.x = 0.0;
        transform.translation.y = sun_y;
        transform.rotation = Quat::from_rotation_z((part.angle + part.spin * t).to_radians());
        let (low, high, rate) = part.fade;
        let swing = ((t * rate * std::f32::consts::TAU).sin() + 1.0) / 2.0;
        let [r, g, b] = part.tint;
        sprite.color = Color::srgba_u8(r, g, b, 255).with_alpha((low + swing * (high - low)) * sun_alpha);
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
    // DOTween's default out-quad on the way in and on the way back (SurvivalBackground's
    // 1 s curse-mode tween).
    let p = state.wave_mix;
    mix.0 = if wave { 1.0 - (1.0 - p) * (1.0 - p) } else { p * p };
    // Redraw only while the colours are moving.
    if mix.0 == state.wave_drawn {
        return;
    }
    state.wave_drawn = mix.0;
    let colors: Vec<Color> = weather
        .survival_sky
        .iter()
        .zip(&weather.survival_wave_sky)
        .map(|(&[r, g, b], &[wr, wg, wb])| {
            Color::srgb_u8(r, g, b).mix(&Color::srgb_u8(wr, wg, wb), mix.0)
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
