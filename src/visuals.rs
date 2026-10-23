//! Visuals: bricks, platforms, goal and spell lines, a drop guide, a preview
//! of the next brick, a camera that keeps every tower in view, and a text HUD.
//! Bricks, platforms and backgrounds use the original's art when it has been
//! extracted from your own install (see [`crate::art`]); otherwise everything
//! is drawn as plain coloured shapes.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::art::{Animated, Art};
use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{
    ActiveBrick, Brick, FloorBlock, FloorPart, Match, Phase, Platform, Tower, TowerState,
};
use crate::gameplay::{Placed, piece_def};
use crate::spells::{Bubbled, Grassy, LifeOrb, SpellLog, Stone, Vined, Zapped};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .init_resource::<Art>()
            .init_resource::<ViewSize>()
            .init_resource::<MenuLook>()
            .init_resource::<MenuScene>()
            .init_resource::<SceneState>()
            .init_resource::<ViewClimb>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    track_menu_scene.before(spawn_backgrounds),
                    show_scene_in_menus.after(spawn_backgrounds),
                    bob_islands.before(fit_backgrounds),
                    wave_islands.after(bob_islands),
                    dissolve_zapped,
                    ripple_water.after(fit_strips),
                    pulse_band,
                    run_waves.after(fit_backgrounds),
                ),
            )
            .add_systems(
                Update,
                (
                    wobble_bubbles,
                    show_edge_delays,
                    turn_flowers,
                    grow_edges,
                    pulse_eggs,
                ),
            )
            .add_systems(Update, twinkle_stars.before(fit_backgrounds))
            .add_systems(Update, fade_eclipse)
            .add_systems(Update, sweep_lasers)
            .add_systems(Update, place_pillar_bases.after(frame_towers))
            .add_systems(Update, drop_stars.before(twinkle_stars))
            .add_systems(Update, (dress_tower_pieces, show_vines.before(dress_bricks), wave_finish_tiles))
            .add_systems(
                Update,
                (
                    dress_bricks,
                    dress_floor.after(dress_bricks),
                    dress_bubbles,
                    dress_life_orbs,
                    track_view_size.before(fit_backgrounds).before(frame_towers),
                    spawn_backgrounds,
                    fit_backgrounds,
                    light_layers,
                    grey_wave_layers,
                    scroll_lane_layers,
                    fit_strips,
                    redress_on_art_change,
                    dress_platforms,
                    update_drop_guide.after(frame_towers),
                    frame_towers.in_set(FramedCamera),
                    update_hud,
                    update_lines,
                    show_trial_clock,
                    (update_mist, spin_orbs),
                ),
            );
    }
}

/// A copy of `source` that stands in for the original's additive beams
/// (Custom/BeamLaneShaderAdditive) drawn at `strength`: sprites only blend, so each
/// pixel is whitened and given the alpha its added light would have.
// ponytail: fixed whitening, not true additive blending; a Material2d with an add blend if it matters.
pub(crate) fn glow_copy(source: &Image, strength: f32) -> Option<Image> {
    let mut out = source.clone();
    let data = out.data.as_mut()?;
    for px in data.chunks_exact_mut(4) {
        let peak = px[0].max(px[1]).max(px[2]) as f32 / 255.0;
        let alpha = (2.0 * strength * peak * px[3] as f32 / 255.0).min(1.0);
        for c in &mut px[..3] {
            let bright = if peak > 0.0 { *c as f32 / peak } else { 255.0 };
            *c = (bright + (255.0 - bright) * 0.6) as u8;
        }
        px[3] = (alpha * 255.0) as u8;
    }
    Some(out)
}

/// An Endless Puzzle eclipse layer and its full colour: it fades in over 2 s from
/// 1.5 s into the match (EndlessPuzzleGameModeIntroController: 0.5 s, the laser
/// moving in, 0.5 s, the eclipse sound, EclipseEffect; the start is ?).
#[derive(Component)]
pub(crate) struct EclipseFade(pub(crate) Color);

const ECLIPSE_START: f32 = 1.5;
const ECLIPSE_TIME: f32 = 2.0;

/// How far Endless Puzzle's eclipse is, 0 to 1: done behind the menus, else over its 2 s
/// in the intro.
pub(crate) fn eclipse_progress(game: &Match) -> f32 {
    if game.phase == Phase::Menu {
        1.0
    } else {
        ((game.elapsed - ECLIPSE_START) / ECLIPSE_TIME).clamp(0.0, 1.0)
    }
}

