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
        Self { scroll: true }
    }
}

impl Plugin for PacksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GalaxyDebug>()
            .init_resource::<GalaxySky>();
        app.add_systems(
            Update,
            (
                decorate_bricks.after(dress_bricks),
                grow_snow,
                switch_lights,
                fade_halloween,
                pulse_decorations,
                glitch_holograms,
                gloss_diamonds,
                turn_galaxy_bricks,
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

/// A snow cap growing in (`edge_growth`) after its own 0 to 0.3 s delay: `time` starts at
/// minus the delay. `full` is its finished scale, flipped sideways or not at random, as the
/// original's EdgeDecorationModifier does.
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
    /// Its resting turn, in degrees (up to 15 either way).
    tilt: f32,
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
    (
        "J",
        [0.419, 0.857, 0.919, 0.522],
        [0.502, 0.690, 0.949, 0.522],
    ),
    (
        "L",
        [0.882, 0.365, 0.200, 0.616],
        [1.0, 0.923, 0.375, 0.428],
    ),
    (
        "O",
        [0.878, 0.663, 1.0, 0.741],
        [0.792, 0.318, 0.867, 0.522],
    ),
    (
        "S",
        [0.431, 0.925, 0.655, 0.465],
        [0.806, 0.971, 0.881, 0.716],
    ),
    (
        "T",
        [0.956, 0.893, 0.759, 0.766],
        [0.585, 1.0, 0.522, 0.603],
    ),
    ("Z", [1.0, 0.294, 0.294, 0.341], [1.0, 1.0, 1.0, 0.559]),
];

/// A Galaxy brick's look, drawn by its own material (`brick_material::GalaxyBrick`).
#[derive(Component)]
struct GalaxyQuad;

/// The original's one Galaxy sky, shared by every Galaxy brick: its 49 stars' scale and turn.
/// Each star starts at a random turn and breathes 1 +/- 0.3 (sin(2 t + x y), with its position
/// in the scene, which sits 1000 units to the left); every 2 to 6 s one random star twirls:
/// grows to 2 in 0.3 s, turns 180 degrees over 1 s (InBack), spins on from 600 degrees a second
/// slowing by 200 a second squared, and shrinks back after 2 s (Star, GalaxyBrickPackBehaviour).
#[derive(Resource)]
struct GalaxySky {
    turn: [f32; 49],
    spin: [f32; 49],
    twirl: [Option<f32>; 49],
    wait: f32,
    seed: u32,
}

impl Default for GalaxySky {
    fn default() -> Self {
        let mut seed = 0x9e37_79b9;
        let mut turn = [0.0; 49];
        for t in &mut turn {
            *t = roll(&mut seed) * std::f32::consts::TAU;
        }
        Self {
            turn,
            spin: [0.0; 49],
            twirl: [None; 49],
            wait: 0.0,
            seed,
        }
    }
}

/// When a Halloween brick came: its look fades from the base skin to the Halloween one from
/// then (HalloweenBrickModifier: 1 s after it spawns, over 1 s, InOutQuad).
#[derive(Component)]
struct HalloweenSince(f32);

/// The Halloween look over a brick's base skin, fading in (`fade_halloween`).
#[derive(Component)]
struct HalloweenFade(f32);

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
    (-1.12, -1.19),
    (-1.6, 2.78),
    (-3.89, -0.14),
    (0.42, 1.98),
    (2.6, -2.46),
    (2.93, -0.3),
    (-1.19, -3.28),
    (3.77, 2.1),
    (2.24, 3.39),
    (4.03, 3.6),
    (-2.18, 0.26),
    (0.05, 0.18),
    (0.72, -2.52),
    (1.19, -0.6),
    (1.86, -4.28),
    (-1.24, -4.2),
    (-2.62, -3.29),
    (-0.46, -2.03),
    (2.24, -1.56),
    (3.69, -1.32),
    (0.08, 3.47),
    (-1.57, 4.53),
    (-4.08, 2.26),
    (-3.26, 3.36),
    (1.25, 4.1),
    (3.25, 4.34),
    (1.68, 2.18),
    (-0.6, 2.55),
    (-0.74, -0.09),
    (2.06, -3.46),
    (2.94, 0.74),
    (4.45, 1.39),
    (4.46, 2.59),
    (-0.07, 4.63),
    (-2.61, 4.52),
    (-4.48, 4.54),
    (-4.25, 3.41),
    (-4.22, 1.06),
    (-2.86, -0.56),
    (-4.18, -1.3),
    (-3.57, -4.4),
    (-0.01, -4.35),
    (-4.26, -2.82),
    (1.33, -1.57),
    (0.75, -4.6),
    (3.19, -4.32),
    (3.14, -3.37),
    (4.58, -0.53),
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
        (Entity, &Brick, Has<Stone>, Has<Placed>),
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
    mut holo_bricks: Option<ResMut<Assets<crate::brick_material::HologramBrick>>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut galaxy_bricks: Option<ResMut<Assets<GalaxyBrick>>>,
    mut diamond_bricks: Option<ResMut<Assets<DiamondBrick>>>,
    mut lit_bricks: Option<ResMut<Assets<crate::brick_material::LitBrick>>>,
    mut images: Option<ResMut<Assets<Image>>>,
) {
    for (entity, brick, stone, placed) in &bricks {
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
                // Snow on top edges: 25% each, once the brick is placed (the original applies
                // ChristmasBrickModifier then).
                let snows: Vec<Handle<Image>> = (1..=3)
                    .filter_map(|i| art.image(&format!("brick_christmas_snow_{i}")))
                    .filter(|_| placed)
                    .collect();
                for &cell in cells {
                    let covered = cells.iter().any(|&o| (o - cell - Vec2::Y).length() < 0.1);
                    if covered || snows.is_empty() || roll(&mut seed) >= 0.25 {
                        continue;
                    }
                    let pick = snows[(roll(&mut seed) * snows.len() as f32) as usize % snows.len()]
                        .clone();
                    let flip = if roll(&mut seed) < 0.5 { 1.0 } else { -1.0 };
                    let full = Vec3::new(flip, 1.0, 1.0);
                    let delay = roll(&mut seed) * 0.3;
                    brick_entity.with_child((
                        Snow { time: -delay, full },
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
                let key = format!("brick_halloween_{shape}");
                if let Some(look) = art.image(&key) {
                    let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                    let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                    let size = (max - min + Vec2::ONE) * c;
                    let center = (min + max) * 0.5 * c;
                    let clear = Color::srgba(1.0, 1.0, 1.0, 0.0);
                    let at = Transform::from_translation(center.extend(0.015));
                    let lit = meshes.as_mut().zip(lit_bricks.as_mut()).and_then(
                        |(meshes, lit_bricks)| {
                            crate::brick_material::lit_brick(
                                &art, &key, size, clear, meshes, lit_bricks,
                            )
                        },
                    );
                    match lit {
                        Some((mesh, material)) => {
                            crate::brick_material::spawn_quad(
                                &mut brick_entity,
                                at,
                                clear,
                                HalloweenFade(from),
                                (),
                                mesh,
                                material,
                            );
                        }
                        None => {
                            brick_entity.with_child((
                                HalloweenFade(from),
                                Sprite {
                                    image: look,
                                    color: clear,
                                    custom_size: Some(size),
                                    ..default()
                                },
                                at,
                            ));
                        }
                    }
                }
                // Up to 2 front decorations, 40% chance each, added once the brick is placed
                // (HalloweenBrickModifier's placed handler), each after its own 0 to 0.3 s and
                // grown in like the other edge decorations.
                let kinds = ["bat01", "spider01", "pumpkin01", "spiderweb"];
                for _ in 0..2 {
                    if !placed || roll(&mut seed) >= 0.4 {
                        continue;
                    }
                    let kind = kinds[(roll(&mut seed) * 4.0) as usize % 4];
                    let name = format!("brick_halloween_{kind}");
                    let Some(image) = art.image(&name) else {
                        continue;
                    };
                    // The picture's own size, at its pixels per unit (a cell is one unit).
                    let ppu = art
                        .entries
                        .get(&name)
                        .and_then(|e| e.pixels_per_unit)
                        .unwrap_or(65.0);
                    let size = images
                        .as_ref()
                        .and_then(|images| images.get(&image))
                        .map_or(Vec2::splat(c * 0.8), |i| i.size_f32() / ppu * c);
                    let flip = if roll(&mut seed) < 0.5 { 1.0 } else { -1.0 };
                    let delay = roll(&mut seed) * 0.3;
                    let cell = cells[(roll(&mut seed) * cells.len() as f32) as usize % cells.len()];
                    let tilt = between(&mut seed, -15.0, 15.0);
                    brick_entity.with_child((
                        Pulse {
                            wait: between(&mut seed, 5.0, 15.0),
                            time: -1.0,
                            seed,
                            kind: 0,
                            side: 1.0,
                            tilt,
                        },
                        Snow {
                            time: -delay,
                            full: Vec3::new(flip, 1.0, 1.0),
                        },
                        Sprite {
                            image,
                            custom_size: Some(size),
                            ..default()
                        },
                        // Turned up to 15 degrees either way (the original's angle variation).
                        Transform::from_translation((cell * c).extend(0.07))
                            .with_rotation(Quat::from_rotation_z(tilt.to_radians()))
                            .with_scale(Vec3::ZERO),
                    ));
                }
            }
            BrickPack::Hologram => {
                // One quad over the piece's box drawn by the Hologram material: the base skin
                // with the pack's scan lines and glitch (docs/research/hologram.md). The glitch
                // starts once the brick is placed (HologramBrickModifier is applied then).
                let key = format!("brick_{}", config.pieces[brick.piece].name);
                let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                let size = (max - min + Vec2::ONE) * c;
                let center = (min + max) * 0.5 * c;
                if let Some((mesh, material)) = meshes.as_mut().zip(holo_bricks.as_mut()).and_then(
                    |(meshes, holo_bricks)| {
                        crate::brick_material::hologram_brick(&art, &key, size, c, meshes, holo_bricks)
                    },
                ) {
                    // An unplaced brick's glitch never comes.
                    let glitch = Glitch {
                        wait: if placed { 0.0 } else { f32::INFINITY },
                        left: 0.0,
                        span: 0.0,
                        seed,
                    };
                    crate::brick_material::spawn_quad(
                        &mut brick_entity,
                        Transform::from_translation(center.extend(0.0)),
                        Color::WHITE,
                        CellSprite,
                        glitch,
                        mesh,
                        material,
                    );
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
                if let (
                    Some(meshes),
                    Some(diamond_bricks),
                    Some((lr, lg, lb)),
                    Some(bevel),
                    Some(pattern),
                    Some((_, g1, g2)),
                ) = (
                    meshes.as_mut(),
                    diamond_bricks.as_mut(),
                    line,
                    bevel,
                    pattern,
                    gloss,
                ) {
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
                            tint: Vec4::ONE,
                            water: Vec4::ZERO,
                            water_line: Vec4::ZERO,
                            water_colour: Vec4::ZERO,
                        },
                        bevel,
                        pattern,
                    });
                    crate::brick_material::spawn_quad(
                        &mut brick_entity,
                        Transform::from_translation(center.extend(0.02)),
                        Color::WHITE,
                        CellSprite,
                        // The sweep starts once the brick is placed (DiamondBrickModifier
                        // is applied then); the bands are the pack's shader, there from spawn.
                        Gloss {
                            wait: if placed { 0.0 } else { f32::INFINITY },
                            time: -1.0,
                            seed,
                            turn: 1.0,
                        },
                        meshes.add(Rectangle::new(size.x, size.y)),
                        material,
                    );
                }
            }
            BrickPack::Galaxy => {
                // One quad over the piece's box, drawn by the Galaxy material: the sky turned by
                // the shape's hue, lit from the bevel map in world space, outlined in the
                // border colour (docs/research/galaxy.md). The stars are in the sky (`GalaxySky`).
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
                let star = art.image("brick_galaxy_star");
                if let (
                    Some(meshes),
                    Some(galaxy_bricks),
                    Some(((hr, hg, hb, ha), (sr, sg, sb, sa))),
                    Some((lr, lg, lb)),
                    Some(bevel),
                    Some(sky),
                    Some(star),
                ) = (
                    meshes.as_mut(),
                    galaxy_bricks.as_mut(),
                    light,
                    line,
                    bevel,
                    sky,
                    star,
                ) {
                    if let Some(images) = images.as_mut() {
                        repeat_starfield(images, &sky);
                    }
                    let red = GALAXY_HUES
                        .iter()
                        .find(|(s, _)| s == shape)
                        .map_or(0.0, |h| h.1);
                    let hue = std::f32::consts::TAU * (red + 0.5).fract() - std::f32::consts::PI;
                    let material = galaxy_bricks.add(GalaxyBrick {
                        look: GalaxyLook {
                            highlight: Vec4::new(hr, hg, hb, ha),
                            shadow: Vec4::new(sr, sg, sb, sa),
                            border: Vec4::new(lr, lg, lb, 1.0),
                            params: Vec4::new(hue, 0.0, config.cell_size, 0.0),
                            stars: [Vec4::ZERO; 49],
                            tint: Vec4::ONE,
                            water: Vec4::ZERO,
                            water_line: Vec4::ZERO,
                            water_colour: Vec4::ZERO,
                        },
                        bevel,
                        sky,
                        star,
                    });
                    crate::brick_material::spawn_quad(
                        &mut brick_entity,
                        Transform::from_translation(center.extend(0.02)),
                        Color::WHITE,
                        CellSprite,
                        GalaxyQuad,
                        meshes.add(Rectangle::new(size.x, size.y)),
                        material,
                    );
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
        let eased = if x < 0.5 {
            2.0 * x * x
        } else {
            1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
        };
        sprite.color = Color::srgba(1.0, 1.0, 1.0, eased);
    }
}

fn grow_snow(time: Res<Time>, mut snow: Query<(&mut Snow, &mut Transform)>) {
    for (mut cap, mut transform) in &mut snow {
        if cap.time >= 0.5 {
            continue;
        }
        cap.time += time.delta_secs();
        transform.scale = cap.full * edge_growth(cap.time);
    }
}

/// How big an edge decoration is `t` seconds after its delay (EdgeDecorationModifier): to 0.7
/// over 0.1 s (linear), then to full size over 0.4 s (OutElastic). Nothing before.
pub(crate) fn edge_growth(t: f32) -> f32 {
    if t <= 0.0 {
        0.0
    } else if t < 0.1 {
        0.7 * t / 0.1
    } else {
        0.7 + 0.3 * out_elastic(((t - 0.1) / 0.4).min(1.0))
    }
}

/// DOTween's OutElastic.
pub(crate) fn out_elastic(x: f32) -> f32 {
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
                transform.rotation =
                    Quat::from_rotation_z((pulse.tilt + angle * pulse.side).to_radians());
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
            transform.rotation = Quat::from_rotation_z(pulse.tilt.to_radians());
            transform.scale = Vec3::ONE;
            pulse.time = -1.0;
        }
    }
}

