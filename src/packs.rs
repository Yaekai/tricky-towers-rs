//! What each brick pack adds on top of its textures (docs/research/skins.md):
//! Christmas snow caps on top edges, Halloween decorations
//! that pulse, the Hologram glitch and see-through touch, the Diamond gloss and the
//! Galaxy starfield.
//! The bricks themselves are drawn by `visuals::dress_bricks`; this runs right after
//! it and adds children, so a re-dressed brick gets its decorations again.
//! Spring's grass and flowers are in visuals.rs (dress_bricks); its eggs aren't staged yet.
//! Looks only: nothing here touches gameplay.

use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;

use crate::art::Art;
use crate::brick_material::{DiamondBrick, DiamondLook, GalaxyBrick, GalaxyLook};
use crate::config::{BrickPack, GameConfig};
use crate::gameplay::{Brick, Placed, TowerSkin};
use crate::spells::{Grassy, Stone, Vined};
use crate::visuals::{CellSprite, dress_bricks};

pub struct PacksPlugin;

/// DEBUG-TOOL: whether the Galaxy starfield scrolls. On in the game; the debug command
/// `GalaxyFrame` switches it for a still picture of one piece.
#[derive(Resource)]
pub struct GalaxyDebug {
    pub scroll: bool,
}

impl Default for GalaxyDebug {
    fn default() -> Self {
        Self {
            scroll: true,
        }
    }
}

impl Plugin for PacksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GalaxyDebug>()
            .add_systems(Startup, make_holo_ramp);
        app.add_systems(
            Update,
            (
                decorate_bricks.after(dress_bricks),
                grow_snow,
                switch_lights,
                fade_halloween,
                pulse_decorations,
                glitch_holograms,
                scan_holograms,
                gloss_diamonds,
                turn_galaxy_bricks,
                place_galaxy_stars,
                twirl_stars,
            ),
        );
    }
}

/// A Christmas brick's lights: which frame of the two-frame picture shows, from the time and
/// the brick's own offset.
#[derive(Component)]
struct Lights {
    offset: f32,
}

/// A snow cap appearing within 0.3 s.
#[derive(Component)]
struct Snow {
    time: f32,
    full: Vec3,
}

/// A Halloween decoration that, every 5 to 15 s, either squashes one axis to 0.8× and
/// springs back (the other axis 0.1 s later) or wobbles 25° and springs back.
#[derive(Component)]
struct Pulse {
    wait: f32,
    time: f32,
    seed: u32,
    /// 0: wobble (±1 picks the side in `side`), 1: squash x first, 2: squash y first.
    kind: u8,
    side: f32,
}

/// A Hologram brick: glitches every 6 to 15 s, the first right away (when it spawns). A
/// glitch takes 0.1 to 0.5 s to come in and as long to go out (HologramBrickModifier, DOTween's
/// default OutQuad both ways), its distortion rising to 1 and falling back.
#[derive(Component)]
struct Glitch {
    wait: f32,
    left: f32,
    span: f32,
    seed: u32,
}

/// One cell's scan lines on a Hologram brick: the original's shader blends the base colour
/// towards (0.8, 1, 1) by k x frac(3 y + t), k = 0.2 + 0.025 (sin 50 t + 1), with y the world
/// height in cells. A ramp picture tiled up the cell stands in for frac().
#[derive(Component)]
struct HoloScan;

/// The scan-line ramp: alpha from 0 to 1 up one period, repeating.
#[derive(Resource)]
struct HoloRamp(Handle<Image>);

/// Texels in one scan-line period of `HoloRamp`.
const HOLO_RAMP: u32 = 32;

/// The original blends in gamma space and we blend in linear light, which makes the same
/// alpha read stronger here. This scales the scan lines' alpha down to match Max's Hologram
/// frame (a measured stand-in for the blending difference, not part of the shader).
const HOLO_SCAN_LINEAR: f32 = 0.35;

/// A Diamond brick's sweep (DiamondBrickModifier): every 6 to 15 s, the first right away, its
/// strength goes to 0.7 over 0.15 s (OutQuad), holds, and falls back over the last 0.15 s of
/// 0.6 s, while its angle turns a full circle, either way (InOutQuad over the 0.6 s). On the
/// brick's material quad.
#[derive(Component)]
struct Gloss {
    wait: f32,
    time: f32,
    seed: u32,
    turn: f32,
}