fn fade_eclipse(
    mut commands: Commands,
    game: Res<Match>,
    art: Res<Art>,
    mut layers: Query<(&EclipseFade, &mut Sprite)>,
    mut played: Local<u64>,
) {
    let t = eclipse_progress(&game);
    if game.phase == Phase::Playing && game.elapsed >= ECLIPSE_START && *played != game.round {
        *played = game.round;
        if let Some(sound) = art.sounds.get("sfx_eclipse") {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
    for (fade, mut sprite) in &mut layers {
        let want = fade.0.with_alpha(fade.0.alpha() * t);
        if sprite.color != want {
            sprite.color = want;
        }
    }
}

/// A twinkling star (StarrySky): its scale breathes 1 ± 0.3, and now and then one
/// swells to 2x in 0.3 s, turns half a turn in 1 s (in-back) with a spin that slows,
/// waits 2 s and shrinks back in 0.3 s; the next comes 2-5 s later.
#[derive(Component, Default)]
pub(crate) struct Star {
    time: f32,
    turn: f32,
    spin: f32,
    twirl: Option<f32>,
    /// The scale this frame, applied by `fit_backgrounds`.
    factor: f32,
    /// Knocked loose: how far it has fallen and its speed, in screen units.
    fall: Option<(Vec2, Vec2)>,
}

/// Survival's moon: it drops when a brick of 2.5 or more lands (SurvivalBackground
/// `_HandlePlaceBrick`, where the moon is a hinged rigid body with gravity 3).
#[derive(Component)]
pub(crate) struct Moon;

/// A background layer knocked loose and falling: how far it has dropped and its
/// speed, in screen units. Gravity is 9.81 units/s² (the moon's is 3 times that).
#[derive(Component, Clone, Copy)]
pub(crate) struct Falling {
    offset: Vec2,
    velocity: Vec2,
    gravity: f32,
}

/// Star sounds (SFX_STAR_MOVE_1-4), one picked at random.
const STAR_SOUNDS: [&str; 4] = [
    "sfx_amb_ambience_stars_01",
    "sfx_amb_ambience_stars_02",
    "sfx_amb_ambience_stars_03",
    "sfx_amb_ambience_stars_04",
];

/// A Large brick placed in Survival knocks a star out of the sky (SurvivalBackground
/// `_HandlePlaceBrick`, StarrySkyLayer.DropStar): it leaves a dark copy (#2E1D5B)
/// behind and falls from an upward kick of (-2..2, 6) units/s, spinning 40x its
/// sideways speed. The moon only falls for bricks 2.5x or more: a brick hit by Large twice
/// (LargeBrickModifier: 2 + 0.5 per earlier Large).
#[allow(clippy::too_many_arguments)]
fn drop_stars(
    mut commands: Commands,
    game: Res<Match>,
    art: Res<Art>,
    placed: Query<&Brick, Added<crate::gameplay::Placed>>,
    mut stars: Query<(
        Entity,
        &mut Star,
        &ScreenLayer,
        &Sprite,
        &Anchor,
        &Transform,
        &ChildOf,
    )>,
    moons: Query<Entity, (With<Moon>, Without<Falling>)>,
    mut seed: Local<u32>,
) {
    if game.mode.look() != ModeKind::Survival {
        return;
    }
    for brick in &placed {
        if brick.scale + 0.01 >= 2.5
            && let Ok(moon) = moons.single()
        {
            commands.entity(moon).insert(Falling {
                offset: Vec2::ZERO,
                velocity: Vec2::ZERO,
                gravity: 3.0 * 9.81,
            });
        }
        if brick.scale + 0.01 < 2.0 {
            continue;
        }
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) | 1;
        let standing: Vec<Entity> = stars
            .iter()
            .filter(|(_, s, ..)| s.fall.is_none())
            .map(|(e, ..)| e)
            .collect();
        if let Some(sound) = art.sounds.get(STAR_SOUNDS[(*seed >> 3) as usize % 4]) {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
        }
        if standing.is_empty() {
            continue;
        }
        let pick = standing[(*seed >> 8) as usize % standing.len()];
        let Ok((_, mut star, layer, sprite, anchor, transform, parent)) = stars.get_mut(pick)
        else {
            continue;
        };
        let kick = (((*seed >> 12) % 400) as f32 / 100.0) - 2.0;
        star.fall = Some((Vec2::ZERO, Vec2::new(kick, 6.0)));
        star.spin = kick * 40.0;
        star.twirl = None;
        let mut dark = sprite.clone();
        dark.color = Color::srgb_u8(0x2E, 0x1D, 0x5B);
        let mut copy = layer.clone();
        copy.drift = 0.0;
        commands.spawn((
            copy,
            dark,
            *anchor,
            Transform::from_translation(transform.translation - Vec3::Z * 0.001),
            WaveLook::Keep,
            ChildOf(parent.parent()),
        ));
    }
}

fn twinkle_stars(
    time: Res<Time>,
    mut next: Local<(f32, u32)>,
    mut stars: Query<(&mut Star, &mut Transform, &GlobalTransform), Without<crate::visuals::MainCamera>>,
) {
    let dt = time.delta_secs();
    let count = stars.iter().count() as u32;
    next.0 -= dt;
    if next.0 <= 0.0 && count > 0 {
        next.1 = next.1.wrapping_mul(1_103_515_245).wrapping_add(12_345);
        let pick = (next.1 >> 8) % count;
        if let Some((mut star, ..)) = stars.iter_mut().nth(pick as usize)
            && star.twirl.is_none()
        {
            star.twirl = Some(0.0);
        }
        next.0 = 2.0 + ((next.1 >> 4) % 1000) as f32 / 1000.0 * 3.0;
    }
    for (mut star, mut transform, at) in &mut stars {
        star.time += dt;
        star.turn += star.spin * dt;
        star.spin = (star.spin - 200.0 * dt).max(0.0);
        let p = at.translation();
        let mut scale = 1.0 + (star.time * 2.0 + p.x * p.y * 1.0e-4).sin() * 0.3;
        let mut turn = 0.0;
        if let Some(t) = star.twirl.as_mut() {
            *t += dt;
            let t = *t;
            let back = |x: f32| {
                let c = 1.70158;
                (c + 1.0) * x * x * x - c * x * x
            };
            scale = if t < 0.3 {
                1.0 + t / 0.3
            } else if t < 3.3 {
                2.0
            } else {
                2.0 - ((t - 3.3) / 0.3).min(1.0)
            };
            turn = 180.0 * back(((t - 0.3) / 1.0).clamp(0.0, 1.0));
            if (1.3..1.3 + dt).contains(&t) {
                star.spin = 600.0;
            }
            if t >= 3.6 {
                star.turn += 180.0;
                star.twirl = None;
                star.time = 0.0;
            }
        }
        transform.rotation = Quat::from_rotation_z((star.turn + turn).to_radians());
        star.factor = scale;
    }
}

/// The mode whose background shows behind a menu, if any, and whether in its Special look:
/// the picked Endless mode on the Endless screen, the picked mode and difficulty on Select Mode.
#[derive(Resource, Default, PartialEq)]
pub struct MenuLook(pub Option<(ModeKind, bool)>);

/// The water strip at the depth where dropped bricks are lost. Modes whose water band
/// is turned off (Survival, Puzzle) have no water at all.
#[derive(Component)]
struct WaterStrip;

/// After this set the camera is where it will be drawn from this frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FramedCamera;

/// Size of the view in logical pixels: the window's, or the offscreen picture's.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewSize(pub Vec2);

impl Default for ViewSize {
    fn default() -> Self {
        Self(Vec2::new(1280.0, 720.0))
    }
}

/// The view's size as the camera's projection sees it (its visible area over its zoom), so
/// everything sized from it covers the screen exactly. With display scaling (125% on Max's
/// screen) the window's own size didn't match it, and the background stopped short of the
/// screen's sides and top.
fn track_view_size(
    cameras: Query<(&Camera, &Projection), With<MainCamera>>,
    windows: Query<&Window>,
    mut view: ResMut<ViewSize>,
) {
    let size = cameras
        .iter()
        .find_map(|(camera, projection)| match projection {
            Projection::Orthographic(o) if o.area.width() > 0.0 && o.scale > 0.0 => {
                Some(o.area.size() / o.scale)
            }
            _ => camera.logical_viewport_size(),
        })
        .or_else(|| windows.single().ok().map(|w| Vec2::new(w.width(), w.height())));
    if let Some(size) = size
        && view.0 != size
    {
        view.0 = size;
    }
}

#[derive(Component)]
struct Hud;

/// A tower's drop guide: a column the falling brick's width from the top of the view
/// down to the bottom of the view or its lowest point (BrickGuide).
#[derive(Component)]
pub(crate) struct DropGuide {
    tower: Entity,
}

/// A tower's spell line.
#[derive(Component)]
struct SpellLine {
    tower: Entity,
}

/// The pieces in a spell line (SpellContainerSpawner's container width).
const SPELL_LINE_PIECES: u32 = 16;

/// One diamond of a spell line.
#[derive(Component)]
struct SpellDiamond;

/// The race trial's pace line (PaceLinePersonalBest): a thin line across the screen
/// that climbs from the base to the finish as the time runs out, with the hourglass
/// and the time left on it; `0` is its height over the line.
#[derive(Component)]
struct TrialClock(f32);

/// Where the trial clock's time stands on the pace line; the time itself is a UI text over
/// it (`ClockLabel`), so it gets the original's letters and outline.
#[derive(Component)]
struct ClockAnchor;

/// The trial clock's time in EnzoBlackYellowOrange: pale yellow face, orange outline
/// (dilate and outline 0.2), the world font size 20 (1.75 cells here).
#[derive(Component)]
struct ClockLabel;
const CLOCK_FACE: Color = Color::srgb(1.0, 0.984, 0.749);
const CLOCK_OUTLINE: Color = Color::srgb(1.0, 0.373, 0.094);

/// The race finish line or puzzle laser over a tower; both can come down.
#[derive(Component)]
pub(crate) struct FinishLine {
    pub(crate) tower: Entity,
}

/// A Mist spell's fog (Game/Magic/Mist): a bank at a fixed height that sends out puffs
/// for its first 10 s, fading out over 1 s when the match ends.
#[derive(Component)]
struct MistBank {
    age: f32,
    fade: f32,
    due: f32,
}

/// One of a bank's puffs (MistParticles): a soft streak stretched along its drift.
#[derive(Component)]
struct MistPuff {
    age: f32,
    colour: Color,
}

/// MistParticles: 2 puffs a second for 10 s, each living 5 s, born 9 units right of the
/// bank anywhere in its 5-unit band and drifting right at 0.5; 4 units tall, drawn as a
/// stretched billboard 5 sizes long behind its head; colour between its two start colours,
/// alpha along its lifetime keys (Particles/Alpha Blended at tint 0.5, so as given).
/// Mist.HEIGHT: a bank's band, and the step between stacked banks.
const MIST_HEIGHT: f32 = 5.0;
const MIST_RATE: f32 = 2.0;
const MIST_EMIT: f32 = 10.0;
const MIST_LIFE: f32 = 5.0;
const MIST_HEAD_X: f32 = 9.0;
const MIST_SPEED: f32 = 0.5;
const MIST_SIZE: f32 = 4.0;
const MIST_LENGTH: f32 = 5.0;
const MIST_COLOURS: [(f32, f32, f32); 2] = [(0.816, 0.816, 0.992), (0.769, 0.769, 0.976)];
const MIST_ALPHA: [(f32, f32); 5] = [
    (0.0, 0.0),
    (0.203, 0.71),
    (0.488, 0.49),
    (0.774, 0.824),
    (1.0, 0.0),
];

/// Child sprite of a brick, one per cell.
#[derive(Component)]
pub(crate) struct CellSprite;

/// The camera the game is seen through (the screen grab for the drop guide is another).
#[derive(Component)]
pub struct MainCamera;

fn setup_scene(mut commands: Commands, config: Res<GameConfig>, art: Res<Art>) {
    commands.spawn((
        Camera2d,
        MainCamera,
        IsDefaultUiCamera,
        Transform::from_xyz(0.0, config.cells(6.0), 0.0),
    ));
    // Water under every lane, its top at the depth where dropped bricks are lost.
    let depth = Vec3::new(0.0, -config.cells(config.kill_depth), -800.0);
    let width = config.cells(config.lanes.screen_width * 3.0);
    if let Some(water) = art_strip(&art, "water", config.cell_size, width, depth) {
        commands.spawn((WaterStrip, RippleWater, water));
    }
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

pub(crate) fn cell_sprites(
    parent: &mut ChildSpawnerCommands,
    cells: &[Vec2],
    cell: f32,
    color: Color,
) {
    for &p in cells {
        parent.spawn((
            CellSprite,
            Sprite::from_color(color, Vec2::splat(cell * 0.94)),
            Transform::from_translation((p * cell).extend(0.0)),
        ));
    }
}

const STONE: Color = Color::srgb(0.45, 0.45, 0.48);
const VINES: Color = Color::srgb(0.2, 0.7, 0.25);
const GRASS: Color = Color::srgb(0.35, 0.8, 0.2);

/// A sprite from the extracted art, anchored at its pivot and scaled from its
/// pixels to cells.
/// An art sprite repeated sideways to a set width (lines, water). Its height is
/// the image's own, at `pixels_per_unit`.
#[derive(Component)]
struct Strip {
    width: f32,
    /// Drawn this tall (world units) instead of the picture's own height.
    height: Option<f32>,
}

/// An art sprite for a strip `width` world units wide, or `None` without the art.
fn art_strip(art: &Art, name: &str, cell: f32, width: f32, at: Vec3) -> Option<impl Bundle> {
    let (sprite, anchor, transform) = art_sprite(art, name, cell)?;
    Some((
        Strip { width, height: None },
        sprite,
        anchor,
        transform.with_translation(at),
    ))
}

/// An art strip drawn `height` world units tall (a quad the original stretched).
fn art_strip_tall(art: &Art, name: &str, cell: f32, width: f32, height: f32, at: Vec3) -> Option<impl Bundle> {
    let (sprite, anchor, transform) = art_sprite(art, name, cell)?;
    Some((
        Strip { width, height: Some(height) },
        sprite,
        anchor,
        transform.with_translation(at),
    ))
}

/// The bright Puzzle laser that sweeps in at a battle's start, and the plain one.
#[derive(Component)]
struct LaserSweep {
    width: f32,
    tower: Entity,
    /// Seconds since a zap flashed it.
    flash: Option<f32>,
}
#[derive(Component)]
struct LaserRest;

/// The bright laser grows in from the right edge, from 2.5 times its height, then
/// thins to nothing while the plain laser shows (out-quad, DOTween's default). A zap
/// flashes it twice as tall, thinning over 1 s.
fn sweep_lasers(
    time: Res<Time>,
    config: Res<GameConfig>,
    laser: Option<Res<crate::intro::LaserIn>>,
    towers: Query<&TowerState>,
    mut bright: Query<(&mut LaserSweep, &mut Transform, &mut Visibility)>,
    mut rest: Query<&mut Visibility, (With<LaserRest>, Without<LaserSweep>)>,
) {
    let look = &config.intro;
    let t = laser.and_then(|l| l.0);
    let out_quad = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        1.0 - (1.0 - x) * (1.0 - x)
    };
    let resting = t.is_none_or(|t| t >= look.laser_sweep);
    for mut visibility in &mut rest {
        visibility.set_if_neq(if resting {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
    }
    for (mut sweep, mut transform, mut visibility) in &mut bright {
        let zapped = towers
            .get(sweep.tower)
            .is_ok_and(|s| s.status == crate::gameplay::TowerStatus::Out);
        if !zapped {
            sweep.flash = None;
        } else if let Some(flash) = sweep.flash.as_mut() {
            *flash += time.delta_secs();
        } else {
            sweep.flash = Some(0.0);
        }
        if let Some(flash) = sweep.flash.filter(|&f| f < 1.0) {
            visibility.set_if_neq(Visibility::Inherited);
            transform.translation.x = 0.0;
            transform.scale = Vec3::new(1.0, (2.0 * (1.0 - out_quad(flash))).max(0.001), 1.0);
            continue;
        }
        let Some(t) = t.filter(|&t| t >= 0.0) else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        visibility.set_if_neq(Visibility::Inherited);
        let (x, w, h) = if t < look.laser_sweep {
            let k = out_quad(t / look.laser_sweep.max(0.01));
            (sweep.width * 0.5 * (1.0 - k), k, 2.5 + (1.0 - 2.5) * k)
        } else {
            let k = out_quad((t - look.laser_sweep) / look.laser_fade.max(0.01));
            (0.0, 1.0, 1.0 - k)
        };
        transform.translation.x = x;
        transform.scale = Vec3::new(w.max(0.001), h.max(0.001), 1.0);
    }
}

/// Sizes strips once their image is loaded: the image repeats at its own size.
fn fit_strips(images: Res<Assets<Image>>, mut strips: Query<(&Strip, &mut Sprite, &Transform)>) {
    for (strip, mut sprite, transform) in &mut strips {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        if sprite.custom_size.is_some() {
            continue;
        }
        let size = image.size_f32();
        let scale = transform.scale.x.max(f32::EPSILON);
        let height = strip.height.map_or(size.y, |h| h / transform.scale.y.max(f32::EPSILON));
        sprite.custom_size = Some(Vec2::new(strip.width / scale, height));
        // A stretched quad (the Puzzle laser: its picture spans the whole quad once) isn't
        // tiled; the other strips repeat their picture along.
        if strip.height.is_none() {
            sprite.image_mode = SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: 1.0,
            };
        }
    }
}

pub(crate) fn art_sprite(art: &Art, name: &str, cell: f32) -> Option<(Sprite, Anchor, Transform)> {
    let image = art.image(name)?;
    let entry = art.entries.get(name)?;
    let (px, py) = entry.pivot.unwrap_or((0.5, 0.5));
    let scale = cell / entry.pixels_per_unit.unwrap_or(100.0).max(1.0);
    Some((
        Sprite::from_image(image),
        Anchor(Vec2::new(px - 0.5, py - 0.5)),
        Transform::from_scale(Vec3::splat(scale)),
    ))
}

/// Draws the Puzzle floor with the original's pieces when they are staged: a column
/// piece hanging from each block's top-left corner with tiles stacked behind it, or
/// the Pro pillar and its ledges. Unstaged, the floor stays plain cells.
#[allow(clippy::type_complexity)]
fn dress_floor(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    blocks: Query<(Entity, &FloorBlock, &Brick), Or<(Added<FloorBlock>, Changed<Brick>)>>,
) {
    let c = config.cell_size;
    for (entity, block, brick) in &blocks {
        let name = match block.part {
            FloorPart::Column(0) => "puzzle_piece_1".to_string(),
            FloorPart::Column(_) => "puzzle_piece_3".to_string(),
            FloorPart::Wide => "puzzle_piece_2".to_string(),
            FloorPart::Pillar => "puzzle_pro_center".to_string(),
            FloorPart::Ledge(n) => format!("puzzle_pro_ledge_{}", n + 1),
            FloorPart::TopLedge => "puzzle_pro_ledge_top".to_string(),
        };
        let Some((sprite, anchor, mut transform)) = art_sprite(&art, &name, c) else {
            continue;
        };
        let half = brick.cells.len() as f32 * 0.5;
        let mut node = commands.entity(entity);
        // Its pictures are children, so it needs visibility of its own.
        node.despawn_children().insert(Visibility::default());
        if matches!(block.part, FloorPart::Column(_) | FloorPart::Pillar | FloorPart::Wide) {
            // Pivot on the top-left, a hair left of and above the block's corner (two cells
            // wide for the wide piece).
            let left = if block.part == FloorPart::Wide { -1.0 } else { -0.5 };
            transform.translation = Vec3::new(c * (left - 0.03), c * (half + 0.03), 0.0);
        }
        node.with_child((sprite, anchor, transform));
        if matches!(block.part, FloorPart::Column(_)) {
            // Tiles behind the column every 2 cells down, and a cap on the stack.
            for k in 0..6 {
                if let Some((sprite, anchor, mut t)) = art_sprite(&art, "puzzle_piece_base", c) {
                    t.translation = Vec3::new(0.0, c * (half - 1.0 - 2.0 * k as f32), -0.1);
                    node.with_child((sprite, anchor, t));
                }
            }
            if let Some((sprite, anchor, mut t)) = art_sprite(&art, "puzzle_piece_base_top", c) {
                t.translation = Vec3::new(0.0, c * (half + 0.37), -0.1);
                node.with_child((sprite, anchor, t));
            }
        }
    }
}

/// Draws each brick: the original's sprite when one is staged (untinted, with its
/// stone or ice version when the brick is petrified or iced), else coloured cells.
/// Runs again when a brick changes, turns to stone, or gets vines or grass.
#[allow(clippy::type_complexity)]
/// A decoration that grows in from nothing when it shows, as the original's edge decorations do
/// (`packs::edge_growth`).
#[derive(Component)]
struct EdgeGrow {
    start: f32,
}

/// An Easter egg that pulses after its delay: one axis squeezes to 80% in 0.1 s, then springs
/// back (elastic, 1 s), the other axis after it or before it, as the original's egg tween.
#[derive(Component)]
struct EggPulse {
    start: f32,
    x_first: bool,
}

fn grow_edges(mut commands: Commands, time: Res<Time>, mut grows: Query<(Entity, &EdgeGrow, &mut Transform)>) {
    let now = time.elapsed_secs();
    for (entity, grow, mut transform) in &mut grows {
        let t = now - grow.start;
        let k = crate::packs::edge_growth(t);
        transform.scale = Vec3::new(k, k, 1.0);
        // Grown: leave the scale to the egg's pulse.
        if t >= 0.5 {
            transform.scale = Vec3::ONE;
            commands.entity(entity).remove::<EdgeGrow>();
        }
    }
}

/// The next Easter decoration move starts 5 to 15 s after the last one ends
/// (EasterBrickModifier._AnimateDecoration repeats for as long as the brick lives).
fn next_easter_move(entity: Entity, now: f32) -> f32 {
    let h = (entity.to_bits() ^ now.to_bits() as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
    now + 5.0 + (h >> 40) as f32 / (1u64 << 24) as f32 * 10.0
}

fn pulse_eggs(time: Res<Time>, mut eggs: Query<(Entity, &mut EggPulse, &mut Transform)>) {
    let now = time.elapsed_secs();
    let elastic = |x: f32| {
        if x >= 1.0 {
            1.0
        } else {
            2f32.powf(-10.0 * x) * ((x - 0.075) * std::f32::consts::TAU / 0.3).sin() + 1.0
        }
    };
    for (entity, mut egg, mut transform) in &mut eggs {
        let t = now - egg.start;
        if t > 1.1 {
            egg.start = next_easter_move(entity, now);
            transform.scale = Vec3::ONE;
            continue;
        }
        // Before its time the egg is left to its growth in; then one axis squeezes first.
        if t < 0.0 {
            continue;
        }
        let squeeze = (t / 0.1).min(1.0);
        let spring = ((t - 0.1) / 1.0).clamp(0.0, 1.0);
        let k = if t < 0.1 { 1.0 - 0.2 * squeeze } else { 0.8 + 0.2 * elastic(spring) };
        let (x, y) = if egg.x_first {
            (k, if t < 0.1 { 1.0 } else { k })
        } else {
            (if t < 0.1 { 1.0 } else { k }, k)
        };
        transform.scale = Vec3::new(x, y, 1.0);
    }
}

/// An Easter flower that turns after its delay: a quarter turn (back ease, 0.5 s), then a
/// fast spin back (expo, 1.5 s), ending where it started, as the original's rotation tween.
#[derive(Component)]
struct FlowerTurn {
    start: f32,
    turn: f32,
}

fn turn_flowers(time: Res<Time>, mut flowers: Query<(Entity, &mut FlowerTurn, &mut Transform)>) {
    let now = time.elapsed_secs();
    let ease_in_back = |x: f32| {
        let c = 1.70158;
        x * x * ((c + 1.0) * x - c)
    };
    let ease_out_expo = |x: f32| if x >= 1.0 { 1.0 } else { 1.0 - 2f32.powf(-10.0 * x) };
    for (entity, mut flower, mut transform) in &mut flowers {
        let t = now - flower.start;
        if t > 2.0 {
            flower.start = next_easter_move(entity, now);
            transform.rotation = Quat::IDENTITY;
            continue;
        }
        let angle = if t < 0.0 {
            0.0
        } else if t < 0.5 {
            ease_in_back(t / 0.5) * 90.0 * flower.turn
        } else {
            90.0 * flower.turn - ease_out_expo(((t - 0.5) / 1.5).min(1.0)) * 450.0 * flower.turn
        };
        transform.rotation = Quat::from_rotation_z(angle.to_radians());
    }
}

/// An edge decoration that shows once its time comes.
#[derive(Component)]
struct EdgeDelay {
    show_at: f32,
}

/// A decoration whose growth starts with a sound (Grassy's: one of SFX_GRAS_GROW_1..4).
#[derive(Component)]
struct EdgeSound(&'static str);

fn show_edge_delays(
    mut commands: Commands,
    time: Res<Time>,
    art: Res<Art>,
    mut delays: Query<(Entity, &EdgeDelay, &mut Visibility, Option<&EdgeSound>)>,
) {
    let now = time.elapsed_secs();
    for (entity, delay, mut visibility, sound) in &mut delays {
        if now >= delay.show_at && *visibility == Visibility::Hidden {
            visibility.set_if_neq(Visibility::Inherited);
            if let Some(handle) = sound.and_then(|s| art.sounds.get(s.0)) {
                commands.spawn((AudioPlayer(handle.clone()), PlaybackSettings::DESPAWN));
                commands.entity(entity).remove::<EdgeSound>();
            }
        }
    }
}

pub(crate) fn dress_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    time: Res<Time>,
    bricks: Query<
        (
            Entity,
            &Brick,
            Has<Stone>,
            Has<Vined>,
            Option<&Grassy>,
            Option<&Placed>,
            Option<&ActiveBrick>,
            Has<crate::spells::VineDelay>,
            Has<crate::spells::IvyBrick>,
        ),
        (
            Or<(
                Changed<Brick>,
                Added<Stone>,
                Added<Vined>,
                Added<Grassy>,
                Added<Placed>,
            )>,
            Without<FloorBlock>,
            Without<crate::spells::TowerPiece>,
        ),
    >,
    skins: Query<&crate::gameplay::TowerSkin>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut lit_bricks: Option<ResMut<Assets<crate::brick_material::LitBrick>>>,
    images: Option<Res<Assets<Image>>>,
) {
    for (entity, brick, landed_stone, vined, grass, placed, active, vine_waits, ivy) in &bricks {
        // Joined bricks show their vines once their turn in the spread comes (show_vines);
        // the Ivy brick itself wears them from the cast, in hand too.
        let vined = (vined && !vine_waits) || ivy || active.is_some_and(|a| a.mods.ivy);
        let grassy = grass.is_some();
        let stone = landed_stone || active.is_some_and(|a| a.mods.petrify);
        let cell = config.cell_size * brick.scale;
        let def = piece_def(&config, brick.piece);
        let pack = skins.get(brick.tower).map(|s| s.pack).unwrap_or_default();
        // Galaxy, Diamond and Hologram bricks are drawn by their own materials in packs.rs;
        // their Ivy and grass still go on here.
        let own_material = matches!(
            pack,
            crate::config::BrickPack::Galaxy
                | crate::config::BrickPack::Diamond
                | crate::config::BrickPack::Hologram
        ) && brick.piece < config.pieces.len()
            && !stone
            && !brick.ice;
        // A normal brick takes its tower's brick pack art (default when not staged);
        // stone and ice keep their own looks.
        let key = if brick.piece < config.pieces.len() {
            let packed = pack.brick_art(&def.name);
            if stone || brick.ice || art.image(&packed).is_none() {
                format!("brick_{}", def.name)
            } else {
                packed
            }
        } else {
            format!("mystery_{}", def.name.to_lowercase())
        };
        let whole = brick.cells.len() == def.cells.len();
        let variant = |suffix: &str| art.image(&format!("{key}_{suffix}"));
        // The plain sprite stands in for a missing stone or ice version, tinted.
        let (image, mut color) = match (whole, stone, brick.ice) {
            (false, ..) => (None, brick.color),
            // Petrified: iced too has its own look (_PETRIFIED_ICE_PACK), and the Puzzle
            // desert its sandy stone (_PETRIFIED_DESERT_PACK, worldId 2).
            (true, true, ice) => match (ice
                .then(|| variant("stone_ice"))
                .flatten())
            .or_else(|| {
                (config.mode.look() == ModeKind::Puzzle)
                    .then(|| art.image(&format!("desert_stone_{}", def.name)))
                    .flatten()
            })
            .or_else(|| variant("stone"))
            {
                Some(image) => (Some(image), Color::WHITE),
                None => (art.image(&key), Color::WHITE.mix(&STONE, 0.75)),
            },
            (true, false, true) => match variant("ice") {
                Some(image) => (Some(image), Color::WHITE),
                None => (art.image(&key), brick.color),
            },
            (true, false, false) => (art.image(&key), Color::WHITE),
        };
        if image.is_none() {
            color = if stone {
                brick.color.mix(&STONE, 0.75)
            } else {
                brick.color
            };
        }
        let art_drawn = image.is_some();
        // Vines and grass are drawn over the sprite when staged, else tint it.
        // Vines grow over the brick with an animation made for its shape, when staged.
        let vine_name = [
            // By the shape, whatever the pack: a pack's key (brick_christmas_I) found no vine
            // animation, so its bricks got the generic T picture stretched over their box.
            if brick.piece < config.pieces.len() {
                format!("vine_overlay_{}", def.name)
            } else {
                format!("vine_overlay_mystery_{}", def.name.to_lowercase())
            },
            "vine_overlay".into(),
        ]
        .into_iter()
        .find(|name| art.image(name).is_some())
        .unwrap_or_default();
        let vine_art = art.image(&vine_name).filter(|_| art_drawn && vined);
        // Grassy grows grass and moss on the brick's top edges (below), not over it.
        let grass_art = art.image("grass_large_1").is_none()
            .then(|| art.image("grass_overlay").filter(|_| art_drawn && grassy))
            .flatten();
        if vined && vine_art.is_none() {
            color = color.mix(&VINES, if art_drawn { 0.25 } else { 0.4 });
        }
        if grassy && grass_art.is_none() {
            color = color.mix(&GRASS, if art_drawn { 0.3 } else { 0.5 });
        }
        let mut entity = commands.entity(entity);
        entity.insert(Visibility::default()).despawn_children();
        match image {
            Some(image) => {
                let min = brick.cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                let max = brick.cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                // The brick's picture at its own size (pixels over pixels a unit): the
                // original's art reaches a little past the cells, so neighbours' outlines meet.
                let native = |name: &str| -> Option<Vec2> {
                    let ppu = art.entries.get(name)?.pixels_per_unit?;
                    let size = images.as_ref()?.get(&art.image(name)?)?.size_f32();
                    Some(size / ppu.max(1.0) * cell)
                };
                let size = whole
                    .then(|| {
                        native(&key).or_else(|| {
                            let bevel = art.entries.get(&key)?.bevel.clone()?;
                            native(&bevel)
                        })
                    })
                    .flatten()
                    .unwrap_or((max - min + Vec2::ONE) * cell);
                let center = (min + max) * 0.5 * cell;
                // Later bricks sit in front of earlier ones: equal depths draw in
                // no fixed order, which looked like bricks on different layers.
                let at = Transform::from_translation(
                    center.extend(placed.map_or(0.0, |p| p.0 as f32 * 5.0e-5)),
                );
                // A plain brick with light colours is lit in the world by its material.
                let lit = (whole && !stone && !brick.ice && !own_material)
                    .then_some(())
                    .and(meshes.as_mut().zip(lit_bricks.as_mut()))
                    .and_then(|(meshes, lit_bricks)| {
                        crate::brick_material::lit_brick(&art, &key, size, color, meshes, lit_bricks)
                    });
                match lit {
                    _ if own_material => {}
                    Some((mesh, material)) => {
                        crate::brick_material::spawn_quad(&mut entity, at, color, CellSprite, (), mesh, material);
                    }
                    None => {
                        entity.with_child((
                            CellSprite,
                            Sprite {
                                image,
                                color,
                                custom_size: Some(size),
                                ..default()
                            },
                            at,
                        ));
                    }
                }
                // Mystery 2 carries its rotor on top, over a dark shadow (MysteryBrick02 prefab).
                if key == "mystery_blob" && whole {
                    let rotor = match (stone, brick.ice) {
                        (true, _) => "brick_mystery02_rotor_petrified",
                        (false, true) => "brick_mystery02_rotor_ice",
                        _ => "brick_mystery02_rotor",
                    };
                    let shadow = Color::srgba(0.043, 0.051, 0.173, 0.828);
                    for (name, at, z, tint) in [
                        (
                            "brick_mystery02_rotor_white",
                            Vec2::new(0.1, 0.24),
                            0.05,
                            shadow,
                        ),
                        (rotor, Vec2::new(-0.02, 0.43), 0.06, Color::WHITE),
                    ] {
                        if let Some((mut sprite, anchor, mut transform)) =
                            art_sprite(&art, name, cell)
                        {
                            sprite.color = tint;
                            transform.translation = (center + at * cell).extend(z);
                            entity.with_child((sprite, anchor, transform));
                        }
                    }
                }
                for (z, overlay, name) in
                    [(0.1, vine_art, vine_name.as_str()), (0.2, grass_art, "")]
                {
                    if let Some(overlay) = overlay {
                        let mut child = entity.commands();
                        // A vine animation's frames are bigger than the brick (leaves hang over
                        // its edges) and keep their own size and pivot; stretching them to the
                        // brick's box squashed the leaves.
                        let native = art
                            .frame_name(name, 1.0e6)
                            .and_then(|frame| art_sprite(&art, frame, cell));
                        let id = match native {
                            Some((_, anchor, scale)) => child.spawn((
                                Sprite::from_image(overlay),
                                anchor,
                                Transform::from_translation(center.extend(z)).with_scale(scale.scale),
                            )),
                            None => child.spawn((
                                Sprite {
                                    image: overlay,
                                    custom_size: Some(size),
                                    ..default()
                                },
                                Transform::from_translation(center.extend(z)),
                            )),
                        }
                        .id();
                        if art.is_animation(name) {
                            child.entity(id).insert(Animated::new(name));
                        }
                        entity.add_child(id);
                    }
                }
                // Easter puts grass on a brick's top edge and sometimes a flower (the original's
                // EdgeDecoration chances: 25% and 40%). Picked per brick by a hash of its id, so
                // it's the same each time it's drawn. Christmas snow is in packs.rs.
                let brick_bits = entity.id().to_bits();
                let roll = |salt: u64| {
                    let h = (brick_bits ^ salt).wrapping_mul(0x9E37_79B9_7F4A_7C15);
                    (h >> 40) as f32 / (1u64 << 24) as f32
                };
                // Easter's decorations, as the original's EasterBrickModifier places them on the
                // brick prefab's anchors: grass on the top edges (25% each anchor; the edge
                // anchors facing up are the cells' tops), and an egg or a flower on the face of
                // each cell (40% each; of the four front anchors per cell only the upright one
                // is kept), one of 8 eggs and 8 flowers. Each shows after its own 0 to 0.3 s and
                // grows in (`packs::edge_growth`).
                let mut picks: Vec<(String, Vec2, f32)> = Vec::new();
                // Petrified bricks, once landed, grow grass and moss on their top edges
                // (PetrifyBrickModifier: each anchor 50%, grass GRASS_0..6 tinted one of three
                // blue-greens, or the desert's #9DA247, after 0.5 to 0.8 s; moss MOSS_0..2, or
                // MOSS_DESERT_0..2, after 0.2 to 0.5 s).
                let desert = config.mode.look() == ModeKind::Puzzle;
                let mut petrified: Vec<(String, Vec2, f32, Color)> = Vec::new();
                if landed_stone {
                    for (i, top) in top_cells(&brick.cells).into_iter().enumerate() {
                        let at = (top + Vec2::Y * 0.5) * cell;
                        let salt = 700 + i as u64 * 13;
                        if roll(salt) < 0.5 {
                            let n = 1 + (roll(salt + 1) * 7.0) as usize;
                            let colour = if desert {
                                Color::srgb_u8(0x9D, 0xA2, 0x47)
                            } else {
                                [
                                    Color::srgb_u8(37, 134, 162),
                                    Color::srgb_u8(27, 128, 134),
                                    Color::srgb_u8(35, 146, 124),
                                ][(roll(salt + 2) * 3.0) as usize % 3]
                            };
                            petrified.push((format!("grass_tuft_{}", n.min(7)), at, 0.5 + roll(salt + 3) * 0.3, colour));
                        }
                        if roll(salt + 4) < 0.5 {
                            let n = 1 + (roll(salt + 5) * 3.0) as usize;
                            let name = if desert { "moss_desert" } else { "moss" };
                            petrified.push((format!("{name}_{}", n.min(3)), at, 0.2 + roll(salt + 6) * 0.3, Color::WHITE));
                        }
                    }
                }
                // Grassy (GrassyBrickModifier): large grass on every top edge after 0.5 to 0.8 s,
                // moss on half of them after 0.2 to 0.5 s, both from the cast's own delay.
                let mut first_grass = true;
                if let Some(g) = grass.filter(|_| art.image("grass_large_1").is_some()) {
                    for (i, top) in top_cells(&brick.cells).into_iter().enumerate() {
                        let at = (top + Vec2::Y * 0.5) * cell;
                        let salt = 900 + i as u64 * 13;
                        let n = 1 + (roll(salt) * 4.0) as usize;
                        petrified.push((format!("grass_large_{}", n.min(4)), at, g.delay + 0.5 + roll(salt + 1) * 0.3, Color::WHITE));
                        if roll(salt + 2) < 0.5 {
                            let n = 1 + (roll(salt + 3) * 6.0) as usize;
                            petrified.push((format!("moss_large_{}", n.min(6)), at, g.delay + 0.2 + roll(salt + 4) * 0.3, Color::WHITE));
                        }
                    }
                }
                for (name, at, wait, colour) in petrified {
                    let sound = (first_grass && name.starts_with("grass_large")).then(|| {
                        first_grass = false;
                        ["sfx_grass_grow_01", "sfx_grass_grow_02", "sfx_grass_grow_03", "sfx_grass_grow_04"]
                            [(roll(77) * 4.0) as usize % 4]
                    });
                    if let Some((mut sprite, anchor, transform)) = art_sprite(&art, &name, cell) {
                        sprite.color = colour;
                        let show_at = time.elapsed_secs() + wait;
                        entity.with_children(|parent| {
                            let mut holder = parent.spawn((
                                Transform::from_translation(at.extend(0.2)),
                                Visibility::Hidden,
                                EdgeDelay { show_at },
                                EdgeGrow { start: show_at },
                            ));
                            if let Some(sound) = sound {
                                holder.insert(EdgeSound(sound));
                            }
                            holder.with_child((sprite, anchor, transform));
                        });
                    }
                }
                // EasterBrickModifier is applied when the brick is placed, not before.
                if pack == crate::config::BrickPack::Spring && placed.is_some() {
                    for (i, top) in top_cells(&brick.cells).into_iter().enumerate() {
                        let salt = 100 + i as u64 * 7;
                        if roll(salt) < 0.25 {
                            let n = 1 + (roll(salt + 1) * 3.0) as usize;
                            picks.push((format!("fx_grass_easter_var0{}", n.min(3)), (top + Vec2::Y * 0.5) * cell, 0.2));
                        }
                    }
                    for (i, &at) in brick.cells.iter().enumerate() {
                        let salt = 300 + i as u64 * 7;
                        if roll(salt) < 0.4 {
                            let n = 1 + (roll(salt + 1) * 8.0) as usize;
                            let kind = if roll(salt + 2) < 0.5 { "flower" } else { "egg" };
                            picks.push((format!("brick_easter_decorations_{kind}0{}", n.min(8)), at * cell, 0.3));
                        }
                    }
                }
                for (i, (name, at, z)) in picks.into_iter().enumerate() {
                    let salt = 500 + i as u64 * 11;
                    if let Some((sprite, anchor, transform)) = art_sprite(&art, &name, cell) {
                        let show_at = time.elapsed_secs() + roll(salt) * 0.3;
                        let turn = if roll(salt + 1) < 0.5 { -1.0 } else { 1.0 };
                        // The animations go on a holder for the decoration, never on the brick
                        // (they set its Transform, which would shrink and spin the brick's
                        // body); the picture keeps its own scale under the holder.
                        entity.with_children(|parent| {
                            let mut piece = parent.spawn((
                                Transform::from_translation(at.extend(z)),
                                Visibility::Hidden,
                                EdgeDelay { show_at },
                                EdgeGrow { start: show_at },
                            ));
                            piece.with_child((sprite, anchor, transform));
                            // Flowers turn and eggs pulse after a random 5 to 15 s.
                            if name.contains("flower") {
                                piece.insert(FlowerTurn {
                                    start: show_at + 5.0 + roll(salt + 2) * 10.0,
                                    turn,
                                });
                            } else if name.contains("egg") {
                                piece.insert(EggPulse {
                                    start: show_at + 5.0 + roll(salt + 2) * 10.0,
                                    x_first: roll(salt + 3) < 0.5,
                                });
                            }
                        });
                    }
                }
            }
            None => {
                entity.with_children(|parent| cell_sprites(parent, &brick.cells, cell, color));
            }
        }
    }
}

/// The cells with nothing above them in the same brick: where edge decorations sit.
fn top_cells(cells: &[Vec2]) -> Vec<Vec2> {
    cells
        .iter()
        .copied()
        .filter(|&c| !cells.iter().any(|&o| (o - c - Vec2::Y).length() < 0.1))
        .collect()
}

/// A joined Ivy brick's vines show when its delay is up: the brick is drawn again with them,
/// so their growth animation starts then.
fn show_vines(
    mut commands: Commands,
    time: Res<Time>,
    mut waiting: Query<(Entity, &crate::spells::VineDelay, &mut Brick)>,
) {
    let now = time.elapsed_secs();
    for (entity, delay, mut brick) in &mut waiting {
        if now >= delay.0 {
            commands.entity(entity).remove::<crate::spells::VineDelay>();
            brick.set_changed();
        }
    }
}

/// A tile of the race finish line.
#[derive(Component)]
struct FinishTile;

/// Custom/FinishLine's vertex wave: each tile rises and falls by 1% of the view's half height
/// along sin(2 x + 2 t), x being its place across the view (-1 to 1).
fn wave_finish_tiles(
    time: Res<Time>,
    view: Res<ViewSize>,
    camera: Query<(&GlobalTransform, &Projection), With<MainCamera>>,
    mut tiles: Query<(&mut Transform, &GlobalTransform), With<FinishTile>>,
) {
    let Ok((eye, Projection::Orthographic(ortho))) = camera.single() else {
        return;
    };
    let half = view.0 * 0.5 * ortho.scale;
    let t = time.elapsed_secs();
    for (mut transform, at) in &mut tiles {
        let x = (at.translation().x - eye.translation().x) / half.x.max(1.0);
        transform.translation.y = 0.01 * half.y * (2.0 * x + 2.0 * t).sin();
    }
}

/// Draws a Minibase tower piece with its own picture (TowerPieceL/R01 or 02's Graphic).
fn dress_tower_pieces(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    pieces: Query<(Entity, &crate::spells::TowerPiece), Added<crate::spells::TowerPiece>>,
) {
    for (entity, piece) in &pieces {
        let Some(shape) = config.magic.minibase_pieces.get(piece.0) else { continue };
        let Some((sprite, anchor, transform)) = art_sprite(&art, &shape.picture, config.cell_size) else {
            continue;
        };
        let at = Vec2::new(shape.graphic.0, shape.graphic.1) * config.cell_size;
        commands.entity(entity).insert(Visibility::default()).with_child((
            sprite,
            anchor,
            transform.with_translation(at.extend(0.0)),
        ));
    }
}

/// The bubble drawn around a brick lifted by the Bubble spell.
#[derive(Component)]
struct BubbleSprite;

#[allow(clippy::type_complexity)]
fn dress_bubbles(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<(Entity, &Brick, Option<&Children>), Or<(With<Bubbled>, With<crate::spells::BubbleFail>)>>,
    mut popped: RemovedComponents<Bubbled>,
    mut failed: RemovedComponents<crate::spells::BubbleFail>,
    children: Query<&Children>,
    bubbles: Query<(), With<BubbleSprite>>,
) {
    // Every bubbled brick wears its bubble; it comes back if the brick's pictures were
    // redrawn (they take all its children with them).
    for (entity, brick, kids) in &bricks {
        if kids.is_some_and(|k| k.iter().any(|c| bubbles.contains(c))) {
            continue;
        }
        let max = brick
            .cells
            .iter()
            .fold(0.0f32, |a, b| a.max(b.abs().max_element()));
        let size = (max + 1.0) * 2.0 * config.cell_size * brick.scale;
        // The original has bubbles for bricks 2, 3 and 4 cells across, in 3 looks each.
        let across = ((max * 2.0 + 1.0) * brick.scale).round().clamp(2.0, 4.0) as u32;
        let look = entity.index_u32() % 3 + 1;
        let Some(image) = [
            format!("bubble_{across}_{look}"),
            format!("bubble_{across}"),
            "bubble".into(),
        ]
        .iter()
        .find_map(|name| art.image(name)) else {
            continue;
        };
        commands.entity(entity).with_child((
            BubbleSprite,
            Sprite {
                image,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
        ));
    }
    // A bubble that goes pops (SFX_BUBBLE_POP).
    for entity in popped.read().chain(failed.read()) {
        if bricks.contains(entity) {
            continue;
        }
        let mut had = false;
        for child in children.iter_descendants(entity) {
            if bubbles.contains(child) {
                commands.entity(child).despawn();
                had = true;
            }
        }
        if had && let Some(sound) = art.sounds.get("sfx_bubble_pop") {
            commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
}

/// A bubble as BubbleBrickModifier draws it: it wobbles (scale + (sin(20t) + 1) x 0.025 across,
/// + (sin(10t) + 1) x 0.025 up), turns 50 degrees its brick's way over its 3 s, and from 2 s
/// flashes, out to +0.2 and back every 1/6 s.
fn wobble_bubbles(
    mut bubbles: Query<(&mut Transform, &ChildOf), With<BubbleSprite>>,
    bricks: Query<&crate::spells::Bubbled>,
) {
    for (mut transform, parent) in &mut bubbles {
        let Ok(bubble) = bricks.get(parent.parent()) else {
            continue;
        };
        let t = bubble.age;
        let scale = if t < 2.0 {
            Vec2::new(1.0 + ((t * 20.0).sin() + 1.0) * 0.025, 1.0 + ((t * 10.0).sin() + 1.0) * 0.025)
        } else {
            let phase = ((t - 2.0) * 6.0) % 2.0;
            let k = if phase < 1.0 { phase } else { 2.0 - phase };
            Vec2::splat(1.0 + 0.2 * (1.0 - (1.0 - k) * (1.0 - k)))
        };
        transform.scale = scale.extend(1.0);
        // The brick under it turns too; the bubble's own turn is what is left of its 50.
        let brick_turn = if t < 1.0 {
            bubble.tilt * EaseFunction::ElasticOut.sample_clamped(t)
        } else {
            let k = ((t - 1.0) / 2.0).min(1.0);
            bubble.tilt + 20.0 * bubble.dir() * k * k
        };
        let own = 50.0 * bubble.dir() * (t / 3.0).min(1.0);
        transform.rotation = Quat::from_rotation_z((own - brick_turn).to_radians());
    }
}

/// The quad a zapped brick is drawn with while it dissolves.
#[derive(Component)]
struct ZapQuad;

/// A zapped brick (Undo, or the Puzzle laser) is drawn white in its outline and eaten away
/// with a coloured edge as its zap runs (TowerPartLaserEffect); its own look is hidden, and
/// one of the two zap sounds plays at 0.95 to 1.05 pitch.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn dissolve_zapped(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    started: Query<(Entity, &Brick, &Zapped, Option<&Children>), Added<Zapped>>,
    running: Query<(&Zapped, &Children)>,
    quads: Query<&MeshMaterial2d<crate::brick_material::ZapBrick>, With<ZapQuad>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut zaps: Option<ResMut<Assets<crate::brick_material::ZapBrick>>>,
) {
    let (Some(meshes), Some(zaps)) = (meshes.as_mut(), zaps.as_mut()) else {
        return;
    };
    for (entity, brick, zap, children) in &started {
        // The brick or its pieces may go in the same frame (redrawn, or the zap ends).
        for &child in children.into_iter().flatten() {
            commands.entity(child).try_insert(Visibility::Hidden);
        }
        let def = piece_def(&config, brick.piece);
        let key = if brick.piece < config.pieces.len() {
            format!("brick_{}", def.name)
        } else {
            format!("mystery_{}", def.name.to_lowercase())
        };
        let mask = art
            .entries
            .get(&key)
            .and_then(|e| e.bevel.as_ref())
            .and_then(|b| art.image(b))
            .or_else(|| art.image(&key));
        if let (Some(mask), Some(noise)) = (mask, art.image("brick_zap_noise")) {
            let cell = config.cell_size * brick.scale;
            let min = brick.cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
            let max = brick.cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
            let size = (max - min + Vec2::ONE) * cell;
            let [r, g, b] = zap.edge;
            let material = zaps.add(crate::brick_material::ZapBrick {
                look: crate::brick_material::ZapLook {
                    edge: Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0),
                    params: Vec4::ZERO,
                    tint: Vec4::ONE,
                },
                mask,
                noise,
            });
            let quad = (
                ZapQuad,
                Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
                MeshMaterial2d(material),
                Transform::from_translation(((min + max) * 0.5 * cell).extend(0.05)),
            );
            commands
                .entity(entity)
                .queue_silenced(move |mut brick: EntityWorldMut| {
                    brick.with_child(quad);
                });
        }
        let pick = (entity.to_bits() % 2) as usize;
        let name = ["sfx_cast_light", "sfx_cast_lightning_level_02"][pick];
        if let Some(sound) = art.sounds.get(name) {
            let pitch = 0.95 + 0.1 * ((entity.to_bits() / 2 % 100) as f32 / 100.0);
            commands.spawn((
                AudioPlayer(sound.clone()),
                PlaybackSettings::DESPAWN.with_speed(pitch),
            ));
        }
    }
    for (zap, children) in &running {
        let cutoff = (1.0 - zap.timer / zap.length.max(0.01)).clamp(0.0, 1.0);
        for &child in children {
            if let Ok(material) = quads.get(child)
                && let Some(mut look) = zaps.get_mut(&material.0)
            {
                look.look.params.x = cutoff;
            }
        }
    }
}

/// Draws Extra Life's heart orb.
fn dress_life_orbs(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    orbs: Query<Entity, Added<LifeOrb>>,
) {
    for entity in &orbs {
        let mut orb = commands.entity(entity);
        orb.insert(Visibility::default());
        // The Orb prefab: a thick ring and two sun ray layers turning opposite ways at half
        // size, the light spell slot and the Extra Life icon at 0.9 (SpellOrb).
        let c = config.cell_size;
        let parts: Vec<_> = [
            ("cast_sunrays", 0.5 * 0.7575, 0.572, 60.0, 0.0),
            ("cast_sunrays", 0.5, 0.472, -70.0, 0.1),
            ("cast_circle_thick", 0.5, 1.0, 0.0, 0.2),
            ("spell_slot_light", 0.9, 1.0, 0.0, 0.3),
            ("spell_icon_ExtraLife", 0.9, 1.0, 0.0, 0.4),
        ]
        .into_iter()
        .filter_map(|(name, size, alpha, spin, z)| {
            let (mut sprite, anchor, mut t) = art_sprite(&art, name, c)?;
            sprite.color = Color::WHITE.with_alpha(alpha);
            t.scale *= size;
            t.translation.z = z;
            Some((sprite, anchor, t, OrbSpin(spin)))
        })
        .collect();
        if !parts.is_empty() {
            orb.with_children(|o| {
                for part in parts {
                    o.spawn(part);
                }
            });
            continue;
        }
        match art_sprite(&art, "life_orb", config.cell_size) {
            Some(parts) => {
                orb.with_child(parts);
            }
            None => {
                orb.insert(Sprite::from_color(
                    Color::srgb(1.0, 0.3, 0.4),
                    Vec2::splat(config.cell_size * 0.8),
                ));
            }
        }
    }
}

/// A picture on the Extra Life orb that turns (RotationBehaviour), degrees a second.
#[derive(Component)]
struct OrbSpin(f32);

fn spin_orbs(time: Res<Time>, mut spins: Query<(&OrbSpin, &mut Transform)>) {
    for (spin, mut t) in &mut spins {
        if spin.0 != 0.0 {
            t.rotate_z((spin.0 * time.delta_secs()).to_radians());
        }
    }
}

/// A background layer that fills the view (manifest entries without `position`).
#[derive(Component)]
struct Background;

/// A background layer placed in a tower's lane (manifest entries with `position`).
/// `parallax` 1 moves with the tower; 0 stays at the same height on screen.
#[derive(Component)]
struct LaneLayer {
    offset: Vec2,
    parallax: f32,
}

/// The name a mode's art goes by in the manifest. Endless Race is dressed as a race.
pub(crate) fn mode_key(mode: ModeKind) -> &'static str {
    match mode.look() {
        ModeKind::Survival => "survival",
        ModeKind::Puzzle => "puzzle",
        _ => "race",
    }
}

/// Background layer names for this mode, back to front: `background_<mode>_N`
/// when staged for the mode, else `background_N`.
/// The look a mode swaps to on Pro ("Special"): Race Pro's night, Puzzle Pro's eclipse;
/// Endless Puzzle always has its red eclipse.
pub(crate) fn pro_look(
    mode: ModeKind,
    difficulty: Option<crate::config::Difficulty>,
) -> Option<&'static str> {
    // Endless Puzzle plays under its own red eclipse.
    if mode == ModeKind::EndlessPuzzle {
        return Some("endless_puzzle");
    }
    if difficulty != Some(crate::config::Difficulty::Pro) {
        return None;
    }
    match mode {
        ModeKind::Race => Some("race_pro"),
        ModeKind::Puzzle => Some("puzzle_pro"),
        _ => None,
    }
}

fn background_names(art: &Art, mode: ModeKind) -> Vec<String> {
    background_names_for(art, mode, None)
}

fn background_names_for(art: &Art, mode: ModeKind, pro: Option<&str>) -> Vec<String> {
    let endless = mode == ModeKind::Endless;
    let mode = mode_key(mode);
    let layers = |prefix: &str| {
        let mut found: Vec<(f32, u32, String)> = art
            .entries
            .iter()
            .filter(|(name, _)| art.images.raw().contains_key(*name))
            .filter_map(|(name, entry)| {
                let n = name.strip_prefix(prefix)?.parse::<u32>().ok()?;
                Some((entry.z.unwrap_or(n as f32), n, name.clone()))
            })
            .collect();
        found.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        found.into_iter().map(|(.., name)| name).collect::<Vec<_>>()
    };
    // Endless Race has its own versions of the race islands, and Pro its own look,
    // when staged.
    let own = Some(layers("background_endless_"))
        .filter(|l| endless && !l.is_empty())
        .or_else(|| {
            pro.map(|p| layers(&format!("background_{p}_")))
                .filter(|l| !l.is_empty())
        })
        .unwrap_or_else(|| layers(&format!("background_{mode}_")));
    if own.is_empty() {
        layers("background_")
    } else {
        own
    }
}

fn layer_z(art: &Art, name: &str, order: usize) -> f32 {
    let z = art
        .entries
        .get(name)
        .and_then(|e| e.z)
        .unwrap_or(order as f32);
    -900.0 + z
}

/// The Puzzle trial pillar's base (PUZZLE_LASER_PILAR): it stands on the view's bottom edge.
#[derive(Component)]
struct PillarBase;

fn place_pillar_bases(
    view_size: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), With<MainCamera>>,
    mut bases: Query<(&mut Transform, &ChildOf), (With<PillarBase>, Without<MainCamera>)>,
    parents: Query<&GlobalTransform>,
) {
    let Ok((camera, Projection::Orthographic(ortho))) = camera.single() else {
        return;
    };
    let bottom = camera.translation.y - view_size.0.y * ortho.scale * 0.5;
    for (mut transform, parent) in &mut bases {
        if let Ok(line) = parents.get(parent.parent()) {
            transform.translation.y = bottom - line.translation().y;
        }
    }
}

/// A background layer pinned to the screen (manifest entries with `screen`).
#[derive(Component, Clone)]
struct ScreenLayer {
    anchor: Vec2,
    position: Vec2,
    view_height: f32,
    pixels_per_unit: f32,
    scale: f32,
    parallax: f32,
    /// Sideways drift in screen units a second, and how far it has gone.
    drift: f32,
    drifted: f32,
    /// Stretched to the view's width instead of drawn at its size.
    stretch: bool,
    /// The drift before a curse wave sped it up.
    base_drift: Option<f32>,
    /// A cloud's width in units: it wraps from just past the right edge to just before the
    /// left (CloudBehaviour). 0 wraps round half a view either side.
    wrap: f32,
    /// How far it is lifted, in units (a bobbing island and its waterline).
    lift: f32,
    /// Its own stretch across and up, over `scale` (a waterline).
    squash: Vec2,
}

/// Whether the menus show the original's live scene behind them: every menu screen except the
/// pause menu and the explanation popup, which sit over a match (BackgroundController). While
/// it shows, the camera draws only the scene's layers, so a match left behind stays hidden.
#[derive(Resource, Default, PartialEq)]
pub(crate) struct MenuScene(pub bool);

/// How far the camera has climbed above the lowest it has been, in view units (41.54 a view):
/// background layers with parallax 1 (the desert sand) sink by it, as BackgroundLayer.Update.
#[derive(Resource, Default)]
pub(crate) struct ViewClimb(pub f32);

/// Whether the live Race scene is up (its waves run), and how far the camera has climbed.
#[derive(Resource, Default)]
struct SceneState {
    race: bool,
    /// Which of the Race family's looks is up ("race", "race_pro" or "endless").
    look: &'static str,
    climb: f32,
    wave_clock: f32,
    seed: u32,
}

/// The render layer only the scene's layers are on, shown alone while the menus are up.
const SCENE_LAYER: usize = 1;

fn track_menu_scene(
    mut commands: Commands,
    game: Res<Match>,
    screen: Option<Res<crate::menu::MenuScreen>>,
    mut scene: ResMut<MenuScene>,
    camera: Query<Entity, With<crate::visuals::MainCamera>>,
) {
    use crate::menu::MenuScreen;
    let on = game.phase == Phase::Menu
        && !screen.is_some_and(|s| matches!(*s, MenuScreen::Pause | MenuScreen::Explain));
    if scene.0 == on && !camera.is_empty() {
        scene.set_if_neq(MenuScene(on));
    }
    if scene.0 != on || scene.is_added() {
        scene.0 = on;
        for camera in &camera {
            let layer = if on { SCENE_LAYER } else { 0 };
            commands
                .entity(camera)
                .insert(bevy::camera::visibility::RenderLayers::layer(layer));
        }
    }
}

/// Puts every scene layer on both render layers, so it shows in matches and behind the menus.
#[allow(clippy::type_complexity)]
fn show_scene_in_menus(
    mut commands: Commands,
    layers: Query<
        Entity,
        (
            Or<(
                Added<ScreenLayer>,
                Added<Sky>,
                Added<WaterBand>,
                Added<Background>,
                Added<LitLayer>,
                Added<SceneWave>,
                Added<IslandQuad>,
            )>,
            Without<bevy::camera::visibility::RenderLayers>,
        ),
    >,
) {
    for entity in &layers {
        commands
            .entity(entity)
            .insert(bevy::camera::visibility::RenderLayers::from_layers(&[0, SCENE_LAYER]));
    }
}

/// A bobbing island (BobbingIsland): it moves to a random 0.2 to 0.4 units above or below
/// where it sits over 2.5 to 4 s, in-out quad, then the other way, starting either way.
#[derive(Component)]
struct IslandBob {
    from: f32,
    to: f32,
    length: f32,
    time: f32,
    seed: u32,
}

/// A waterline that moves with its island.
#[derive(Component)]
struct BobWith(Entity);

/// Water whose picture is to be drawn with the rippling top (Custom/FlagWave) once it has
/// loaded: the strip under the lanes, and the menus' foreground water.
#[derive(Component)]
struct RippleWater;

/// Replaces each rippling water's picture with a quad of the same size drawn by the water
/// material; the sprite stays, empty, to keep its place and tint.
#[allow(clippy::type_complexity)]
fn ripple_water(
    mut commands: Commands,
    config: Res<GameConfig>,
    view_size: Res<ViewSize>,
    images: Res<Assets<Image>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut waters: Option<ResMut<Assets<crate::scene_material::FlagWave>>>,
    mut pending: Query<
        (
            Entity,
            &mut Sprite,
            &Transform,
            Option<&Anchor>,
            Option<&Strip>,
            Option<&bevy::camera::visibility::RenderLayers>,
        ),
        With<RippleWater>,
    >,
) {
    let (Some(meshes), Some(waters)) = (meshes.as_mut(), waters.as_mut()) else {
        return;
    };
    let scene = &config.race_scene;
    for (entity, mut sprite, transform, anchor, strip, layers) in &mut pending {
        let Some(size) = images.get(&sprite.image).map(|i| i.size_f32()) else {
            continue;
        };
        // A strip is as wide as it reaches; a screen layer is stretched to the view.
        let (width, units) = match strip {
            Some(strip) => {
                let scale = transform.scale.x.max(f32::EPSILON);
                (strip.width / scale, strip.width / config.cell_size.max(0.01))
            }
            None => (
                size.x,
                view_size.0.x / view_size.0.y.max(1.0) * config.hud.camera_height,
            ),
        };
        let (amplitude, speed, up) = scene.water_ripple;
        let material = waters.add(crate::scene_material::FlagWave {
            look: crate::scene_material::WaterLook {
                params: Vec4::new(amplitude, speed, up, units / scene.water_span.max(0.01)),
                tint: Vec4::from_array(sprite.color.to_linear().to_f32_array()),
                extra: Vec4::ZERO,
            },
            picture: sprite.image.clone(),
        });
        let quad = Vec2::new(width, size.y);
        let offset = -anchor.map_or(Vec2::ZERO, |a| a.0) * quad;
        let mut child = commands.spawn((
            Mesh2d(meshes.add(Rectangle::new(quad.x, quad.y))),
            MeshMaterial2d(material),
            Transform::from_translation(offset.extend(0.0)),
            ChildOf(entity),
        ));
        if let Some(layers) = layers {
            child.insert(layers.clone());
        }
        sprite.custom_size = Some(Vec2::ZERO);
        commands.entity(entity).remove::<RippleWater>();
    }
}

/// An island's picture drawn with its rippling waterline, under its screen layer: its
/// height in units (to turn the bob into a share of the picture).
#[derive(Component)]
struct IslandQuad {
    height: f32,
}

/// Hands each island its bob (so the waterline stays put while the island moves, as
/// BobbingIsland's _BobAmount does) and its layer's tint.
fn wave_islands(
    layers: Query<(&ScreenLayer, &Sprite)>,
    quads: Query<(&MeshMaterial2d<crate::scene_material::IslandWave>, &IslandQuad, &ChildOf)>,
    mut materials: Option<ResMut<Assets<crate::scene_material::IslandWave>>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    for (material, quad, of) in &quads {
        let Ok((layer, sprite)) = layers.get(of.parent()) else {
            continue;
        };
        let bob = -layer.lift / quad.height.max(0.01);
        let tint = Vec4::from_array(sprite.color.to_linear().to_f32_array());
        if materials
            .get(&material.0)
            .is_some_and(|m| (m.look.params.z - bob).abs() > 1e-5 || m.look.tint != tint)
            && let Some(mut island) = materials.get_mut(&material.0)
        {
            island.look.params.z = bob;
            island.look.tint = tint;
        }
    }
}

fn bob_islands(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut islands: Query<(Entity, &mut IslandBob, &mut ScreenLayer), Without<BobWith>>,
    mut lines: Query<(&BobWith, &mut ScreenLayer), Without<IslandBob>>,
    mut lifts: Local<Vec<(Entity, f32)>>,
) {
    let scene = &config.race_scene;
    lifts.clear();
    for (entity, mut bob, mut layer) in &mut islands {
        bob.time += time.delta_secs();
        if bob.time >= bob.length {
            let mut seed = bob.seed;
            let mut roll = || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed % 10_000) as f32 / 10_000.0
            };
            let up = if bob.length == 0.0 { roll() < 0.5 } else { bob.to < 0.0 };
            let distance = scene.bob.0 + (scene.bob.1 - scene.bob.0) * roll();
            let length = scene.bob_time.0 + (scene.bob_time.1 - scene.bob_time.0) * roll();
            bob.seed = seed;
            bob.from = if bob.length == 0.0 { 0.0 } else { bob.to };
            bob.to = if up { distance } else { -distance };
            bob.length = length.max(0.1);
            bob.time = 0.0;
        }
        let x = (bob.time / bob.length).clamp(0.0, 1.0);
        let eased = if x < 0.5 { 2.0 * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(2) / 2.0 };
        layer.lift = bob.from + (bob.to - bob.from) * eased;
        lifts.push((entity, layer.lift));
    }
    for (with, mut layer) in &mut lines {
        if let Some((_, lift)) = lifts.iter().find(|(e, _)| *e == with.0) {
            layer.lift = *lift;
        }
    }
}