/// Runs each placed Hologram brick's glitch and hands its distortion to the material, which
/// draws the flicker, the torn rows, the bright bands and the outline from it.
fn glitch_holograms(
    time: Res<Time>,
    mut quads: Query<(&mut Glitch, &MeshMaterial2d<crate::brick_material::HologramBrick>)>,
    mut materials: Option<ResMut<Assets<crate::brick_material::HologramBrick>>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    let dt = time.delta_secs();
    for (mut glitch, material) in &mut quads {
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
        glitch.seed = seed;
        let amount = if glitch.left > 0.0 {
            glitch_amount(1.0 - glitch.left / glitch.span)
        } else {
            0.0
        };
        if materials.get(&material.0).is_some_and(|m| m.look.params.y != amount)
            && let Some(mut brick) = materials.get_mut(&material.0)
        {
            brick.look.params.y = amount;
        }
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
        if x < 0.5 {
            2.0 * x * x
        } else {
            1.0 - (-2.0 * x + 2.0).powi(2) / 2.0
        }
    };
    let strength = if t < 0.15 {
        0.7 * out_quad(t / 0.15)
    } else if t < 0.45 {
        0.7
    } else {
        0.7 * (1.0 - out_quad((t - 0.45) / 0.15))
    };
    (
        turn * std::f32::consts::TAU * in_out_quad(t / 0.6),
        strength,
    )
}