/// Each Diamond shape's two gloss colours (rgb, strength), from its material
/// (Material/BrickDiamond<shape>.mat).
const DIAMOND_GLOSS: [(&str, [f32; 4], [f32; 4]); 7] = [
    ("I", [1.0, 1.0, 1.0, 0.516], [0.985, 0.710, 0.773, 0.384]),
    ("J", [0.419, 0.857, 0.919, 0.522], [0.502, 0.690, 0.949, 0.522]),
    ("L", [0.882, 0.365, 0.200, 0.616], [1.0, 0.923, 0.375, 0.428]),
    ("O", [0.878, 0.663, 1.0, 0.741], [0.792, 0.318, 0.867, 0.522]),
    ("S", [0.431, 0.925, 0.655, 0.465], [0.806, 0.971, 0.881, 0.716]),
    ("T", [0.956, 0.893, 0.759, 0.766], [0.585, 1.0, 0.522, 0.603]),
    ("Z", [1.0, 0.294, 0.294, 0.341], [1.0, 1.0, 1.0, 0.559]),
];

/// A Galaxy brick's look, drawn by its own material (`brick_material::GalaxyBrick`).
#[derive(Component)]
struct GalaxyQuad;

/// A star on a Galaxy brick. Every 2 to 6 s one random star twirls: grows to 2× in
/// 0.3 s, turns 180° over 1 s (in-back), spins on from 600°/s slowing by 200°/s², and
/// shrinks back after 2 s. Otherwise it breathes 1 ± 0.3.
#[derive(Component)]
struct Star {
    /// Seconds into a twirl, if one is playing.
    twirl: Option<f32>,
    spin: f32,
}

/// One of the original's 49 stars, drawn in one cell of a Galaxy brick. It sits where the
/// shared field puts it inside that cell (`place_galaxy_stars`).
#[derive(Component)]
struct GalaxyStar {
    index: usize,
    /// The cell's position in its brick, in cells, and the cell's size in pixels.
    at: Vec2,
    px: f32,
}

/// When a Halloween brick came: its look fades from the base skin to the Halloween one from
/// then (HalloweenBrickModifier: 1 s after it spawns, over 1 s, InOutQuad).
#[derive(Component)]
struct HalloweenSince(f32);

/// The Halloween look over a brick's base skin, fading in (`fade_halloween`).
#[derive(Component)]
struct HalloweenFade(f32);


/// The original's starfield scene: its camera (orthographic size 5) renders 10 units into a
/// render texture, and the background sprite is 512 px at 51.2 px to the unit, so it fills that
/// view exactly. A brick cell is one unit (the brick sprites are 65 px to the unit).
const GALAXY_FIELD_CELLS: f32 = 10.0;
/// A star's picture is 18 px at 26 px to the unit in the original's scene: 0.69 units.
const GALAXY_STAR_SIZE: f32 = 18.0 / 26.0;
/// Each Galaxy shape turns the sky's hue by its own amount: the original's shader rotates the
/// sky colour by 2pi x (the brick sprite's red + 0.5) - pi, and the prefabs' sprite reds are
/// these (Resources/game/bricks/brickpack/galaxy/Brick<shape>.prefab).
const GALAXY_HUES: [(&str, f32); 7] = [
    ("I", 0.102_941_16),
    ("J", 0.125),
    ("L", 0.058_823_53),
    ("O", 0.029_411_79),
    ("S", 0.0),
    ("T", 0.051_470_58),
    ("Z", 0.147_058_84),
];

/// The 49 star positions of the original's Galaxy scene (Galaxy.prefab, the Stars parent),
/// in the scene's units from its centre.
const GALAXY_STARS: [(f32, f32); 49] = [
    (-1.12, -1.19), (-1.6, 2.78), (-3.89, -0.14), (0.42, 1.98), (2.6, -2.46), (2.93, -0.3),
    (-1.19, -3.28), (3.77, 2.1), (2.24, 3.39), (4.03, 3.6), (-2.18, 0.26), (0.05, 0.18),
    (0.72, -2.52), (1.19, -0.6), (1.86, -4.28), (-1.24, -4.2), (-2.62, -3.29), (-0.46, -2.03),
    (2.24, -1.56), (3.69, -1.32), (0.08, 3.47), (-1.57, 4.53), (-4.08, 2.26), (-3.26, 3.36),
    (1.25, 4.1), (3.25, 4.34), (1.68, 2.18), (-0.6, 2.55), (-0.74, -0.09), (2.06, -3.46),
    (2.94, 0.74), (4.45, 1.39), (4.46, 2.59), (-0.07, 4.63), (-2.61, 4.52), (-4.48, 4.54),
    (-4.25, 3.41), (-4.22, 1.06), (-2.86, -0.56), (-4.18, -1.3), (-3.57, -4.4), (-0.01, -4.35),
    (-4.26, -2.82), (1.33, -1.57), (0.75, -4.6), (3.19, -4.32), (3.14, -3.37), (4.58, -0.53),
    (4.42, -4.44),
];

/// A small repeatable random number in 0..1.
fn roll(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 10_000) as f32 / 10_000.0
}

fn between(seed: &mut u32, lo: f32, hi: f32) -> f32 {
    lo + (hi - lo) * roll(seed)
}