/// The reflected band over the water: its alpha swings round its own (IslandsLayer.Update).
#[derive(Component)]
struct BandPulse;

fn pulse_band(time: Res<Time>, config: Res<GameConfig>, mut bands: Query<&mut Sprite, With<BandPulse>>) {
    let scene = &config.race_scene;
    let alpha = scene.band_alpha
        + scene.band_swing * (time.elapsed_secs() * std::f32::consts::TAU / scene.band_period.max(0.01)).sin();
    for mut sprite in &mut bands {
        sprite.color.set_alpha(alpha);
    }
}

/// A wave crest (WavesLayer): where it was born (units from the view's bottom left), its
/// scale, its row's parallax and its age.
#[derive(Component)]
struct SceneWave {
    at: Vec2,
    scale: f32,
    parallax: f32,
    age: f32,
}

/// Spawns a crest in each row every `wave_every` seconds while the Race scene is up and moves
/// them: alpha up over half its life and down again, its height from `wave_squash.0` to `.1`
/// and back, drifting right by `wave_drift` times its scale (DOTween's default out-quad).
#[allow(clippy::too_many_arguments)]
fn run_waves(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    view_size: Res<ViewSize>,
    mut state: ResMut<SceneState>,
    camera: Query<(Entity, &Projection), With<crate::visuals::MainCamera>>,
    mut waves: Query<(Entity, &mut SceneWave, &mut Transform, &mut Sprite)>,
) {
    let scene = &config.race_scene;
    let Ok((camera, Projection::Orthographic(ortho))) = camera.single() else {
        return;
    };
    let view = view_size.0 * ortho.scale;
    let unit = view.y / config.hud.camera_height.max(0.01);
    let across = view.x / unit.max(0.0001);
    let ppu = art.entries.get(&scene.wave).and_then(|e| e.pixels_per_unit).unwrap_or(26.0);
    if state.race && let Some(image) = art.image(&scene.wave) {
        state.wave_clock += time.delta_secs();
        if state.wave_clock > scene.wave_every {
            state.wave_clock = 0.0;
            let mut seed = state.seed | 1;
            let mut roll = || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed % 10_000) as f32 / 10_000.0
            };
            for row in &scene.waves {
                let n = roll();
                let scale = row.scale.1 - n * (row.scale.1 - row.scale.0);
                let at = Vec2::new(roll() * across, row.y + n * row.height);
                let (r, g, b) = if state.look == "endless" { scene.wave_tint_endless } else { scene.wave_tint };
                let z = match (row.front, state.look) {
                    (true, _) => row.z,
                    (false, "race_pro") => -900.0 + scene.wave_pro_z,
                    _ => -900.0 + row.z,
                };
                commands.spawn((
                    SceneWave { at, scale, parallax: row.parallax, age: 0.0 },
                    Sprite { image: image.clone(), color: Color::srgba(r, g, b, 0.0), ..default() },
                    Anchor::BOTTOM_CENTER,
                    Transform::from_xyz(0.0, 0.0, z).with_scale(Vec3::ZERO),
                    ChildOf(camera),
                ));
            }
            state.seed = seed;
        }
    }
    let out_quad = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        1.0 - (1.0 - x) * (1.0 - x)
    };
    let half = scene.wave_time * 0.5;
    for (entity, mut wave, mut transform, mut sprite) in &mut waves {
        wave.age += time.delta_secs();
        let t = wave.age;
        if t >= scene.wave_time {
            commands.entity(entity).despawn();
            continue;
        }
        let (rise, fall) = (out_quad(t / half), out_quad((t - half) / half));
        let (alpha, squash) = if t < half {
            (rise, scene.wave_squash.0 + (scene.wave_squash.1 - scene.wave_squash.0) * rise)
        } else {
            (1.0 - fall, scene.wave_squash.1 - (scene.wave_squash.1 - scene.wave_squash.0) * fall)
        };
        sprite.color.set_alpha(alpha);
        let x = wave.at.x + scene.wave_drift * wave.scale * out_quad(t / scene.wave_time);
        transform.translation.x = (x - across * 0.5) * unit;
        transform.translation.y = -view.y * 0.5 + wave.at.y * unit - state.climb * wave.parallax * unit;
        let size = unit / ppu * wave.scale;
        transform.scale = Vec3::new(size, size * squash, 1.0);
    }
}