/// Keeps each Galaxy brick's light fixed in the world: the material turns the bevel's slope by
/// the brick's rotation, as the original's shader does with its object-to-world matrix.
fn turn_galaxy_bricks(
    time: Res<Time>,
    galaxy: Res<GalaxyDebug>,
    mut sky: ResMut<GalaxySky>,
    quads: Query<(&MeshMaterial2d<GalaxyBrick>, &GlobalTransform), With<GalaxyQuad>>,
    mut materials: Option<ResMut<Assets<GalaxyBrick>>>,
) {
    let Some(materials) = materials.as_mut() else {
        return;
    };
    let stars = animate_galaxy_sky(
        &mut sky,
        time.delta_secs(),
        time.elapsed_secs(),
        galaxy.scroll,
    );
    for (material, at) in &quads {
        let angle = at.rotation().to_euler(EulerRot::XYZ).2;
        if let Some(mut brick) = materials.get_mut(&material.0) {
            brick.look.params.y = angle;
            brick.look.stars = stars;
        }
    }
}

/// Moves the shared sky's stars on by `dt` and gives their positions, scales and turns.
fn animate_galaxy_sky(sky: &mut GalaxySky, dt: f32, now: f32, running: bool) -> [Vec4; 49] {
    if running {
        sky.wait -= dt;
        if sky.wait <= 0.0 {
            let mut seed = sky.seed;
            sky.wait = between(&mut seed, 2.0, 6.0);
            let pick = (roll(&mut seed) * 49.0) as usize % 49;
            sky.seed = seed;
            if sky.twirl[pick].is_none() {
                sky.twirl[pick] = Some(0.0);
            }
        }
    }
    let turned = |x: f32| 180.0 * in_back(x.clamp(0.0, 1.0));
    let mut out = [Vec4::ZERO; 49];
    for i in 0..49 {
        let (x, y) = GALAXY_STARS[i];
        sky.turn[i] += sky.spin[i].to_radians() * dt;
        sky.spin[i] = (sky.spin[i] - 200.0 * dt).max(0.0);
        let scale = match sky.twirl[i] {
            None => 1.0 + 0.3 * (now * 2.0 + (x - 1000.0) * y).sin(),
            Some(t) => {
                let t = t + dt;
                if t > 0.3 && t - dt < 1.3 {
                    sky.turn[i] += (turned(t - 0.3) - turned(t - dt - 0.3)).to_radians();
                    if t >= 1.3 {
                        sky.spin[i] = 600.0;
                    }
                }
                sky.twirl[i] = if t >= 3.6 { None } else { Some(t) };
                if t < 0.3 {
                    1.0 + t / 0.3
                } else if t < 3.3 {
                    2.0
                } else {
                    2.0 - ((t - 3.3) / 0.3).min(1.0)
                }
            }
        };
        out[i] = Vec4::new(x, y, scale, sky.turn[i]);
    }
    out
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_diamond_sweep_rises_holds_and_turns_a_full_circle() {
        assert_eq!(diamond_sweep(0.0, 1.0), (0.0, 0.0));
        assert!(
            (diamond_sweep(0.3, 1.0).1 - 0.7).abs() < 1e-5,
            "held at 0.7 in the middle"
        );
        let (angle, strength) = diamond_sweep(0.6, -1.0);
        assert!(
            (angle + std::f32::consts::TAU).abs() < 1e-4,
            "a full turn the other way"
        );
        assert!(strength.abs() < 1e-5, "faded out at the end");
    }

    #[test]
    fn edge_decorations_grow_to_0_7_then_spring_to_full_size() {
        assert_eq!(edge_growth(-0.1), 0.0);
        assert!((edge_growth(0.1) - 0.7).abs() < 1e-5);
        assert!((edge_growth(0.5) - 1.0).abs() < 1e-5);
        assert!(edge_growth(0.2) > 0.7, "the elastic overshoots on the way");
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
        assert!(
            (glitch_amount(0.5) - 1.0).abs() < 1e-5,
            "full at the halfway point"
        );
        assert!(glitch_amount(1.0).abs() < 1e-5, "back to rest at the end");
        assert!(
            glitch_amount(0.25) > 0.5,
            "OutQuad: past half-way at a quarter"
        );
    }
}
