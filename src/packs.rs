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
use crate::config::{BrickPack, GameConfig};
use crate::gameplay::{Brick, Placed, TowerSkin};
use crate::spells::{Grassy, Stone, Vined};
use crate::start::keyed;
use crate::visuals::{CellSprite, dress_bricks};

pub struct PacksPlugin;

impl Plugin for PacksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                decorate_bricks.after(dress_bricks),
                grow_snow,
                pulse_decorations,
                land_holograms.before(glitch_holograms),
                glitch_holograms,
                gloss_diamonds,
                scroll_galaxy,
                twirl_stars,
            ),
        );
    }
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

/// A Hologram brick: glitches every 6 to 15 s, the first right away. A glitch takes
/// 0.1 to 0.5 s to come in and as long to go out (HologramBrickModifier), its
/// distortion rising and falling over that time. A landed brick also goes see-through
/// for a moment (`HologramLook`): `touch` counts seconds since it landed.
#[derive(Component)]
struct Glitch {
    wait: f32,
    left: f32,
    span: f32,
    seed: u32,
    touch: f32,
}

/// A Diamond brick: a 0.6 s gloss every 6 to 15 s, the first right away.
#[derive(Component)]
struct Gloss {
    wait: f32,
    time: f32,
    seed: u32,
}

/// One cell's window onto the shared Galaxy starfield.
#[derive(Component)]
struct GalaxyCell;

/// A star on a Galaxy brick. Every 2 to 6 s one random star twirls: grows to 2× in
/// 0.3 s, turns 180° over 1 s (in-back), spins on from 600°/s slowing by 200°/s², and
/// shrinks back after 2 s. Otherwise it breathes 1 ± 0.3.
#[derive(Component)]
struct Star {
    /// Seconds into a twirl, if one is playing.
    twirl: Option<f32>,
    spin: f32,
}

/// A Halloween brick once placed: the base texture goes greyish (guess; the original's
/// grey isn't read yet).
pub(crate) const HALLOWEEN_PLACED: Color = Color::srgb(0.62, 0.62, 0.66);

/// The thin dark purple line: (59, 22, 85) from Max's frame. Its
/// width, a twentieth of a cell, is a guess until the frame's scale is measured.
const GALAXY_LINE: Color = Color::srgb(0.231, 0.086, 0.333);
const GALAXY_LINE_WIDTH: f32 = 0.05;

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
                // Snow on top edges: 25% each. (No lights layer: it blinked over empty air,
                // since it covered the brick's whole box.)
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
                    touch: f32::INFINITY,
                });
            }
            BrickPack::Diamond => {
                brick_entity.insert(Gloss {
                    wait: 0.0,
                    time: -1.0,
                    seed,
                });
            }
            BrickPack::Galaxy => {
                // No texture: each cell is a window onto one shared starfield, edge to edge
                // (the original has no border). A thin dark purple line runs along each
                // cell's edges, so the lines show between cells (Max's frame).
                let background = art.image("brick_galaxy_background");
                let line = c * GALAXY_LINE_WIDTH;
                for &cell in cells {
                    let at = cell * c;
                    let inner = match &background {
                        Some(image) => Sprite {
                            image: image.clone(),
                            custom_size: Some(Vec2::splat(c)),
                            rect: Some(Rect::new(0.0, 0.0, 1.0, 1.0)),
                            ..default()
                        },
                        None => Sprite::from_color(Color::srgb(0.05, 0.02, 0.12), Vec2::splat(c)),
                    };
                    brick_entity.with_child((
                        GalaxyCell,
                        inner,
                        Transform::from_translation(at.extend(0.02)),
                    ));
                    // Only the piece's outside edge gets the line: a side with a neighbour
                    // cell in the same piece stays clear.
                    for (dir, size) in [
                        (Vec2::Y, Vec2::new(c, line)),
                        (Vec2::NEG_Y, Vec2::new(c, line)),
                        (Vec2::X, Vec2::new(line, c)),
                        (Vec2::NEG_X, Vec2::new(line, c)),
                    ] {
                        // Rotated pieces leave float error in their offsets, so a neighbour
                        // counts as touching within a tenth of a cell.
                        if cells.iter().any(|&o| (o - cell - dir).length() < 0.1) {
                            continue;
                        }
                        brick_entity.with_child((
                            Sprite::from_color(GALAXY_LINE, size),
                            Transform::from_translation((at + dir * c * 0.5).extend(0.025)),
                        ));
                    }
                }
                if let Some(star) = art.image("brick_galaxy_star") {
                    for _ in 0..cells.len().div_ceil(2) {
                        let cell =
                            cells[(roll(&mut seed) * cells.len() as f32) as usize % cells.len()];
                        let jitter = Vec2::new(roll(&mut seed) - 0.5, roll(&mut seed) - 0.5) * 0.6;
                        brick_entity.with_child((
                            Star {
                                twirl: None,
                                spin: 0.0,
                            },
                            Sprite {
                                image: star.clone(),
                                custom_size: Some(Vec2::splat(c * 0.35)),
                                ..default()
                            },
                            Transform::from_translation(((cell + jitter) * c).extend(0.03))
                                .with_rotation(Quat::from_rotation_z(
                                    roll(&mut seed) * std::f32::consts::TAU,
                                )),
                        ));
                    }
                }
            }
            _ => {}
        }
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