#[allow(clippy::type_complexity)]
fn decorate_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<
        (Entity, &Brick, Has<Stone>),
        Or<(
            Changed<Brick>,
            Added<Stone>,
            Added<Vined>,
            Added<Grassy>,
            Added<Placed>,
        )>,
    >,
    skins: Query<&TowerSkin>,
    since: Query<&HalloweenSince>,
    time: Res<Time>,
    ramp: Option<Res<HoloRamp>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut galaxy_bricks: Option<ResMut<Assets<GalaxyBrick>>>,
    mut diamond_bricks: Option<ResMut<Assets<DiamondBrick>>>,
    mut images: Option<ResMut<Assets<Image>>>,
) {
    for (entity, brick, stone) in &bricks {
        let Ok(skin) = skins.get(brick.tower) else {
            continue;
        };
        // Only the seven normal shapes, in their own look.
        if brick.piece >= config.pieces.len() || stone || brick.ice {
            continue;
        }
        let c = config.cell_size * brick.scale;
        let cells = &brick.cells;
        let mut seed = (entity.to_bits() as u32).wrapping_mul(2_654_435_761) | 1;
        let mut brick_entity = commands.entity(entity);
        match skin.pack {
            BrickPack::Christmas => {
                // The lights: the original's shader draws them from a two-frame picture (one
                // frame above the other), showing one frame at a time over the brick's box,
                // switching every 2 s from a random 0 to 3 s offset (ChristmasBrickModifier).
                let shape = &config.pieces[brick.piece].name;
                if let Some(lights) = art.image(&format!("brick_christmas_lights_{shape}")) {
                    let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                    let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                    let size = (max - min + Vec2::ONE) * c;
                    let center = (min + max) * 0.5 * c;
                    brick_entity.with_child((
                        Lights {
                            offset: roll(&mut seed) * 3.0,
                        },
                        Sprite {
                            image: lights,
                            custom_size: Some(size),
                            ..default()
                        },
                        Transform::from_translation(center.extend(0.05)),
                        // Hidden until `switch_lights` picks a frame.
                        Visibility::Hidden,
                    ));
                }
                // Snow on top edges: 25% each.
                let snows: Vec<Handle<Image>> = (1..=3)
                    .filter_map(|i| art.image(&format!("brick_christmas_snow_{i}")))
                    .collect();
                for &cell in cells {
                    let covered = cells.iter().any(|&o| (o - cell - Vec2::Y).length() < 0.1);
                    if covered || snows.is_empty() || roll(&mut seed) >= 0.25 {
                        continue;
                    }
                    let pick = snows[(roll(&mut seed) * snows.len() as f32) as usize % snows.len()]
                        .clone();
                    let full = Vec3::ONE;
                    brick_entity.with_child((
                        Snow { time: 0.0, full },
                        Sprite {
                            image: pick,
                            custom_size: Some(Vec2::new(c, c * 0.45)),
                            ..default()
                        },
                        Transform::from_translation((cell * c + Vec2::Y * c * 0.5).extend(0.06))
                            .with_scale(Vec3::ZERO),
                    ));
                }
            }
            BrickPack::Halloween => {
                // The original's shader fades the pattern and its light colours to the Halloween
                // ones as `_BlackWhiteEffect` goes 0 to 1; the staged Halloween bricks carry
                // those colours, so their lit picture fades in over the base one.
                let shape = &config.pieces[brick.piece].name;
                let from = since.get(entity).map_or(time.elapsed_secs(), |s| s.0);
                brick_entity.insert(HalloweenSince(from));
                if let Some(look) = art.image(&format!("brick_halloween_{shape}")) {
                    let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                    let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                    let size = (max - min + Vec2::ONE) * c;
                    let center = (min + max) * 0.5 * c;
                    brick_entity.with_child((
                        HalloweenFade(from),
                        Sprite {
                            image: look,
                            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                            custom_size: Some(size),
                            ..default()
                        },
                        Transform::from_translation(center.extend(0.015)),
                    ));
                }
                // Up to 2 front decorations, 40% chance each.
                let kinds = ["bat", "spider", "pumpkin", "spiderweb"];
                for _ in 0..2 {
                    if roll(&mut seed) >= 0.4 {
                        continue;
                    }
                    let kind = kinds[(roll(&mut seed) * 4.0) as usize % 4];
                    let Some(image) = art.image(&format!("brick_halloween_{kind}")) else {
                        continue;
                    };
                    let cell = cells[(roll(&mut seed) * cells.len() as f32) as usize % cells.len()];
                    brick_entity.with_child((
                        Pulse {
                            wait: between(&mut seed, 5.0, 15.0),
                            time: -1.0,
                            seed,
                            kind: 0,
                            side: 1.0,
                        },
                        Sprite {
                            image,
                            custom_size: Some(Vec2::splat(c * 0.8)),
                            ..default()
                        },
                        Transform::from_translation((cell * c).extend(0.07)),
                    ));
                }
            }
            BrickPack::Hologram => {
                brick_entity.insert(Glitch {
                    wait: 0.0,
                    left: 0.0,
                    span: 0.0,
                    seed,
                });
                if let Some(ramp) = ramp.as_ref() {
                    for &cell in cells {
                        brick_entity.with_child((
                            HoloScan,
                            Sprite {
                                image: ramp.0.clone(),
                                color: Color::srgba(0.8, 1.0, 1.0, 0.0),
                                custom_size: Some(Vec2::splat(c)),
                                rect: Some(Rect::new(0.0, 0.0, 1.0, HOLO_RAMP as f32)),
                                ..default()
                            },
                            Transform::from_translation((cell * c).extend(0.03)),
                        ));
                    }
                }
            }
            BrickPack::Diamond => {
                // One quad over the piece's box, drawn by the Diamond material: the pattern with
                // its moving gloss bands, the sweep and the border (docs/research/diamond.md).
                let shape = &config.pieces[brick.piece].name;
                let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                let size = (max - min + Vec2::ONE) * c;
                let center = (min + max) * 0.5 * c;
                let key = format!("brick_diamond_{shape}");
                let line = art.entries.get(&key).and_then(|e| e.line);
                let bevel = art.image(&format!("{key}_bevel"));
                let pattern = art.image(&key);
                let gloss = DIAMOND_GLOSS.iter().find(|(s, ..)| s == shape);
                if let (Some(meshes), Some(diamond_bricks), Some((lr, lg, lb)), Some(bevel), Some(pattern), Some((_, g1, g2))) =
                    (meshes.as_mut(), diamond_bricks.as_mut(), line, bevel, pattern, gloss)
                {
                    // The stripes run in units of the bevel map's pixels at 65 to the unit.
                    let px = images
                        .as_ref()
                        .and_then(|images| images.get(&bevel))
                        .map_or(size / c * 65.0, |image| image.size_f32());
                    let material = diamond_bricks.add(DiamondBrick {
                        look: DiamondLook {
                            gloss1: Vec4::from_array(*g1),
                            gloss2: Vec4::from_array(*g2),
                            border: Vec4::new(lr, lg, lb, 1.0),
                            params: Vec4::ZERO,
                            size: (px / 65.0).extend(0.0).extend(0.0),
                        },
                        bevel,
                        pattern,
                    });
                    brick_entity.with_child((
                        Gloss {
                            wait: 0.0,
                            time: -1.0,
                            seed,
                            turn: 1.0,
                        },
                        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
                        MeshMaterial2d(material),
                        Transform::from_translation(center.extend(0.02)),
                    ));
                }
            }
            BrickPack::Galaxy => {
                // One quad over the piece's box, drawn by the Galaxy material: the sky turned by
                // the shape's hue, lit from the bevel map in world space, outlined in the
                // border colour (docs/research/galaxy.md). The stars go on top.
                let shape = &config.pieces[brick.piece].name;
                let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                let size = (max - min + Vec2::ONE) * c;
                let center = (min + max) * 0.5 * c;
                let key = format!("brick_galaxy_{shape}");
                let entry = art.entries.get(&key);
                let light = entry.and_then(|e| e.light.as_ref()?.get());
                let line = entry.and_then(|e| e.line);
                let bevel = art.image(&format!("{key}_bevel"));
                let sky = art.image("brick_galaxy_background");
                if let (Some(meshes), Some(galaxy_bricks), Some(((hr, hg, hb, ha), (sr, sg, sb, sa))), Some((lr, lg, lb)), Some(bevel), Some(sky)) =
                    (meshes.as_mut(), galaxy_bricks.as_mut(), light, line, bevel, sky)
                {
                    if let Some(images) = images.as_mut() {
                        repeat_starfield(images, &sky);
                    }
                    let red = GALAXY_HUES.iter().find(|(s, _)| s == shape).map_or(0.0, |h| h.1);
                    let hue = std::f32::consts::TAU * (red + 0.5).fract() - std::f32::consts::PI;
                    let material = galaxy_bricks.add(GalaxyBrick {
                        look: GalaxyLook {
                            highlight: Vec4::new(hr, hg, hb, ha),
                            shadow: Vec4::new(sr, sg, sb, sa),
                            border: Vec4::new(lr, lg, lb, 1.0),
                            params: Vec4::new(hue, 0.0, config.cell_size, 0.0),
                        },
                        bevel,
                        sky,
                    });
                    brick_entity.with_child((
                        GalaxyQuad,
                        Mesh2d(meshes.add(Rectangle::new(size.x, size.y))),
                        MeshMaterial2d(material),
                        Transform::from_translation(center.extend(0.02)),
                    ));
                }
                if let Some(star) = art.image("brick_galaxy_star") {
                    for &cell in cells {
                        for index in 0..GALAXY_STARS.len() {
                            brick_entity.with_child((
                                GalaxyStar {
                                    index,
                                    at: cell,
                                    px: c,
                                },
                                Star {
                                    twirl: None,
                                    spin: 0.0,
                                },
                                Sprite {
                                    image: star.clone(),
                                    custom_size: Some(Vec2::splat(c * GALAXY_STAR_SIZE)),
                                    ..default()
                                },
                                Transform::default(),
                                Visibility::Hidden,
                            ));
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// Shows one frame of each Christmas brick's lights: the bottom half of the picture, then the
/// top half, two seconds each (the shader's frame is floor((offset + t) / 2) mod 2, in Unity's
/// up-is-positive UVs, where frame 0 is the lower half).
fn switch_lights(
    time: Res<Time>,
    images: Res<Assets<Image>>,
    mut lights: Query<(&Lights, &mut Sprite, &mut Visibility)>,
) {
    let t = time.elapsed_secs();
    for (light, mut sprite, mut visibility) in &mut lights {
        let Some(size) = images.get(&sprite.image).map(|i| i.size_f32()) else {
            continue;
        };
        let frame = ((light.offset + t) * 0.5).floor().rem_euclid(2.0);
        let half = size.y * 0.5;
        // Frame 0 is the picture's lower half (rows from `half` down), frame 1 the upper.
        let top = if frame < 1.0 { half } else { 0.0 };
        sprite.rect = Some(Rect::new(0.0, top, size.x, top + half));
        if *visibility == Visibility::Hidden {
            *visibility = Visibility::Inherited;
        }
    }
}

/// Fades each Halloween brick's own look in, 1 s after it came, over 1 s (InOutQuad).
fn fade_halloween(time: Res<Time>, mut fades: Query<(&HalloweenFade, &mut Sprite)>) {
    let now = time.elapsed_secs();
    for (fade, mut sprite) in &mut fades {
        let x = (now - fade.0 - 1.0).clamp(0.0, 1.0);
        let eased = if x < 0.5 { 2.0 * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(2) / 2.0 };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, eased);
    }
}

fn grow_snow(time: Res<Time>, mut snow: Query<(&mut Snow, &mut Transform)>) {
    for (mut cap, mut transform) in &mut snow {
        if cap.time >= 0.3 {
            continue;
        }
        cap.time += time.delta_secs();
        transform.scale = cap.full * (cap.time / 0.3).min(1.0);
    }
}

/// DOTween's OutElastic.
fn out_elastic(x: f32) -> f32 {
    if x <= 0.0 || x >= 1.0 {
        return x.clamp(0.0, 1.0);
    }
    2f32.powf(-10.0 * x) * ((x * 10.0 - 0.75) * std::f32::consts::TAU / 3.0).sin() + 1.0
}

/// DOTween's InBack.
fn in_back(x: f32) -> f32 {
    let c = 1.70158;
    (c + 1.0) * x * x * x - c * x * x
}

/// One axis squashing to 0.8× in 0.1 s and springing back over 1 s.
fn squash(t: f32) -> f32 {
    if t < 0.1 {
        1.0 - 2.0 * t.max(0.0)
    } else {
        0.8 + 0.2 * out_elastic(t - 0.1)
    }
}

fn pulse_decorations(time: Res<Time>, mut items: Query<(&mut Pulse, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut pulse, mut transform) in &mut items {
        if pulse.time < 0.0 {
            pulse.wait -= dt;
            if pulse.wait > 0.0 {
                continue;
            }
            let mut seed = pulse.seed;
            pulse.kind = if roll(&mut seed) < 0.5 {
                0
            } else if roll(&mut seed) < 0.5 {
                1
            } else {
                2
            };
            pulse.side = if roll(&mut seed) < 0.5 { 1.0 } else { -1.0 };
            pulse.wait = between(&mut seed, 5.0, 15.0);
            pulse.seed = seed;
            pulse.time = 0.0;
        }
        pulse.time += dt;
        let t = pulse.time;
        let done = match pulse.kind {
            0 => {
                // 25° over 0.926 s (in-back), then back over 0.45 s (out-elastic).
                let angle = if t < 0.926 {
                    25.0 * in_back(t / 0.926)
                } else {
                    25.0 * (1.0 - out_elastic((t - 0.926) / 0.45))
                };
                transform.rotation = Quat::from_rotation_z((angle * pulse.side).to_radians());
                t >= 0.926 + 0.45
            }
            kind => {
                let (first, second) = (squash(t), squash(t - 0.1));
                transform.scale = if kind == 1 {
                    Vec3::new(first, second, 1.0)
                } else {
                    Vec3::new(second, first, 1.0)
                };
                t >= 1.2
            }
        };
        if done {
            transform.rotation = Quat::IDENTITY;
            transform.scale = Vec3::ONE;
            pulse.time = -1.0;
        }
    }
}

/// The glitch, as the original's shader does it: while the distortion D is up, the brick
/// flickers see-through, alpha 1 - D x h with h a fresh random every tenth of a second, and its
/// rows tear sideways (each cell takes its own share here). At rest it's the base skin, solid.
fn glitch_holograms(
    time: Res<Time>,
    mut bricks: Query<(&mut Glitch, &Children)>,
    mut sprites: Query<(&mut Sprite, &mut Transform), With<CellSprite>>,
) {
    let dt = time.delta_secs();
    // The shader's hash: frac(sin(100 n) x 18336.82) for n = floor(10 t).
    let step = (time.elapsed_secs() * 10.0).floor();
    let flicker = ((step * 100.0).sin() * 18_336.822).rem_euclid(1.0);
    for (mut glitch, children) in &mut bricks {
        let mut seed = glitch.seed;
        if glitch.left > 0.0 {
            glitch.left -= dt;
        } else {
            glitch.wait -= dt;
            if glitch.wait <= 0.0 {
                glitch.span = 2.0 * between(&mut seed, 0.1, 0.5);
                glitch.left = glitch.span;
                glitch.wait = between(&mut seed, 6.0, 15.0);
            }
        }
        let on = glitch.left > 0.0;
        let amount = if on { glitch_amount(1.0 - glitch.left / glitch.span) } else { 0.0 };
        let width = amount * 6.0;
        for child in children.iter() {
            if let Ok((mut sprite, mut transform)) = sprites.get_mut(child) {
                sprite.color = Color::srgba(1.0, 1.0, 1.0, 1.0 - amount * flicker);
                transform.translation.x = glitch_slice(width, roll(&mut seed));
            }
        }
        glitch.seed = seed;
    }
}

/// The distortion `progress` (0 to 1) through a glitch: up to 1 in the first half and back
/// down in the second, both OutQuad (DOTween's default ease).
pub(crate) fn glitch_amount(progress: f32) -> f32 {
    let out_quad = |x: f32| 1.0 - (1.0 - x) * (1.0 - x);
    let p = progress.clamp(0.0, 1.0);
    if p < 0.5 {
        out_quad(p * 2.0)
    } else {
        1.0 - out_quad(p * 2.0 - 1.0)
    }
}

/// Moves each Hologram cell's scan lines with its world height and time, and pulses them.
fn scan_holograms(time: Res<Time>, mut scans: Query<(&mut Sprite, &GlobalTransform), With<HoloScan>>, config: Res<GameConfig>) {
    let t = time.elapsed_secs();
    let k = 0.2 + 0.025 * ((50.0 * t).sin() + 1.0);
    let period = HOLO_RAMP as f32;
    for (mut sprite, at) in &mut scans {
        // frac(3 y + t) up the cell: three periods a cell, the picture's rows run down.
        let y = at.translation().y / config.cell_size;
        let top = -(3.0 * (y + 0.5) + t) * period;
        sprite.rect = Some(Rect::new(0.0, top, 1.0, top + 3.0 * period));
        sprite.color = Color::srgba(0.8, 1.0, 1.0, k * HOLO_SCAN_LINEAR);
    }
}

/// Makes the scan-line ramp once: a 1 x 32 picture, white, alpha rising up it, repeating.
fn make_holo_ramp(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    let mut data = Vec::with_capacity(HOLO_RAMP as usize * 4);
    for row in 0..HOLO_RAMP {
        // Row 0 is the top: alpha 1 there, falling to 0 at the bottom (rising upwards).
        let a = 1.0 - (row as f32 + 0.5) / HOLO_RAMP as f32;
        data.extend_from_slice(&[255, 255, 255, (a * 255.0).round() as u8]);
    }
    let mut image = Image::new(
        Extent3d { width: 1, height: HOLO_RAMP, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: ImageAddressMode::Repeat,
        address_mode_v: ImageAddressMode::Repeat,
        ..ImageSamplerDescriptor::nearest()
    });
    commands.insert_resource(HoloRamp(images.add(image)));
}

/// One cell's sideways tear during a glitch: `roll` (0 to 1) picks how far, within
/// `width` either way.
pub(crate) fn glitch_slice(width: f32, roll: f32) -> f32 {
    (roll - 0.5) * 2.0 * width
}

/// Runs each Diamond brick's sweep and keeps its light fixed in the world (the material turns
/// the bevel's slope by the brick's rotation, as the original's object-to-world matrix does).
fn gloss_diamonds(
    time: Res<Time>,
    mut quads: Query<(&mut Gloss, &MeshMaterial2d<DiamondBrick>, &GlobalTransform)>,
    mut materials: Option<ResMut<Assets<DiamondBrick>>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    let dt = time.delta_secs();
    for (mut gloss, material, at) in &mut quads {
        let mut seed = gloss.seed;
        if gloss.time >= 0.0 {
            gloss.time += dt;
            if gloss.time >= 0.6 {
                gloss.time = -1.0;
                gloss.wait = between(&mut seed, 6.0, 15.0);
            }
        } else {
            gloss.wait -= dt;
            if gloss.wait <= 0.0 {
                gloss.time = 0.0;
                gloss.turn = if roll(&mut seed) > 0.5 { -1.0 } else { 1.0 };
            }
        }
        gloss.seed = seed;
        let (angle, strength) = if gloss.time >= 0.0 {
            diamond_sweep(gloss.time, gloss.turn)
        } else {
            (0.0, 0.0)
        };
        let rotation = at.rotation().to_euler(EulerRot::XYZ).2;
        if let Some(mut brick) = materials.get_mut(&material.0) {
            brick.look.params = Vec4::new(angle, strength, rotation, 0.0);
        }
    }
}

/// The sweep's angle and strength `t` seconds into it, turning one way (`turn` is 1 or -1).
pub(crate) fn diamond_sweep(t: f32, turn: f32) -> (f32, f32) {
    let out_quad = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        1.0 - (1.0 - x) * (1.0 - x)
    };
    let in_out_quad = |x: f32| {
        let x = x.clamp(0.0, 1.0);
        if x < 0.5 { 2.0 * x * x } else { 1.0 - (-2.0 * x + 2.0).powi(2) / 2.0 }
    };
    let strength = if t < 0.15 {
        0.7 * out_quad(t / 0.15)
    } else if t < 0.45 {
        0.7
    } else {
        0.7 * (1.0 - out_quad((t - 0.45) / 0.15))
    };
    (turn * std::f32::consts::TAU * in_out_quad(t / 0.6), strength)
}

/// Keeps each Galaxy brick's light fixed in the world: the material turns the bevel's slope by
/// the brick's rotation, as the original's shader does with its object-to-world matrix.
fn turn_galaxy_bricks(
    quads: Query<(&MeshMaterial2d<GalaxyBrick>, &GlobalTransform), With<GalaxyQuad>>,
    mut materials: Option<ResMut<Assets<GalaxyBrick>>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    for (material, at) in &quads {
        let angle = at.rotation().to_euler(EulerRot::XYZ).2;
        if let Some(mut brick) = materials.get_mut(&material.0)
            && (brick.look.params.y - angle).abs() > 1e-3
        {
            brick.look.params.y = angle;
        }
    }
}

/// How far the original's sky has moved at time `t`, in texture widths, x right and y down: the
/// scene's scroll (0.05, -0.07 a second in Unity's up-is-positive UVs) plus the brick shader's
/// sway, 0.1 cos(0.3 t) across and 0.2 sin(0.1 t) up and down (periods about 21 s and 63 s).
pub(crate) fn galaxy_offset(t: f32) -> Vec2 {
    Vec2::new(0.05 * t + 0.1 * (0.3 * t).cos(), 0.07 * t - 0.2 * (0.1 * t).sin())
}

/// Makes the starfield repeat at its edges (once; the sampler is left set after that).
fn repeat_starfield(images: &mut Assets<Image>, handle: &Handle<Image>) {
    let repeats = images.get(handle).is_some_and(|i| {
        matches!(&i.sampler, ImageSampler::Descriptor(d)
            if d.address_mode_u == ImageAddressMode::Repeat && d.address_mode_v == ImageAddressMode::Repeat)
    });
    if !repeats && let Some(mut image) = images.get_mut(handle) {
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::Repeat,
            address_mode_v: ImageAddressMode::Repeat,
            ..ImageSamplerDescriptor::linear()
        });
    }
}

/// Puts each of the original's stars where the shared field has it inside its cell. The
/// field drifts with the sky (the scene's offset), wraps every 10 cells, and a star shows
/// only inside its own cell.
fn place_galaxy_stars(
    time: Res<Time>,
    galaxy: Res<GalaxyDebug>,
    mut stars: Query<(&GalaxyStar, &ChildOf, &mut Transform, &mut Visibility)>,
    bricks: Query<&GlobalTransform>,
) {
    let t = if galaxy.scroll { time.elapsed_secs() } else { 0.0 };
    let half = GALAXY_FIELD_CELLS * 0.5;
    // The stars are in the sky, so they move with it: the offset turned into cells, the
    // picture going left as the offset grows and up as its row offset grows.
    let offset = galaxy_offset(t);
    let drift = Vec2::new(-offset.x, offset.y) * GALAXY_FIELD_CELLS;
    let wrap = |v: Vec2| (v + Vec2::splat(half)).rem_euclid(Vec2::splat(GALAXY_FIELD_CELLS)) - Vec2::splat(half);
    for (star, parent, mut transform, mut visibility) in &mut stars {
        let Ok(brick) = bricks.get(parent.parent()) else {
            continue;
        };
        let rotation = brick.rotation();
        let centre = brick.translation().truncate() + (rotation * (star.at * star.px).extend(0.0)).truncate();
        // Where this star is in the field, in cells: the half-cell is the window's centre.
        let (x, y) = GALAXY_STARS[star.index];
        let field = Vec2::new(x + half - 0.5, y - half + 0.5) + drift;
        let offset = wrap(field - centre / star.px);
        // Back into the brick's own cell frame, in cells.
        let local = rotation.inverse() * (offset * star.px).extend(0.0);
        let cell = local.truncate() / star.px;
        let inside = cell.x.abs() <= 0.5 && cell.y.abs() <= 0.5;
        *visibility = if inside { Visibility::Inherited } else { Visibility::Hidden };
        transform.translation = ((star.at + cell) * star.px).extend(0.03);
    }
}

fn twirl_stars(
    time: Res<Time>,
    mut next: Local<(f32, u32)>,
    mut stars: Query<(&mut Star, &mut Transform, &GlobalTransform)>,
) {
    let dt = time.delta_secs();
    let now = time.elapsed_secs();
    // One shared clock picks a random star every 2 to 6 s.
    let (wait, seed) = &mut *next;
    if *seed == 0 {
        *seed = 0x9e37_79b9;
    }
    *wait -= dt;
    if *wait <= 0.0 {
        *wait = between(seed, 2.0, 6.0);
        let count = stars.iter().len();
        if count > 0 {
            let pick = (roll(seed) * count as f32) as usize % count;
            if let Some((mut star, ..)) = stars.iter_mut().nth(pick)
                && star.twirl.is_none()
            {
                star.twirl = Some(0.0);
            }
        }
    }
    for (mut star, mut transform, at) in &mut stars {
        transform.rotate_z((star.spin * dt).to_radians());
        star.spin = (star.spin - 200.0 * dt).max(0.0);
        let Some(t) = star.twirl else {
            let p = at.translation();
            transform.scale = Vec3::splat(1.0 + 0.3 * (now * 2.0 + p.x * p.y).sin());
            continue;
        };
        let t = t + dt;
        star.twirl = Some(t);
        let scale = if t < 0.3 {
            1.0 + (t / 0.3)
        } else if t < 3.3 {
            2.0
        } else {
            2.0 - ((t - 3.3) / 0.3).min(1.0)
        };
        transform.scale = Vec3::splat(scale);
        let turned = |x: f32| 180.0 * in_back(x.clamp(0.0, 1.0));
        if t > 0.3 && t - dt < 1.3 {
            let step = turned(t - 0.3) - turned(t - dt - 0.3);
            transform.rotate_z(step.to_radians());
            if t >= 1.3 {
                star.spin = 600.0;
            }
        }
        if t >= 3.6 {
            star.twirl = None;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_glitch_tears_each_cell_within_its_width() {
        assert_eq!(glitch_slice(0.0, 0.9), 0.0, "no glitch, no tear");
        for r in [0.0, 0.25, 0.5, 0.75, 1.0] {
            assert!(glitch_slice(3.0, r).abs() <= 3.0 + 1e-6);
        }
        assert!(glitch_slice(3.0, 0.0) < 0.0 && glitch_slice(3.0, 1.0) > 0.0);
    }

    #[test]
    fn the_diamond_sweep_rises_holds_and_turns_a_full_circle() {
        assert_eq!(diamond_sweep(0.0, 1.0), (0.0, 0.0));
        assert!((diamond_sweep(0.3, 1.0).1 - 0.7).abs() < 1e-5, "held at 0.7 in the middle");
        let (angle, strength) = diamond_sweep(0.6, -1.0);
        assert!((angle + std::f32::consts::TAU).abs() < 1e-4, "a full turn the other way");
        assert!(strength.abs() < 1e-5, "faded out at the end");
    }

    #[test]
    fn pack_eases_start_and_end_where_the_original_does() {
        assert_eq!(out_elastic(0.0), 0.0);
        assert_eq!(out_elastic(1.0), 1.0);
        assert!(in_back(0.0).abs() < 1e-6 && (in_back(1.0) - 1.0).abs() < 1e-5);
        assert!(in_back(0.2) < 0.0, "in-back dips first");
        assert!((squash(0.1) - 0.8).abs() < 1e-5);
        assert!((squash(1.1) - 1.0).abs() < 1e-5);
    }

    #[test]
    fn hologram_glitch_rises_to_one_and_falls_back() {
        assert_eq!(glitch_amount(0.0), 0.0);
        assert!((glitch_amount(0.5) - 1.0).abs() < 1e-5, "full at the halfway point");
        assert!(glitch_amount(1.0).abs() < 1e-5, "back to rest at the end");
        assert!(glitch_amount(0.25) > 0.5, "OutQuad: past half-way at a quarter");
    }
}