/// The brighter copy of a layer, shown as the lightning flashes.
#[derive(Component)]
struct LitLayer;

/// The sky gradient behind every other layer.
#[derive(Component)]
pub(crate) struct Sky;

/// The background scene's water band along the bottom of the screen.
#[derive(Component)]
struct WaterBand {
    height: f32,
    view_height: f32,
    parallax: f32,
}

/// A tall gradient image from `colors`, top to bottom.
pub(crate) fn gradient_image(colors: &[Color]) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    const ROWS: usize = 256;
    let mut data = Vec::with_capacity(ROWS * 4);
    for row in 0..ROWS {
        // Image rows run top to bottom, like `colors`.
        let t = row as f32 / (ROWS - 1) as f32 * (colors.len() - 1) as f32;
        let i = (t.floor() as usize).min(colors.len() - 1);
        let next = (i + 1).min(colors.len() - 1);
        let color = colors[i].mix(&colors[next], t - i as f32);
        data.extend_from_slice(&color.to_srgba().to_u8_array());
    }
    Image::new(
        Extent3d {
            width: 1,
            height: ROWS as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn spawn_backgrounds(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    menu_look: Res<MenuLook>,
    menu_scene: Res<MenuScene>,
    mut scene_state: ResMut<SceneState>,
    config: Res<GameConfig>,
    mut images: ResMut<Assets<Image>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut islands: Option<ResMut<Assets<crate::scene_material::IslandWave>>>,
    camera: Query<Entity, With<crate::visuals::MainCamera>>,
    existing: Query<
        Entity,
        Or<(
            With<Background>,
            With<ScreenLayer>,
            With<Sky>,
            With<WaterBand>,
            With<SceneWave>,
        )>,
    >,
    mut water: Query<(Entity, &mut Visibility, &mut Sprite, Option<&Children>), With<WaterStrip>>,
    mut shown: Local<Option<(ModeKind, Option<&'static str>, bool)>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    // Behind the menus the original shows the Race scene (BackgroundController), or the
    // picked Endless mode's on the Endless screen.
    let (mode, pro) = match (menu_look.0, menu_scene.0) {
        (Some((mode, special)), true) => (
            mode,
            pro_look(mode, special.then_some(crate::config::Difficulty::Pro)),
        ),
        (None, true) => (ModeKind::Race, None),
        _ => (game.mode, pro_look(game.mode, config.difficulty)),
    };
    let in_match = !menu_scene.0;
    if *shown == Some((mode, pro, in_match)) && !art.is_changed() {
        return;
    }
    *shown = Some((mode, pro, in_match));
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let mode_name = mode_key(mode);
    let race_scene = mode == ModeKind::Race && pro.is_none();
    // The looks that share the Race scene's islands, band and waves.
    let island_look = if race_scene {
        Some("race")
    } else if pro == Some("race_pro") {
        Some("race_pro")
    } else if mode == ModeKind::Endless {
        Some("endless")
    } else {
        None
    };
    scene_state.race = island_look.is_some();
    scene_state.look = island_look.unwrap_or("race");
    let scene = &config.race_scene;
    if race_scene {
        spawn_race_clouds(&mut commands, &art, &images, scene, camera);
    }
    // A Pro or Endless look's sky and water band win over the mode's, which win over the shared.
    // Endless Race has its own sunset sky.
    let endless = (mode == ModeKind::Endless).then_some("endless");
    let for_mode = |name: &str| {
        pro.or(endless)
            .and_then(|p| art.entries.get(&format!("{name}_{p}")))
            .or_else(|| art.entries.get(&format!("{name}_{mode_name}")))
            .or_else(|| art.entries.get(name))
    };
    // The readouts' sky colours stand in when the Pro sky isn't staged.
    let weather = &config.weather;
    let rgb = |c: &[[u8; 3]]| -> Vec<Color> {
        c.iter().map(|&[r, g, b]| Color::srgb_u8(r, g, b)).collect()
    };
    let pro_sky = match pro {
        Some("race_pro") => Some(rgb(&weather.race_pro_sky)),
        Some("puzzle_pro") => Some(rgb(&weather.puzzle_pro_sky)),
        _ => None,
    }
    .filter(|_| pro.is_some_and(|p| !art.entries.contains_key(&format!("sky_{p}"))));
    if let Some(colors) = pro_sky {
        commands.entity(camera).with_child((
            Sky,
            Sprite::from_image(images.add(gradient_image(&colors))),
            Transform::from_xyz(0.0, 0.0, -950.0),
        ));
    } else if let Some(sky) = for_mode("sky") {
        let colors = sky.colors();
        if !colors.is_empty() {
            let mut layer = commands.spawn((
                Sky,
                Sprite::from_image(images.add(gradient_image(&colors))),
                Transform::from_xyz(0.0, 0.0, -950.0),
                ChildOf(camera),
            ));
            if mode == ModeKind::EndlessPuzzle && game.phase != Phase::Menu {
                layer.insert(EclipseFade(Color::WHITE));
                if let Some(plain) = art.entries.get("sky_puzzle") {
                    commands.entity(camera).with_child((
                        Sky,
                        Sprite::from_image(images.add(gradient_image(&plain.colors()))),
                        Transform::from_xyz(0.0, 0.0, -950.5),
                    ));
                }
            }
        }
    }
    let band = for_mode("water_band");
    // Race Pro's water band from the readout when its own isn't staged.
    let pro_band = (pro == Some("race_pro") && !art.entries.contains_key("water_band_race_pro"))
        .then(|| {
            (
                rgb(&weather.race_pro_water),
                weather.race_pro_water_height,
                weather.race_pro_water_parallax,
                band.and_then(|b| b.view_height).unwrap_or(10.0),
            )
        });
    let water_off = pro_band.is_none() && band.is_some_and(|b| b.colors().is_empty());
    // Endless Race's strip is its own magenta water (RaceForeground's endless WaterLayer,
    // Max's Endless clip); the menus keep the blue one.
    let strip_art = if mode == ModeKind::Endless && in_match {
        art.image("bg_race_foreground_water_endless").or_else(|| art.image("water"))
    } else {
        art.image("water")
    };
    for (entity, mut visibility, mut sprite, children) in &mut water {
        visibility.set_if_neq(if water_off {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        if let Some(image) = strip_art.clone()
            && sprite.image != image
        {
            // The ripple quad is drawn again from the new picture.
            sprite.image = image;
            for child in children.into_iter().flatten() {
                commands.entity(*child).despawn();
            }
            commands.entity(entity).insert(RippleWater);
        }
    }
    if let Some((colors, height, parallax, view_height)) = pro_band {
        commands.entity(camera).with_child((
            WaterBand {
                height,
                view_height: view_height.max(0.01),
                parallax,
            },
            Sprite::from_image(images.add(gradient_image(&colors))),
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, 0.0, -900.5),
        ));
    } else if let Some(band) = band {
        let colors = band.colors();
        if !colors.is_empty() {
            commands.entity(camera).with_child((
                WaterBand {
                    height: band.position.map_or(6.0, |p| p.1),
                    view_height: band.view_height.unwrap_or(10.0).max(0.01),
                    parallax: band.parallax.unwrap_or(0.0),
                },
                Sprite::from_image(images.add(gradient_image(&colors))),
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(0.0, 0.0, -900.0 + band.z.unwrap_or(-0.5)),
            ));
        }
    }
    // In a match Endless Puzzle starts as plain Puzzle and its eclipse fades in over
    // the plain layers during the intro (`fade_eclipse`).
    let eclipse = mode == ModeKind::EndlessPuzzle && game.phase != Phase::Menu;
    // Behind the menus the foreground water runs along the view's bottom (RaceForeground's
    // WaterLayer) for the Race family (Race, Race Special, Endless Race: Max's Endless
    // screen clip shows it; Survival and Puzzle have none); in a match the water strip
    // under the lanes is it.
    if island_look.is_some()
        && menu_scene.0
        && let (Some(image), Some(entry)) = (art.image("water"), art.entries.get("water"))
    {
        let ppu = entry.pixels_per_unit.unwrap_or(26.0).max(1.0);
        let height = images.get(&image).map_or(53.0, |i| i.size_f32().y) / ppu;
        commands.spawn((
            ScreenLayer {
                anchor: Vec2::new(0.5, 0.0),
                position: Vec2::new(0.0, height * (1.0 - entry.pivot.map_or(0.0, |p| 1.0 - p.1))),
                view_height: config.hud.camera_height,
                pixels_per_unit: ppu,
                scale: 1.0,
                parallax: 1.0,
                drift: 0.0,
                drifted: 0.0,
                base_drift: None,
                stretch: true,
                wrap: 0.0,
                lift: 0.0,
                squash: Vec2::ONE,
            },
            WaveLook::Keep,
            RippleWater,
            Sprite::from_image(image),
            Anchor(Vec2::new(0.0, entry.pivot.map_or(0.0, |p| p.1 - 0.5))),
            Transform::from_xyz(0.0, 0.0, 30.0),
            bevy::camera::visibility::RenderLayers::layer(SCENE_LAYER),
            ChildOf(camera),
        ));
    }
    if let (Some(look), Some(image)) = (island_look, art.image(&scene.band)) {
        let ppu = art.entries.get(&scene.band).and_then(|e| e.pixels_per_unit).unwrap_or(26.0);
        let (r, g, b) = match look {
            "race_pro" => scene.band_tint_pro,
            "endless" => scene.band_tint_endless,
            _ => scene.band_tint,
        };
        commands.spawn((
            ScreenLayer {
                anchor: Vec2::new(0.5, 0.0),
                position: Vec2::new(0.0, scene.island_root),
                view_height: config.hud.camera_height,
                pixels_per_unit: ppu,
                scale: scene.band_scale,
                parallax: 0.3,
                drift: 0.0,
                drifted: 0.0,
                base_drift: None,
                stretch: true,
                wrap: 0.0,
                lift: 0.0,
                squash: Vec2::ONE,
            },
            BandPulse,
            WaveLook::Keep,
            Sprite { image, color: Color::srgba(r, g, b, scene.band_alpha), ..default() },
            Transform::from_xyz(0.0, 0.0, -900.0 + scene.band_z),
            ChildOf(camera),
        ));
    }
    let mut names: Vec<(String, bool)> = background_names_for(&art, mode, pro)
        .into_iter()
        .map(|n| (n, eclipse))
        .collect();
    if eclipse {
        names.extend(
            background_names_for(&art, ModeKind::Puzzle, None)
                .into_iter()
                .map(|n| (n, false)),
        );
    }
    for (order, (name, fades)) in names.iter().enumerate() {
        let name = name.as_str();
        let order = order % names.len().max(1);
        let Some(entry) = art.entries.get(name) else {
            continue;
        };
        if race_scene && scene.replaces.iter().any(|r| r == name) {
            continue;
        }
        let island = island_look.and_then(|look| {
            scene.islands.iter().find(|i| match look {
                "race_pro" => i.pro == name,
                "endless" => i.endless == name,
                _ => i.art == name,
            })
        });
        let endless_look = island_look == Some("endless");
        let (px, py) = entry.pivot.unwrap_or((0.5, 0.5));
        let tint = entry
            .tint
            .map_or(Color::WHITE, |(r, g, b)| Color::srgb(r / 255.0, g / 255.0, b / 255.0))
            .with_alpha(entry.opacity.unwrap_or(1.0));
        let base = (
            Sprite {
                image: art.images[name].clone(),
                color: tint,
                ..default()
            },
            Anchor(Vec2::new(px - 0.5, py - 0.5)),
            Transform::from_xyz(
                0.0,
                0.0,
                layer_z(&art, name, order) + if *fades { 0.005 } else { 0.0 },
            ),
        );
        match (entry.screen, entry.position) {
            (Some((ax, ay)), position) => {
                let (x, y) = position.unwrap_or_default();
                // Each drifting layer picks its speed in the range, by its order.
                let drift = entry.drift.map_or(0.0, |(lo, hi)| {
                    let roll = ((order as f32 + 1.0) * 0.618_034).fract();
                    lo + (hi - lo) * roll
                });
                let lit = entry
                    .lit
                    .as_ref()
                    .and_then(|name| art.image(name))
                    .map(|image| Sprite {
                        image,
                        color: Color::WHITE.with_alpha(0.0),
                        ..default()
                    });
                let anchor = base.1;
                let star = entry.star == Some(true);
                let screen_layer = ScreenLayer {
                    anchor: Vec2::new(ax, ay),
                    position: Vec2::new(x, y),
                    view_height: entry.view_height.unwrap_or(10.0).max(0.01),
                    pixels_per_unit: entry.pixels_per_unit.unwrap_or(100.0).max(1.0),
                    scale: entry.scale.unwrap_or(1.0),
                    parallax: entry.parallax.unwrap_or(0.0),
                    drift,
                    drifted: 0.0,
                    base_drift: None,
                    stretch: entry.stretch == Some(true),
                    wrap: 0.0,
                    lift: 0.0,
                    squash: Vec2::ONE,
                };
                // The Race scene's islands are drawn at its island scale round their roots,
                // with their waterlines, and most of them bob.
                let screen_layer = match island {
                    Some(_) => {
                        let k = scene.island_scale;
                        ScreenLayer {
                            position: Vec2::new(x * k, scene.island_root + (y - scene.island_root) * k),
                            scale: screen_layer.scale * k,
                            ..screen_layer
                        }
                    }
                    None => screen_layer,
                };
                let line = island.and_then(|i| {
                    let image = art.image(&scene.waterline)?;
                    let ppu = art.entries.get(&scene.waterline)?.pixels_per_unit.unwrap_or(26.0);
                    let k = scene.island_scale;
                    let (line_at, line_size) = i
                        .endless_line
                        .filter(|_| endless_look)
                        .unwrap_or((i.line_at, i.line_size));
                    let (r, g, b) = if endless_look { scene.endless_line_tint } else { i.line_tint };
                    Some((
                        ScreenLayer {
                            position: screen_layer.position + Vec2::new(line_at.0, line_at.1) * k,
                            pixels_per_unit: ppu,
                            squash: Vec2::new(line_size.0, line_size.1),
                            ..screen_layer.clone()
                        },
                        Sprite { image, color: Color::srgb(r, g, b), ..default() },
                        Transform::from_xyz(0.0, 0.0, -900.0 + i.line_z),
                    ))
                });
                let mut layer = commands.spawn((screen_layer, base, ChildOf(camera)));
                if island.is_some_and(|i| i.bobs) || scene.bob_also.iter().any(|b| b == name) {
                    layer.insert(IslandBob {
                        from: 0.0,
                        to: 0.0,
                        length: 0.0,
                        time: 0.0,
                        seed: 0x9E37_79B9 ^ (order as u32 + 1).wrapping_mul(2_654_435_761) ^ name.len() as u32,
                    });
                }
                // Its picture is drawn by the island material under the (empty) sprite, which
                // keeps the layer's place, tint and bob.
                if let (Some(island), Some(meshes), Some(islands)) =
                    (island, meshes.as_mut(), islands.as_mut())
                    && let Some(size) = images.get(&art.images[name]).map(|i| i.size_f32())
                {
                    let height = size.y / entry.pixels_per_unit.unwrap_or(26.0).max(1.0) * scene.island_scale;
                    let tint = Vec4::from_array(tint.to_linear().to_f32_array());
                    let material = islands.add(crate::scene_material::IslandWave {
                        look: crate::scene_material::IslandLook {
                            params: Vec4::new(island.wave.0, island.wave.1, 0.0, 0.0),
                            tint,
                        },
                        picture: art.images[name].clone(),
                    });
                    layer.insert(Sprite {
                        image: art.images[name].clone(),
                        custom_size: Some(Vec2::ZERO),
                        ..default()
                    });
                    layer.with_child((
                        IslandQuad { height },
                        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
                        MeshMaterial2d(material),
                        Transform::from_translation((-anchor.0 * size).extend(0.0)),
                    ));
                }
                let island_entity = layer.id();
                if let Some(line) = line {
                    let mut line_entity = commands.spawn((line, ChildOf(camera)));
                    if island.is_some_and(|i| i.bobs) {
                        line_entity.insert(BobWith(island_entity));
                    }
                }
                let mut layer = commands.entity(island_entity);
                if star {
                    layer.insert(Star::default());
                }
                if mode == ModeKind::Survival && name == "background_survival_0" {
                    layer.insert(Moon);
                }
                if *fades {
                    layer.insert(EclipseFade(tint));
                }
                // How a Survival curse wave changes it (SurvivalBackground): the starry
                // sky stays, tinted layers (the large clouds) darken, the rest grey.
                layer.insert(match (entry.star, entry.tint, entry.overlay) {
                    (Some(_), _, _) => WaveLook::Keep,
                    (None, _, Some(true)) => WaveLook::Overlay(tint),
                    (None, Some(_), _) => WaveLook::Tint(tint),
                    (None, None, _) => WaveLook::Grey,
                });
                if let Some(sprite) = lit {
                    layer.with_child((
                        LitLayer,
                        sprite,
                        anchor,
                        Transform::from_xyz(0.0, 0.0, 0.01),
                    ));
                }
            }
            // Drawn in each lane by `spawn_lane_layers`.
            (None, Some(_)) => {}
            (None, None) => {
                let mut layer = commands.spawn((Background, base, ChildOf(camera)));
                if *fades {
                    layer.insert(EclipseFade(tint));
                }
            }
        }
    }
}

/// The Race scene's cloud layers (CloudLayer): each round spreads every picture once across
/// the view, each in its own slice of the width, at a random height in its row's range (the
/// cloud's bottom there, as CloudBehaviour lifts it by half its height) and speed.
fn spawn_race_clouds(
    commands: &mut Commands,
    art: &Art,
    images: &Assets<Image>,
    scene: &crate::config::RaceScene,
    camera: Entity,
) {
    let mut seed = 0x2545_F491u32 ^ (scene.clouds.len() as u32);
    let mut roll = move || {
        seed ^= seed << 13;
        seed ^= seed >> 17;
        seed ^= seed << 5;
        (seed % 100_000) as f32 / 100_000.0
    };
    let view_height = 41.538_46;
    for row in &scene.clouds {
        let slice = 1.0 / row.art.len().max(1) as f32;
        for _ in 0..row.rounds {
            let mut left: Vec<&String> = row.art.iter().collect();
            let mut at = 0.0;
            while !left.is_empty() {
                let name = left.remove((roll() * left.len() as f32) as usize % left.len());
                let x = at + roll() * slice;
                at += slice;
                let y = row.y.0 + roll() * (row.y.1 - row.y.0);
                let speed = row.speed.0 + roll() * (row.speed.1 - row.speed.0);
                let (Some(image), Some(entry)) = (art.image(name), art.entries.get(name)) else {
                    continue;
                };
                let ppu = entry.pixels_per_unit.unwrap_or(26.0).max(1.0);
                let size = images.get(&image).map_or(Vec2::splat(260.0), |i| i.size_f32()) / ppu;
                commands.spawn((
                    ScreenLayer {
                        anchor: Vec2::new(x, y),
                        position: Vec2::new(0.0, size.y * 0.5),
                        view_height,
                        pixels_per_unit: ppu,
                        scale: 1.0,
                        parallax: row.parallax,
                        drift: speed,
                        drifted: 0.0,
                        base_drift: None,
                        stretch: false,
                        wrap: size.x,
                        lift: 0.0,
                        squash: Vec2::ONE,
                    },
                    WaveLook::Grey,
                    Sprite::from_image(image),
                    Transform::from_xyz(0.0, 0.0, -900.0 + row.z),
                    ChildOf(camera),
                ));
            }
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn fit_backgrounds(
    time: Res<Time>,
    view_size: Res<ViewSize>,
    images: Res<Assets<Image>>,
    camera: Query<(&Projection, &Transform), (With<crate::visuals::MainCamera>, Without<Background>)>,
    mut layers: Query<(&Sprite, &mut Transform), (With<Background>, Without<crate::visuals::MainCamera>)>,
    mut sky: Query<
        &mut Sprite,
        (
            With<Sky>,
            Without<Background>,
            Without<ScreenLayer>,
            Without<crate::visuals::MainCamera>,
        ),
    >,
    mut screen: Query<
        (
            &mut ScreenLayer,
            &mut Transform,
            Option<&mut Star>,
            &Sprite,
            Option<&mut Falling>,
        ),
        (Without<Background>, Without<crate::visuals::MainCamera>, Without<Sky>),
    >,
    mut bands: Query<
        (&WaterBand, &mut Sprite, &mut Transform),
        (
            Without<Background>,
            Without<crate::visuals::MainCamera>,
            Without<Sky>,
            Without<ScreenLayer>,
        ),
    >,
    mut lowest: Local<Option<f32>>,
    menu_scene: Res<MenuScene>,
    mut scene_state: ResMut<SceneState>,
    mut view_climb: ResMut<ViewClimb>,
    config: Res<GameConfig>,
    game: Res<Match>,
) {
    let Ok((Projection::Orthographic(ortho), camera)) = camera.single() else {
        return;
    };
    let window = view_size.0;
    let view = window * ortho.scale;
    for (sprite, mut transform) in &mut layers {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let size = image.size_f32();
        let cover = (window.x / size.x).max(window.y / size.y);
        transform.scale = Vec3::splat(cover * ortho.scale);
    }
    for mut sprite in &mut sky {
        sprite.custom_size = Some(view);
    }
    // How far the camera has climbed above the lowest it has been this match.
    // It counts from where the camera sits when play starts (BackgroundLayer keeps the
    // camera's first bottom): the menus and the intro's sweep don't count.
    if menu_scene.0 || !matches!(game.phase, Phase::Playing | Phase::Over) {
        *lowest = None;
    }
    let low = lowest.get_or_insert(camera.translation.y);
    *low = low.min(camera.translation.y);
    // Behind the menus the scene sits as at a match's start.
    let climb = if menu_scene.0 {
        0.0
    } else {
        camera.translation.y - *low
    };
    scene_state.climb = climb / (view.y / config.hud.camera_height.max(0.01)).max(0.0001);
    view_climb.0 = scene_state.climb;
    for (mut layer, mut transform, mut star, sprite, falling) in &mut screen {
        let unit = view.y / layer.view_height;
        let across = view.x / unit;
        let slide = if layer.wrap > 0.0 {
            // A cloud leaves past the right edge and comes back from just before the left.
            layer.drifted += layer.drift * time.delta_secs();
            let start = layer.anchor.x * across + layer.position.x;
            let x = (start + layer.drifted + layer.wrap * 0.5).rem_euclid(across + layer.wrap)
                - layer.wrap * 0.5;
            x - start
        } else {
            // A drifting layer wraps round the view with half a view to spare each side.
            let span = across * 2.0;
            layer.drifted =
                (layer.drifted + layer.drift * time.delta_secs()).rem_euclid(span.max(0.01));
            // A drifting layer starts where it was placed and wraps half a span either side.
            if layer.drift != 0.0 {
                (layer.drifted + span * 0.5).rem_euclid(span.max(0.01)) - span * 0.5
            } else {
                0.0
            }
        };
        let slide = Vec2::X * slide * unit;
        let at = (layer.anchor - Vec2::splat(0.5)) * view
            + (layer.position + Vec2::Y * layer.lift) * unit
            + slide
            - Vec2::Y * climb * layer.parallax;
        transform.translation.x = at.x;
        transform.translation.y = at.y;
        let twinkle = star.as_ref().map_or(1.0, |s| s.factor);
        if let Some(mut falling) = falling {
            let dt = time.delta_secs();
            let gravity = falling.gravity;
            let velocity = falling.velocity + Vec2::NEG_Y * gravity * dt;
            falling.velocity = velocity;
            falling.offset += velocity * dt;
            transform.translation += (falling.offset * unit).extend(0.0);
        }
        // A falling star drops under gravity (9.81 units/s²) from its kick.
        if let Some((offset, velocity)) = star.as_mut().and_then(|s| s.fall.as_mut()) {
            *velocity += Vec2::NEG_Y * 9.81 * time.delta_secs();
            *offset += *velocity * time.delta_secs();
            transform.translation += (*offset * unit).extend(0.0);
        }
        let height = unit / layer.pixels_per_unit * layer.scale * twinkle;
        transform.scale = if layer.stretch {
            let width = images
                .get(&sprite.image)
                .map_or(1.0, |i| i.size_f32().x.max(1.0));
            Vec3::new(view.x / width, height, 1.0)
        } else {
            Vec3::new(height * layer.squash.x, height * layer.squash.y, height)
        };
    }
    for (band, mut sprite, mut transform) in &mut bands {
        let unit = view.y / band.view_height;
        sprite.custom_size = Some(Vec2::new(view.x, band.height * unit));
        transform.translation.y = -view.y * 0.5 - climb * band.parallax;
    }
}

/// What a Survival curse wave does to a screen layer.
#[derive(Component, Clone, Copy)]
pub(crate) enum WaveLook {
    Keep,
    /// Tinted layers (the large clouds) shift from their tint to (56, 34, 105).
    Tint(Color),
    /// The reflected band shifts from its tint to the curse band colour.
    Overlay(Color),
    /// Islands and the small clouds grey; the small clouds also speed up.
    Grey,
}

/// Survival's curse waves grey the background layers (clouds and islands) and speed
/// up the drifting clouds, shifting with the sky over `wave_shift` (gap-scan.md 2).
#[allow(clippy::type_complexity)]
fn grey_wave_layers(
    config: Res<GameConfig>,
    game: Res<Match>,
    mix: Option<Res<crate::weather::WaveMix>>,
    mut layers: Query<
        (&mut Sprite, Option<&mut ScreenLayer>, Option<&WaveLook>),
        (Or<(With<ScreenLayer>, With<Background>)>, Without<LitLayer>),
    >,
) {
    if game.mode.look() != ModeKind::Survival {
        return;
    }
    let mix = mix.map_or(0.0, |m| m.0);
    let weather = &config.weather;
    let [r, g, b] = weather.survival_wave_grey;
    let grey = Color::WHITE.mix(&Color::srgb_u8(r, g, b), mix);
    for (mut sprite, layer, look) in &mut layers {
        let look = look.copied().unwrap_or(WaveLook::Grey);
        let want = match look {
            WaveLook::Keep => continue,
            WaveLook::Tint(base) => base.mix(&Color::srgb_u8(56, 34, 105), mix),
            WaveLook::Overlay(base) => {
                let [r, g, b] = weather.survival_wave_band;
                base.mix(&Color::srgb_u8(r, g, b), mix)
            }
            WaveLook::Grey => grey,
        };
        // An Endless Puzzle eclipse fade owns its alpha.
        let want = want.with_alpha(sprite.color.alpha());
        if sprite.color != want {
            sprite.color = want;
        }
        if let Some(mut layer) = layer.filter(|_| matches!(look, WaveLook::Grey)) {
            let current = layer.drift;
            let base = *layer.base_drift.get_or_insert(current);
            let drift = base * (1.0 + (weather.survival_wave_drift - 1.0) * mix);
            if layer.drift != drift {
                layer.drift = drift;
            }
        }
    }
}

/// Layers lit by the lightning show their bright picture as the flash fades.
fn light_layers(
    flash: Option<Res<crate::weather::Flash>>,
    mut lit: Query<&mut Sprite, With<LitLayer>>,
) {
    let alpha = flash.map_or(0.0, |f| f.0);
    for mut sprite in &mut lit {
        if sprite.color.alpha() != alpha {
            sprite.color.set_alpha(alpha);
        }
    }
}

/// Spawns the positioned background layers into a new platform's lane.
pub(crate) fn spawn_lane_layers(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    mode: ModeKind,
    cell: f32,
) {
    for (order, name) in background_names(art, mode).iter().enumerate() {
        let Some(entry) = art.entries.get(name) else {
            continue;
        };
        let Some((x, y)) = entry.position.filter(|_| entry.screen.is_none()) else {
            continue;
        };
        let Some((sprite, anchor, mut transform)) = art_sprite(art, name, cell) else {
            continue;
        };
        transform.scale *= entry.scale.unwrap_or(1.0);
        // The platform's centre is half a cell below its top, where positions start.
        let offset = Vec2::new(x, y + 0.5) * cell;
        transform.translation = offset.extend(layer_z(art, name, order));
        parent.spawn((
            LaneLayer {
                offset,
                parallax: entry.parallax.unwrap_or(1.0),
            },
            sprite,
            anchor,
            transform,
        ));
    }
}

/// Moves lane layers with less than full parallax towards the camera.
fn scroll_lane_layers(
    camera: Query<&Transform, (With<crate::visuals::MainCamera>, Without<LaneLayer>)>,
    platforms: Query<&GlobalTransform, With<Platform>>,
    mut layers: Query<(&LaneLayer, &ChildOf, &mut Transform)>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    for (layer, child_of, mut transform) in &mut layers {
        let Ok(platform) = platforms.get(child_of.parent()) else {
            continue;
        };
        // Vertical only: lanes stay side by side as the camera climbs.
        let lag = camera.translation.y - platform.translation().y;
        transform.translation.x = layer.offset.x;
        transform.translation.y = layer.offset.y + lag * (1.0 - layer.parallax);
    }
}

/// Redraws bricks and platforms when art drops out (an image failed to load).
fn redress_on_art_change(
    mut commands: Commands,
    art: Res<Art>,
    mut bricks: Query<&mut Brick>,
    platforms: Query<(Entity, &Platform)>,
) {
    if !art.is_changed() || art.is_added() {
        return;
    }
    for mut brick in &mut bricks {
        brick.set_changed();
    }
    for (entity, platform) in &platforms {
        let tower = platform.tower;
        commands
            .entity(entity)
            .despawn_children()
            .remove::<(Platform, Sprite)>()
            .insert(Platform { tower });
    }
}

/// A trial's own floor: its pictures (mirrored ones flipped about their pivot, like
/// the original), or one plain block per box when none is staged.
fn dress_trial_floor(
    base: &mut EntityCommands,
    art: &Art,
    floor: &crate::config::TrialFloor,
    c: f32,
) {
    let pictures: Vec<_> = floor
        .art
        .iter()
        .filter_map(|(key, x, y, flip, turn)| {
            let (sprite, anchor, mut transform) = art_sprite(art, key, c)?;
            transform.translation = Vec3::new(x * c, y * c + c * 0.5, 0.1);
            transform.rotation = Quat::from_rotation_z(turn.to_radians());
            if *flip {
                transform.scale.x = -transform.scale.x;
            }
            Some((sprite, anchor, transform))
        })
        .collect();
    if pictures.is_empty() {
        base.with_children(|parent| {
            for b in &floor.boxes {
                let size = Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) * c;
                let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * c;
                parent.spawn((
                    Sprite::from_color(Color::srgb(0.62, 0.52, 0.40), size),
                    Transform::from_translation((centre + Vec2::Y * c * 0.5).extend(0.0)),
                ));
            }
        });
    }
    for picture in pictures {
        base.with_child(picture);
    }
}
/// Draws each new platform and its goal line (race finish or puzzle laser).
fn dress_platforms(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    images: Res<Assets<Image>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut flags: Option<ResMut<Assets<crate::scene_material::CheckpointFlag>>>,
    platforms: Query<(Entity, &Platform, &Transform), Added<Platform>>,
) {
    let c = config.cell_size;
    let art_name = format!("platform_{}", mode_key(game.mode));
    let art_name = art_name.as_str();
    for (entity, platform, placed) in &platforms {
        let mut base = commands.entity(entity);
        // In Puzzle the floor blocks stand in for the base, and a trial's own floor
        // may replace it.
        let trial = config.trial_floor();
        let floor = (game.mode.rules() == ModeKind::Puzzle
            && config.puzzle.floor != crate::config::PuzzleFloor::Plain)
            || trial.is_some_and(|f| !f.keep_base);
        match art_sprite(&art, art_name, c).filter(|_| !floor) {
            // A child, so the sprite's scale never touches the platform's collider.
            // Its pivot sits on the platform's top, half a cell above the centre.
            Some((sprite, anchor, mut transform)) => {
                transform.translation.y = c * 0.5;
                base.insert(Visibility::default())
                    .with_child((sprite, anchor, transform));
            }
            None if floor => {
                base.insert(Visibility::default());
            }
            // No art: one plain block per collider box.
            None => {
                base.insert(Visibility::default()).with_children(|parent| {
                    for b in config.base_boxes(game.mode) {
                        let size = Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) * c;
                        let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * c;
                        parent.spawn((
                            Sprite::from_color(Color::srgb(0.55, 0.50, 0.45), size),
                            Transform::from_translation((centre + Vec2::Y * c * 0.5).extend(0.0)),
                        ));
                    }
                });
            }
        }
        if let Some(trial) = trial {
            dress_trial_floor(&mut base, &art, trial, c);
        }
        base.with_children(|parent| {
            spawn_lane_layers(parent, &art, game.mode, c);
            let lane = config.cells(config.platform_width + 4.0);
            // Race lines run across the tower's share of the screen, as each tower's
            // own camera shows them in the original.
            let lanes = if config.trial.is_some() {
                2
            } else {
                ((config.players.max(1) + config.opponents) as usize)
                    .min(crate::gameplay::MAX_TOWERS)
            };
            // A lone tower's camera is the whole screen, so its lines reach both edges
            // (Max's Endless Race clip); shared screens keep the lanes' margins.
            let race_lane = if lanes == 1 {
                config.cells(config.lanes.screen_width)
            } else {
                config.cells(config.lanes.width(lanes))
            };
            // A Puzzle trial's laser runs from the screen's left edge to its ball,
            // 21 cells right of the lane (the ball's place in Max's screenshot; the
            // original sets it 9 units into its zoomed background view, ?).
            let ball_x = c * 21.0;
            let trial_laser = config.trial.is_some() && game.mode.rules() == ModeKind::Puzzle;
            // A race battle's finish line is one flag across the whole screen: each tower's
            // part meets its neighbour's halfway and the outer ones reach the screen's edges.
            let race_span = {
                let half_screen = config.cells(config.lanes.screen_width) * 0.5;
                let half_step = config.cells(config.lanes.spacing(lanes)) * 0.5;
                let outer = half_step * (lanes as f32 - 1.0);
                let x = placed.translation.x;
                let left = if x <= -outer + c { -half_screen - x } else { -half_step };
                let right = if x >= outer - c { half_screen - x } else { half_step };
                (right - left, (left + right) * 0.5)
            };
            let (line_width, line_x) = match game.mode.rules() {
                ModeKind::Race if lanes > 1 && config.trial.is_none() => race_span,
                ModeKind::Race | ModeKind::Endless => (race_lane, 0.0),
                ModeKind::Puzzle if trial_laser => (ball_x + c * 24.0, (ball_x - c * 24.0) * 0.5),
                // A battle's laser is one line across the screen: each tower's copy is wide
                // enough that together they cover it.
                ModeKind::Puzzle => (config.cells(200.0), 0.0),
                _ => (lane, 0.0),
            };
            let line = match game.mode.rules() {
                ModeKind::Race => Some((
                    config.race.finish_height,
                    "finish_line",
                    Color::srgba(1.0, 1.0, 1.0, 0.6),
                )),
                ModeKind::Puzzle => Some((
                    config.puzzle.line_height,
                    "laser",
                    Color::srgba(1.0, 0.2, 0.2, 0.8),
                )),
                // The pacer starts below the base and rises from there.
                ModeKind::Endless => Some((
                    config.endless.intro_from,
                    "pacer_line",
                    Color::srgba(1.0, 0.55, 0.1, 0.85),
                )),
                _ => None,
            };
            if let Some((height, name, color)) = line {
                // The platform's centre is half a cell below its top.
                let mut line = parent.spawn((
                    FinishLine {
                        tower: platform.tower,
                    },
                    Transform::from_xyz(0.0, config.cells(height) + c * 0.5, -1.0),
                    Visibility::default(),
                ));
                // A battle's Puzzle laser rests as the plain (off) laser; the bright one
                // only sweeps in at the start (PuzzleMultiplayerLaserLayer).
                let battle_laser = name == "laser" && !trial_laser;
                let rest = if battle_laser && art.image("fx_laser_off").is_some() {
                    "fx_laser_off"
                } else {
                    name
                };
                let (off_h, on_h) = config.puzzle.laser_heights;
                if battle_laser && let Some(bright) = art_strip_tall(&art, name, c, line_width, config.cells(on_h), Vec3::ZERO) {
                    line.with_children(|line| {
                        line.spawn((
                            LaserSweep {
                                width: line_width,
                                tower: platform.tower,
                                flash: None,
                            },
                            Transform::from_xyz(0.0, 0.0, 0.2),
                            Visibility::Hidden,
                        ))
                        .with_child(bright);
                    });
                }
                let rest_strip = if battle_laser {
                    art_strip_tall(&art, rest, c, line_width, config.cells(off_h), Vec3::X * line_x)
                        .map(|strip| line.with_child((strip, LaserRest)).id())
                } else if trial_laser && name == "laser" {
                    // A Puzzle trial's laser is on all the time (PuzzleSinglePlayerLaserLayer).
                    art_strip_tall(&art, rest, c, line_width, config.cells(config.puzzle.trial_laser_height), Vec3::X * line_x)
                        .map(|strip| line.with_child(strip).id())
                } else if name == "finish_line" {
                    // FinishFlag: tiles 1.2 cells apart at 0.7 alpha, each one bobbing on the
                    // Custom/FinishLine wave (`wave_finish_tiles`).
                    let step = c * 1.2;
                    let count = (line_width / step).ceil().max(1.0) as i32;
                    let mut any = None;
                    for i in 0..count {
                        let x = line_x - line_width * 0.5 + step * (i as f32 + 0.5);
                        if let Some((mut sprite, anchor, transform)) = art_sprite(&art, rest, c) {
                            sprite.color = Color::WHITE.with_alpha(0.7);
                            any = Some(line.with_child((
                                sprite,
                                anchor,
                                transform.with_translation(Vec3::new(x, 0.0, 0.0)),
                                FinishTile,
                            )).id());
                        }
                    }
                    any
                } else {
                    art_strip(&art, rest, c, line_width, Vec3::X * line_x).map(|strip| line.with_child(strip).id())
                };
                if rest_strip.is_none() {
                    line.with_child((
                        Sprite::from_color(color, Vec2::new(line_width, 3.0)),
                        Transform::from_xyz(line_x, 0.0, 0.0),
                    ));
                }
                // The finish line's tiles end in a cap at each side, the right one turned
                // 180° (docs/research/trials-obstacles-roofs.md, "Finish line").
                // The flag's end caps sit 33.6 / 34 cells out, past a shared screen's lanes:
                // only a lone tower's view reaches them.
                if name == "finish_line" && lanes == 1 {
                    for side in [-1.0f32, 1.0] {
                        if let Some((mut sprite, anchor, transform)) =
                            art_sprite(&art, "finish_line_end", c)
                        {
                            sprite.flip_x = side > 0.0;
                            sprite.flip_y = side > 0.0;
                            let lane = line_width;
                            line.with_child((
                                sprite,
                                anchor,
                                transform.with_translation(Vec3::new(
                                    side * (lane * 0.5 - c * 0.7),
                                    0.0,
                                    0.5,
                                )),
                            ));
                        }
                    }
                }
                // A Puzzle trial's laser comes from a glowing ball on a pillar at its
                // right end, laid out as the original's PUZZLE_LASER prefab (in cells, from
                // the ball): the pink top 5.2 under it, six column tiles from 9.08 down every
                // 3.2, each picture at its own size; and at the view's bottom the
                // PUZZLE_LASER_PILAR base of grassy floor columns and two cacti (`PillarBase`).
                if name == "laser"
                    && config.trial.is_some()
                    && art.image("fx_laserball").is_some()
                {
                    let x = ball_x;
                    let piece = |key: &str, at: Vec2, z: f32, flip: bool, l: &mut EntityCommands| {
                        if let Some((sprite, anchor, mut transform)) = art_sprite(&art, key, c) {
                            transform.translation = Vec3::new(x + at.x * c, at.y * c, z);
                            if flip {
                                transform.scale.x = -transform.scale.x;
                            }
                            l.with_child((sprite, anchor, transform));
                        }
                    };
                    for y in [-9.08, -12.28, -15.48, -18.66, -21.86, -25.06] {
                        piece("fx_laser_pillar_tile", Vec2::new(0.0, y), -0.2, false, &mut line);
                    }
                    piece("fx_laser_pillar_top", Vec2::new(0.0, -5.2), -0.1, false, &mut line);
                    piece("fx_laserball", Vec2::ZERO, 0.6, false, &mut line);
                    let mut parts = Vec::new();
                    for (key, at, z, flip) in [
                        ("bg_race_cactus_right", Vec2::new(1.6, 9.21), -0.1, false),
                        ("bg_race_cactus_left", Vec2::new(-1.53, 9.04), -0.1, false),
                        ("puzzle_piece_2", Vec2::new(0.57, 8.82), 0.0, false),
                        ("puzzle_piece_1", Vec2::new(-1.16, 8.53), 0.0, true),
                        ("puzzle_piece_2", Vec2::new(1.48, 8.02), 0.1, true),
                        ("puzzle_piece_1", Vec2::new(-0.54, 6.62), 0.1, true),
                    ] {
                        if let Some((sprite, anchor, mut transform)) = art_sprite(&art, key, c) {
                            transform.translation = Vec3::new(at.x * c, at.y * c, z);
                            if flip {
                                transform.scale.x = -transform.scale.x;
                            }
                            parts.push((sprite, anchor, transform));
                        }
                    }
                    let base = line
                        .commands()
                        .spawn((PillarBase, Transform::from_xyz(x, 0.0, -0.3), Visibility::default()))
                        .with_children(|b| {
                            for part in parts {
                                b.spawn(part);
                            }
                        })
                        .id();
                    line.add_child(base);
                }
                // A race trial's finish line carries the time left beside an hourglass
                // (gap-scan.md 4), fading in over the first second.
                let clock = config
                    .trial
                    .and_then(|i| config.trials.get(i))
                    .and_then(|t| t.seconds)
                    .filter(|_| game.mode.rules() == ModeKind::Race);
                if clock.is_some() {
                    // Centred on the screen (the trial's player lane sits left of it),
                    // the hourglass 5.3 cells left of the time, both 0.45 over the line.
                    let centre = config.cells(config.lanes.spacing(2)) * 0.5;
                    let glass = match art.image("hourglass") {
                        Some(image) => Sprite {
                            image,
                            custom_size: Some(Vec2::new(c * 1.0, c * 1.75)),
                            color: Color::WHITE.with_alpha(0.0),
                            ..default()
                        },
                        None => Sprite::from_color(
                            Color::srgba(1.0, 0.85, 0.4, 0.0),
                            Vec2::splat(c * 0.8),
                        ),
                    };
                    line.with_children(|l| {
                        l.spawn((
                            TrialClock(0.0),
                            Sprite::from_color(
                                Color::srgba(0.94, 0.9, 0.6, 0.0),
                                Vec2::new(config.cells(config.lanes.screen_width + 10.0), c * 0.12),
                            ),
                            Transform::from_xyz(centre, 0.0, 0.9),
                        ));
                        l.spawn((
                            TrialClock(c * 0.45),
                            glass,
                            Transform::from_xyz(centre - c * 5.3, c * 0.45, 1.0),
                        ));
                        l.spawn((
                            TrialClock(c * 0.45),
                            ClockAnchor,
                            Transform::from_xyz(centre, c * 0.45, 1.0),
                        ));
                    });
                }
            }
            if config.magic.enabled {
                // A row of diamonds across the lane, all one colour per line
                // (`update_lines` picks it); no caps at the ends.
                let look = &config.hud;
                // SpellContainerSpawner makes every line 16 pieces wide, one a unit
                // (LineSpellContainer puts piece i at i + 0.5).
                let width = config.cells(SPELL_LINE_PIECES as f32 * look.diamond_step);
                let sprite = art_sprite(&art, "spell_line", c);
                let count = art
                    .entries
                    .get("spell_line")
                    .and_then(|e| e.count)
                    .filter(|_| sprite.is_some())
                    .unwrap_or(SPELL_LINE_PIECES)
                    .max(1);
                let mut line = parent.spawn((
                    SpellLine {
                        tower: platform.tower,
                    },
                    Transform::from_xyz(0.0, 0.0, -1.5),
                    Visibility::Hidden,
                ));
                // Endless Race's lines are checkpoints with a flag at each end
                // (EndlessRaceLineSpellContainer): the right one is the left turned round, its
                // flag turned over and set 1.12 out.
                let scene = &config.race_scene;
                let piece = |name: &str| {
                    let image = art.image(name)?;
                    let entry = art.entries.get(name)?;
                    let size = images.get(&image)?.size_f32();
                    let (px, py) = entry.pivot.unwrap_or((0.5, 0.5));
                    Some((image, size, Vec2::new(px - 0.5, py - 0.5), c / entry.pixels_per_unit.unwrap_or(26.0)))
                };
                if game.mode == ModeKind::Endless
                    && let (Some(pole), Some(flag), Some(meshes), Some(flags)) = (
                        piece("checkpoint_flag_pole"),
                        piece("checkpoint_flag"),
                        meshes.as_mut(),
                        flags.as_mut(),
                    )
                {
                    let (aw, fq, sp) = scene.flag_wave;
                    for side in [-1.0f32, 1.0] {
                        let (pole_image, _, pole_anchor, pole_scale) = pole.clone();
                        let (flag_image, flag_size, flag_anchor, flag_scale) = flag.clone();
                        let material = flags.add(crate::scene_material::CheckpointFlag {
                            params: Vec4::new(aw, fq, sp, 0.0),
                            picture: flag_image,
                        });
                        let mesh = meshes.add(Rectangle::new(flag_size.x, flag_size.y));
                        let (at, turn) = if side < 0.0 {
                            (Vec2::new(scene.flag_at.0, scene.flag_at.1), 0.0)
                        } else {
                            (Vec2::new(scene.flag_right_x, scene.flag_at.1), std::f32::consts::PI)
                        };
                        line.with_children(|line| {
                            line.spawn((
                                Transform::from_xyz(side * width * 0.5, 0.0, 0.2)
                                    .with_scale(Vec3::new(-side, 1.0, 1.0)),
                                Visibility::Inherited,
                            ))
                            .with_children(|root| {
                                root.spawn((
                                    Sprite::from_image(pole_image),
                                    Anchor(pole_anchor),
                                    Transform::from_scale(Vec3::splat(pole_scale)),
                                ));
                                root.spawn((
                                    Transform::from_translation((at * c).extend(0.01))
                                        .with_rotation(Quat::from_rotation_z(turn))
                                        .with_scale(Vec3::splat(flag_scale)),
                                    Visibility::Inherited,
                                ))
                                .with_child((
                                    Mesh2d(mesh),
                                    MeshMaterial2d(material),
                                    Transform::from_translation((-flag_anchor * flag_size).extend(0.0)),
                                ));
                            });
                        });
                    }
                }
                line.with_children(|line| {
                    for i in 0..count {
                        let x = width * ((i as f32 + 0.5) / count as f32 - 0.5);
                        match &sprite {
                            Some((sprite, anchor, transform)) => {
                                line.spawn((
                                    SpellDiamond,
                                    sprite.clone(),
                                    *anchor,
                                    transform.with_translation(Vec3::new(x, 0.0, 0.0)),
                                ));
                            }
                            None => {
                                let side = config.cells(look.diamond_size)
                                    * std::f32::consts::FRAC_1_SQRT_2;
                                line.spawn((
                                    SpellDiamond,
                                    Sprite::from_color(Color::WHITE, Vec2::splat(side)),
                                    Transform::from_xyz(x, 0.0, 0.0).with_rotation(
                                        Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                                    ),
                                ));
                            }
                        }
                    }
                });
            }
        });
    }
}

/// A column the width of each falling brick, from the top of the view down to its bottom
/// or the guide's lowest point (BrickGuide), behind the bricks. With the guide material it
/// is laid over the background in an overlay blend; otherwise it is a plain tinted sprite.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn update_drop_guide(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    view: Res<ViewSize>,
    grab: Option<Res<crate::post::GrabTarget>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<crate::post::GuideOverlay>>>,
    camera: Query<(&Transform, &Projection), (With<crate::visuals::MainCamera>, Without<DropGuide>)>,
    towers: Query<(Entity, &TowerState), With<Tower>>,
    bricks: Query<(&ActiveBrick, &ColliderAabb)>,
    mut guides: Query<(
        Entity,
        &DropGuide,
        &mut Transform,
        &mut Visibility,
        Option<&MeshMaterial2d<crate::post::GuideOverlay>>,
    )>,
) {
    for (entity, guide, ..) in &guides {
        if !towers.contains(guide.tower) {
            commands.entity(entity).despawn();
        }
    }
    let look = &config.hud;
    let (mut tint, below) = match game.mode {
        ModeKind::Endless => (look.guide_endless, look.guide_below_race),
        mode => match mode.look() {
            ModeKind::Survival => (look.guide_survival, look.guide_below_survival),
            ModeKind::Puzzle => (look.guide_puzzle, config.camera_floor()),
            _ => (look.guide_race, look.guide_below_race),
        },
    };
    if game.mode.rules() == ModeKind::Puzzle && config.trial.is_some() {
        tint[3] = look.guide_trial_alpha;
    }
    let tint = Vec4::from_array(tint);
    for (tower, _) in &towers {
        if guides.iter().any(|(_, g, ..)| g.tower == tower) {
            continue;
        }
        let place = (DropGuide { tower }, Transform::from_xyz(0.0, 0.0, -2.0), Visibility::Hidden);
        match (meshes.as_mut(), materials.as_mut(), grab.as_ref()) {
            (Some(meshes), Some(materials), Some(grab)) => {
                commands.spawn((
                    place,
                    Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                    MeshMaterial2d(materials.add(crate::post::GuideOverlay {
                        tint,
                        grab: grab.0.clone(),
                    })),
                ));
            }
            _ => {
                let colour = Color::srgba(tint.x, tint.y, tint.z, tint.w * 0.4);
                commands.spawn((place, Sprite::from_color(colour, Vec2::ONE)));
            }
        }
    }
    let Ok((eye, projection)) = camera.single() else {
        return;
    };
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    let half = view.0.y * 0.5 * scale;
    let top = eye.translation.y + half;
    let bottom = (eye.translation.y - half).max(-config.cells(below));
    for (_, guide, mut transform, mut visibility, material) in &mut guides {
        let active = towers
            .get(guide.tower)
            .ok()
            .filter(|_| game.phase == Phase::Playing)
            .and_then(|(_, state)| state.active)
            .and_then(|entity| bricks.get(entity).ok());
        let Some((_, aabb)) = active else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        if let (Some(material), Some(materials)) = (material, materials.as_mut())
            && materials.get(&material.0).is_some_and(|m| m.tint != tint)
            && let Some(mut m) = materials.get_mut(&material.0)
        {
            m.tint = tint;
        }
        let width = aabb.max.x - aabb.min.x;
        transform.translation =
            Vec3::new((aabb.min.x + aabb.max.x) * 0.5, (top + bottom) * 0.5, -2.0);
        transform.scale = Vec3::new(width, (top - bottom).max(1.0), 1.0);
        visibility.set_if_neq(Visibility::Visible);
    }
}

/// Keeps every tower, from the platforms to the top of the highest one, in view.
fn frame_towers(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    view_size: Res<ViewSize>,
    towers: Query<(&Tower, &TowerState)>,
    rival_lanes: Query<&crate::gameplay::RivalLane>,
    lift: Option<Res<crate::intro::CameraLift>>,
    mut camera: Query<(&mut Transform, &mut Projection), With<crate::visuals::MainCamera>>,
) {
    let lift = lift.map_or(0.0, |l| l.0);
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    let window = view_size.0;
    if towers.is_empty() {
        return;
    }
    let (mut min_x, mut max_x) = (f32::MAX, f32::MIN);
    let (mut lowest, mut highest, mut widest) = (f32::MAX, f32::MIN, 0.0f32);
    for (tower, state) in &towers {
        min_x = min_x.min(tower.origin_x);
        max_x = max_x.max(tower.origin_x);
        lowest = lowest.min(state.stack_top);
        highest = highest.max(state.stack_top);
        widest = widest.max(state.half_width * 2.0);
    }
    let mut lanes = towers.iter().count();
    for lane in &rival_lanes {
        min_x = min_x.min(lane.origin_x);
        max_x = max_x.max(lane.origin_x);
        lanes += 1;
    }
    // The original gives each tower its own camera over its share of the screen,
    // all with the same top and bottom; one camera over the screen-split lanes
    // does the same. Cells under the base top it may show: the floor's height.
    let floor = -config.camera_floor();
    let look = &config.hud;
    let (top, bottom) = if game.mode.rules() == ModeKind::Puzzle {
        // WidthTowerCamera: the bottom stays on the floor, the height fits the
        // widest tower (+2, at least 10) across a lane, at least 40 cells. A
        // Puzzle trial's lane is two thirds of the screen.
        let share = if config.trial.is_some() {
            2.0 / 3.0
        } else {
            1.0 / lanes.max(1) as f32
        };
        let aspect = window.x * share / window.y.max(1.0);
        let height = ((widest + 2.0).max(10.0) / aspect.max(0.01)).max(40.0);
        (floor + height, floor)
    } else {
        // SingleTowerCamera.
        let height = look.camera_height;
        let mut top = (highest + height - look.camera_tower_room).max(floor + height);
        let mut bottom = (lowest - look.camera_below_lowest).max(floor);
        bottom = bottom.min(top - height);
        if game.phase == Phase::Over {
            // Once the match is over it pulls back until every base fits.
            bottom = floor;
        }
        top = top.max(bottom + height);
        // The race intro's sweep lifts both edges together.
        (top + lift, bottom + lift)
    };
    let height = config.cells(top - bottom);
    let scale = (height / window.y.max(1.0)).max(0.01);
    // Trials split the screen around its centre; otherwise centre the lanes.
    let x = if config.trial.is_some() {
        0.0
    } else {
        (min_x + max_x) * 0.5
    };
    let target = Vec2::new(x, config.cells(bottom) + height * 0.5);

    // The original eases top and bottom by 5 × dt each frame; the intro's sweep
    // sets them directly.
    let t = if lift > 0.0 {
        1.0
    } else {
        1.0 - (-5.0 * time.delta_secs()).exp()
    };
    let current = transform.translation.truncate();
    transform.translation += ((target - current) * t).extend(0.0);
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale += (scale - ortho.scale) * t;
    }
}

/// The debug readout in the top-left corner (height, spells, who won). The
/// original has none, so it stays empty until F3 turns it on.
fn update_hud(
    config: Res<GameConfig>,
    game: Res<Match>,
    log: Res<SpellLog>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    towers: Query<(&Tower, &TowerState)>,
    mut hud: Query<&mut Text, With<Hud>>,
    mut debug: Local<bool>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
    // F3 shows a text readout (height, spells, result) for development.
    if keys.is_some_and(|k| k.just_pressed(KeyCode::F3)) {
        *debug = !*debug;
    }
    if !*debug {
        if !text.0.is_empty() {
            text.0.clear();
        }
        return;
    }
    let mut sorted: Vec<_> = towers.iter().collect();
    sorted.sort_by_key(|(t, _)| t.index);
    let Some((_, you)) = sorted.iter().find(|(t, _)| t.is_first_player()) else {
        text.0.clear();
        return;
    };

    let goal = match game.mode {
        ModeKind::Race => {
            let hold = if you.hold_timer > 0.0 {
                format!(
                    "  hold {:.1}/{:.0}s",
                    you.hold_timer, config.race.hold_seconds
                )
            } else {
                String::new()
            };
            format!(
                "RACE  height {:.1} / {:.0}{hold}",
                you.height, game.finish_height
            )
        }
        ModeKind::Survival => format!(
            "SURVIVAL  bricks {} / {}",
            you.bricks_used(),
            config.survival.bricks_to_place,
        ),
        ModeKind::Puzzle => format!(
            "PUZZLE  bricks {} / {}  stay under the laser ({:.1})",
            you.bricks_used(),
            config.puzzle.bricks,
            you.laser(&config)
        ),
        ModeKind::Endless => format!(
            "ENDLESS  score {}  best {}  pacer {:.1}",
            config.endless.score(you.height),
            config.endless.score(you.best_height),
            you.pacer
        ),
        ModeKind::EndlessSurvival => format!(
            "ENDLESS SURVIVAL  score {}  bricks {}",
            you.endless_score(),
            you.bricks_placed,
        ),
        ModeKind::EndlessPuzzle => format!(
            "ENDLESS PUZZLE  score {}  stay under the laser ({:.1})",
            you.endless_score(),
            you.laser(&config)
        ),
    };
    let spell = if config.magic.enabled {
        you.spell_line
            .map(|h| format!("spell line at {h:.1}"))
            .unwrap_or_default()
    } else {
        String::new()
    };
    let mist = if you.mist_timer > 0.0 { "   MIST!" } else { "" };
    let mut lines = vec![
        goal,
        format!("dropped {}   {spell}{mist}", you.bricks_dropped),
    ];

    let name = |tower: &Tower| match tower.player {
        Some(p) => format!("Player {}", p + 1),
        None => format!("Wizard {}", tower.index),
    };
    let names: Vec<String> = sorted.iter().map(|(t, _)| name(t)).collect();
    if let Some(last) = log.0.last() {
        let who = |i: usize| {
            if i == 0 {
                "You".to_string()
            } else if i == crate::spells::ENEMY_WIZARD {
                "The enemy wizard".to_string()
            } else {
                names.get(i).cloned().unwrap_or_default()
            }
        };
        let on = if last.caster == last.target {
            String::new()
        } else {
            format!(" on {}", who(last.target).to_lowercase())
        };
        lines.push(format!(
            "{} cast {}{on}",
            who(last.caster),
            last.spell.name()
        ));
    }
    if game.phase == Phase::Over {
        lines.push(match game.winner {
            Some(0) => "YOU WIN! Press R to play again".into(),
            Some(i) => format!(
                "{} wins. Press R to try again",
                names.get(i).cloned().unwrap_or_default()
            ),
            None => match game.mode {
                ModeKind::Puzzle => "PUZZLE FAILED. Press R to try again".into(),
                ModeKind::Endless => format!(
                    "CAUGHT! Score {}. Press R to try again",
                    config.endless.score(you.best_height)
                ),
                ModeKind::EndlessSurvival | ModeKind::EndlessPuzzle => format!(
                    "RUN OVER! Score {}. Press R to try again",
                    you.endless_score()
                ),
                _ => "OUT OF LIVES. Press R to try again".into(),
            },
        });
    }
    text.0 = lines.join("\n");
}

/// Moves each tower's spell line and the race finish line to where they are now.
/// Counts the race trial's seconds down on the finish line, fading it in.
#[allow(clippy::type_complexity)]
fn show_trial_clock(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    view: Res<ViewSize>,
    camera: Query<(&Camera, &GlobalTransform, &Projection), With<MainCamera>>,
    mut sprites: Query<(&TrialClock, &mut Sprite, &mut Transform)>,
    mut anchors: Query<(&TrialClock, &mut Transform, &GlobalTransform), (With<ClockAnchor>, Without<Sprite>)>,
    mut labels: Query<
        (Entity, &mut Text, &mut TextFont, &mut crate::letters::TextLook, &mut Node, &ComputedNode),
        With<ClockLabel>,
    >,
) {
    let seconds = config
        .trial
        .and_then(|i| config.trials.get(i))
        .and_then(|t| t.seconds);
    // The label goes with its anchor (a new match or leaving the trial).
    if anchors.is_empty() || seconds.is_none() {
        for (entity, ..) in &labels {
            commands.entity(entity).despawn();
        }
    } else if labels.is_empty() {
        commands.spawn((
            ClockLabel,
            Text::new(""),
            TextFont::default(),
            TextLayout::justify(Justify::Center),
            crate::letters::TextLook::default(),
            Node {
                position_type: PositionType::Absolute,
                ..default()
            },
            GlobalZIndex(4),
        ));
    }
    let Some(seconds) = seconds else {
        return;
    };
    let alpha = game.elapsed.clamp(0.0, 1.0);
    // The line climbs from the base (finish × time used / time) and sits on the
    // finish when the time is up; the clock is a child of the finish line.
    let used = (game.elapsed / seconds.max(0.01)).clamp(0.0, 1.0);
    let drop = config.cells(config.race.finish_height * (used - 1.0));
    // Minutes, seconds and hundredths, as the original's clock reads (0:58:44).
    let hundredths = ((seconds - game.elapsed).max(0.0) * 100.0) as u32;
    let words = format!(
        "{}:{:02}:{:02}",
        hundredths / 6000,
        hundredths / 100 % 60,
        hundredths % 100
    );
    for (clock, mut sprite, mut transform) in &mut sprites {
        sprite.color.set_alpha(alpha);
        transform.translation.y = drop + clock.0;
    }
    let Ok((camera, eye, projection)) = camera.single() else {
        return;
    };
    let scale = match projection {
        Projection::Orthographic(o) => o.scale.max(0.001),
        _ => 1.0,
    };
    for (clock, mut transform, at) in &mut anchors {
        transform.translation.y = drop + clock.0;
        let Ok(spot) = camera.world_to_viewport(eye, at.translation()) else {
            continue;
        };
        for (_, mut text, mut font, mut look, mut node, computed) in &mut labels {
            if text.0 != words {
                text.0 = words.clone();
            }
            let size = FontSize::Px(config.cell_size * 1.75 / scale);
            if font.font_size != size {
                font.font_size = size;
            }
            let want = crate::letters::TextLook {
                face: Some(CLOCK_FACE.with_alpha(alpha)),
                outline: Some(CLOCK_OUTLINE.with_alpha(alpha)),
                outline_edge: 0.5 - 0.4 * 0.8333 / 2.0,
                ..default()
            };
            if *look != want {
                *look = want;
            }
            let half = computed.size() * computed.inverse_scale_factor() * 0.5;
            node.left = Val::Px((spot.x - half.x).clamp(0.0, view.0.x));
            node.top = Val::Px(spot.y - half.y);
        }
    }
}

fn update_lines(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<&TowerState>,
    art: Res<Art>,
    mut lines: Query<(&SpellLine, &Children, &mut Transform, &mut Visibility), Without<FinishLine>>,
    mut finish: Query<(&FinishLine, &mut Transform, &mut Visibility), Without<SpellLine>>,
    mut diamonds: Query<(&mut Sprite, &Transform), (With<SpellDiamond>, Without<SpellLine>, Without<FinishLine>)>,
    time: Res<Time>,
) {
    let c = config.cell_size;
    // Once a match is decided, the finish line and spell lines go at once.
    let over = game.phase == Phase::Over && game.mode.rules() != ModeKind::Puzzle;
    let _ = &art;
    let colors: Vec<Color> = config
        .hud
        .line_colors
        .iter()
        .map(|&[r, g, b]| Color::srgba_u8(r, g, b, 192))
        .collect();
    // How far the colours have run (a colour a step).
    let run = time.elapsed_secs() / config.hud.line_color_seconds.max(0.01);
    for (line, mut transform, mut visibility) in &mut finish {
        visibility.set_if_neq(if over {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        let height = match game.mode.rules() {
            ModeKind::Puzzle => towers
                .get(line.tower)
                .map_or(config.puzzle.line_height, |s| s.laser(&config)),
            ModeKind::Endless => towers
                .get(line.tower)
                .map_or(config.endless.intro_from, |s| s.pacer),
            _ => game.finish_height,
        };
        // The line is a child of its platform, which rises with Puzzle drops: take that rise
        // back out so the line stays where the shared laser is.
        let rise = towers.get(line.tower).map_or(0.0, |s| s.base_shown) * c;
        transform.translation.y = config.cells(height) + c * 0.5 - rise;
    }
    for (line, children, mut transform, mut visibility) in &mut lines {
        let state = towers.get(line.tower).ok();
        let height = state.and_then(|s| s.spell_line).filter(|_| !over);
        if !colors.is_empty() {
            let n = colors.len();
            for child in children {
                if let Ok((mut sprite, at)) = diamonds.get_mut(*child) {
                    // The cell this diamond sits in, from the line's left end.
                    let cell = (at.translation.x / c).floor();
                    let v = cell / n as f32 + run;
                    let i = v.floor().rem_euclid(n as f32) as usize;
                    let color = colors[i].mix(&colors[(i + 1) % n], v.fract());
                    if sprite.color != color {
                        sprite.color = color;
                    }
                }
            }
        }
        // Lines right under the finish are hidden, like the original.
        let near_finish = game.mode == ModeKind::Race
            && height.is_some_and(|h| h > game.finish_height - config.magic.line_hide_near_finish);
        match height {
            Some(h) if !near_finish => {
                transform.translation.y = config.cells(h) + c * 0.5;
                visibility.set_if_neq(Visibility::Inherited);
            }
            _ => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
}

/// Mist (SpawnMistEffect): each cast lays a bank at the tower's height, or 5 units over the
/// highest bank already there; banks stay where they were laid and puff out their fog.
#[allow(clippy::too_many_arguments)]
fn update_mist(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    time: Res<Time>,
    art: Res<Art>,
    towers: Query<&TowerState>,
    platforms: Query<(Entity, &Platform, Option<&Children>)>,
    mut banks: Query<(Entity, &mut MistBank, &Transform)>,
    mut puffs: Query<(Entity, &ChildOf, &mut MistPuff, &mut Sprite, &mut Transform), Without<MistBank>>,
    mut last: Local<std::collections::HashMap<Entity, f32>>,
    mut seed: Local<u32>,
) {
    let dt = time.delta_secs();
    let mut roll = || {
        *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) | 1;
        (*seed >> 8) as f32 / (1u32 << 24) as f32
    };
    for (platform, owner, children) in &platforms {
        let Ok(state) = towers.get(owner.tower) else {
            continue;
        };
        let before = last.insert(owner.tower, state.mist_timer).unwrap_or(0.0);
        if state.mist_timer <= before + 0.5 {
            continue;
        }
        let highest = children
            .into_iter()
            .flatten()
            .filter_map(|&c| banks.get(c).ok())
            .map(|(_, _, t)| t.translation.y)
            .fold(f32::MIN, f32::max);
        let y = (config.cells(state.height) + config.cell_size * 0.5).max(highest + config.cells(MIST_HEIGHT));
        commands.spawn((
            MistBank { age: 0.0, fade: 1.0, due: 0.0 },
            Transform::from_xyz(0.0, y, 5.0),
            Visibility::default(),
            ChildOf(platform),
        ));
        if let Some(sound) = art.sounds.get("sfx_spawn_mist") {
            commands.spawn((AudioPlayer::new(sound.clone()), PlaybackSettings::DESPAWN));
        }
    }
    let ended = game.phase != Phase::Playing;
    let image = art.image("fx_gradient_reflected_soft").unwrap_or_default();
    for (bank, mut state, _) in &mut banks {
        state.age += dt;
        if ended {
            state.fade = (state.fade - dt).max(0.0);
        }
        if state.age > MIST_EMIT + MIST_LIFE || state.fade <= 0.0 {
            commands.entity(bank).despawn();
            continue;
        }
        while state.due <= state.age.min(MIST_EMIT) {
            state.due += 1.0 / MIST_RATE;
            let (a, b) = (MIST_COLOURS[0], MIST_COLOURS[1]);
            let t = roll();
            let colour = Color::srgb(a.0 + (b.0 - a.0) * t, a.1 + (b.1 - a.1) * t, a.2 + (b.2 - a.2) * t);
            let dy = (roll() - 0.5) * MIST_HEIGHT;
            commands.spawn((
                MistPuff { age: 0.0, colour },
                Sprite {
                    image: image.clone(),
                    color: colour.with_alpha(0.0),
                    custom_size: Some(Vec2::new(
                        config.cells(MIST_SIZE * MIST_LENGTH),
                        config.cells(MIST_SIZE),
                    )),
                    ..default()
                },
                Anchor::CENTER_RIGHT,
                Transform::from_xyz(config.cells(MIST_HEAD_X), config.cells(dy), roll() * 0.1),
                ChildOf(bank),
            ));
        }
    }
    for (puff, parent, mut state, mut sprite, mut transform) in &mut puffs {
        let Ok((_, bank, _)) = banks.get(parent.parent()) else {
            continue;
        };
        state.age += dt;
        if state.age >= MIST_LIFE {
            commands.entity(puff).despawn();
            continue;
        }
        transform.translation.x += config.cells(MIST_SPEED) * dt;
        let life = state.age / MIST_LIFE;
        let alpha = MIST_ALPHA
            .windows(2)
            .find(|k| life <= k[1].0)
            .map_or(0.0, |k| k[0].1 + (k[1].1 - k[0].1) * (life - k[0].0) / (k[1].0 - k[0].0));
        sprite.color = state.colour.with_alpha(alpha * bank.fade);
    }
}

#[cfg(test)]
mod tests {
    use super::top_cells;
    use bevy::prelude::Vec2;

    #[test]
    fn top_cells_are_the_ones_with_nothing_above() {
        // An L: a column of three with a foot to the right at the bottom.
        let l = [
            Vec2::new(0.0, 0.0),
            Vec2::new(0.0, 1.0),
            Vec2::new(0.0, 2.0),
            Vec2::new(1.0, 0.0),
        ];
        let mut tops = top_cells(&l);
        tops.sort_by(|a, b| (a.x, a.y).partial_cmp(&(b.x, b.y)).unwrap());
        assert_eq!(tops, vec![Vec2::new(0.0, 2.0), Vec2::new(1.0, 0.0)]);
    }
}