fn land_holograms(mut bricks: Query<&mut Glitch, Added<Placed>>) {
    for mut glitch in &mut bricks {
        glitch.touch = 0.0;
    }
}

/// The glitch shakes and tints the brick's sprite while it lasts (the original's
/// shader distortion, approximated, ?). A landed brick's see-through overrides only
/// the alpha, and only until the touch curve ends; collisions never change.
fn glitch_holograms(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut bricks: Query<(&mut Glitch, &Children)>,
    mut sprites: Query<(&mut Sprite, &mut Transform), With<CellSprite>>,
) {
    let dt = time.delta_secs();
    let touch_curve = &config.hologram.touch_alpha;
    let touch_end = touch_curve.last().map_or(0.0, |key| key.0);
    for (mut glitch, children) in &mut bricks {
        glitch.touch += dt;
        let touch = (glitch.touch < touch_end).then(|| keyed(touch_curve, glitch.touch));
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
        // Rises to full distortion halfway through, then falls back.
        let progress = if glitch.span > 0.0 {
            (1.0 - glitch.left / glitch.span).clamp(0.0, 1.0)
        } else {
            0.0
        };
        let amount = 1.0 - (2.0 * progress - 1.0).abs();
        // The glitch's width this frame; each cell takes its own share of it, so the
        // brick tears into slices rather than sliding whole.
        let width = if on { amount * 6.0 } else { 0.0 };
        for child in children.iter() {
            if let Ok((mut sprite, mut transform)) = sprites.get_mut(child) {
                // Hologram bricks are see-through and cyan-tinted, more so in a glitch.
                let (red, green, blue, alpha) = if on {
                    (0.5, 1.0, 1.0, config.hologram.glitch_alpha)
                } else {
                    (0.8, 1.0, 1.0, config.hologram.rest_alpha)
                };
                sprite.color = Color::srgba(red, green, blue, touch.unwrap_or(alpha));
                transform.translation.x = glitch_slice(width, roll(&mut seed));
            }
        }
        glitch.seed = seed;
    }
}

/// One cell's sideways tear during a glitch: `roll` (0 to 1) picks how far, within
/// `width` either way.
pub(crate) fn glitch_slice(width: f32, roll: f32) -> f32 {
    (roll - 0.5) * 2.0 * width
}

/// The gloss: a diagonal band of light sweeps across the brick's cells in 0.6 s, each
/// cell brightening as the band passes it (the original turns the gloss at a random
/// angle; the diagonal is a guess until a clip shows it).
fn gloss_diamonds(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut bricks: Query<(&mut Gloss, &Children)>,
    mut sprites: Query<(&mut Sprite, &Transform), With<CellSprite>>,
) {
    let dt = time.delta_secs();
    let band = config.cell_size * 0.8;
    for (mut gloss, children) in &mut bricks {
        let mut seed = gloss.seed;
        let sweep = if gloss.time >= 0.0 {
            gloss.time += dt;
            let t = gloss.time;
            if t >= 0.6 {
                gloss.time = -1.0;
                gloss.wait = between(&mut seed, 6.0, 15.0);
                None
            } else {
                Some(t / 0.6)
            }
        } else {
            gloss.wait -= dt;
            if gloss.wait <= 0.0 {
                gloss.time = 0.0;
            }
            None
        };
        gloss.seed = seed;
        let (mut lo, mut hi) = (f32::MAX, f32::MIN);
        for child in children.iter() {
            if let Ok((_, transform)) = sprites.get(child) {
                let at = transform.translation.x + transform.translation.y;
                lo = lo.min(at);
                hi = hi.max(at);
            }
        }
        for child in children.iter() {
            if let Ok((mut sprite, transform)) = sprites.get_mut(child) {
                let glow = sweep.map_or(0.0, |progress| {
                    let at = transform.translation.x + transform.translation.y;
                    gloss_glow(at, lo, hi, progress, band)
                });
                sprite.color = Color::WHITE.mix(&Color::srgb(1.3, 1.3, 1.3), glow);
            }
        }
    }
}

/// How bright a cell at diagonal `at` is while the gloss sweep is `progress` (0 to 1)
/// through a brick spanning `lo` to `hi`: up to 0.7 at the front, fading over `band`.
pub(crate) fn gloss_glow(at: f32, lo: f32, hi: f32, progress: f32, band: f32) -> f32 {
    let front = (lo - band) + ((hi - lo) + 2.0 * band) * progress.clamp(0.0, 1.0);
    0.7 * (1.0 - ((at - front).abs() / band).min(1.0))
}

/// Every Galaxy cell shows the part of one big scrolling starfield behind it, so all
/// bricks share the same sky. The field repeats, so a window may run past the picture's
/// edge without a seam.
fn scroll_galaxy(
    time: Res<Time>,
    config: Res<GameConfig>,
    mut images: ResMut<Assets<Image>>,
    mut cells: Query<(&mut Sprite, &GlobalTransform), With<GalaxyCell>>,
) {
    let t = time.elapsed_secs();
    for (mut sprite, at) in &mut cells {
        repeat_starfield(&mut images, &sprite.image);
        let Some(size) = images.get(&sprite.image).map(|i| i.size_f32()) else {
            continue;
        };
        sprite.rect = Some(galaxy_window(
            at.translation().truncate(),
            config.cell_size,
            size,
            t,
        ));
    }
}

/// The texels of the shared starfield one cell shows. A cell is 64 texels of the field
/// wide and the field drifts (0.05, -0.07) of the texture a second. Positions are not
/// wrapped, so neighbouring cells always line up.
pub(crate) fn galaxy_window(world: Vec2, cell: f32, texture: Vec2, t: f32) -> Rect {
    let scale = 64.0 / cell;
    let side = 64.0;
    let p = Vec2::new(world.x * scale, -world.y * scale) + texture * Vec2::new(0.05, -0.07) * t;
    Rect::new(p.x, p.y, p.x + side, p.y + side)
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
    fn the_gloss_band_sweeps_from_one_end_of_the_brick_to_the_other() {
        // Brick diagonals from 0 to 4 cells, band 1 cell wide.
        let (lo, hi, band) = (0.0, 4.0, 1.0);
        assert!(gloss_glow(0.0, lo, hi, 0.0, band) < 0.01, "not yet at the start");
        assert!((gloss_glow(2.0, lo, hi, 0.5, band) - 0.7).abs() < 1e-5, "centre at mid sweep");
        assert!(gloss_glow(4.0, lo, hi, 0.0, band) < 0.01, "the far end is still ahead");
        assert!(gloss_glow(0.0, lo, hi, 1.0, band) < 0.01, "the sweep has passed the near end");
    }

    #[test]
    fn neighbouring_cells_line_up_across_the_whole_field() {
        // One cell to the right is 64 texels further along, wherever it is, even past
        // the texture's edge (the field repeats). The drift is the same for both.
        let texture = Vec2::new(1024.0, 512.0);
        for world in [Vec2::new(-3.0, 1.0), Vec2::new(40.0, -9.5), Vec2::new(900.0, 0.0)] {
            for t in [0.0, 3.7, 120.0] {
                let a = galaxy_window(world, 2.0, texture, t);
                let b = galaxy_window(world + Vec2::new(2.0, 0.0), 2.0, texture, t);
                assert!((b.min.x - a.min.x - 64.0).abs() < 1e-3);
                assert!((b.min.y - a.min.y).abs() < 1e-3);
                let c = galaxy_window(world + Vec2::new(0.0, 2.0), 2.0, texture, t);
                assert!((c.min.y - a.min.y + 64.0).abs() < 1e-3, "one cell up is 64 texels up");
            }
        }
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
    fn hologram_touch_dips_then_ends_at_rest_alpha() {
        let curve = crate::config::HologramLook::default().touch_alpha;
        assert_eq!(
            curve.last().unwrap().1,
            0.85,
            "touch hands back to the resting alpha"
        );
        assert!(keyed(&curve, 0.2) < 0.6, "see-through while touching");
        assert!(
            keyed(&curve, 0.4) > 0.9,
            "flashes brighter before going solid"
        );
    }
}
