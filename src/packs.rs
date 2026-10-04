//! What each brick pack adds on top of its textures (docs/research/skins.md):
//! Christmas lights that blink and snow caps on top edges, Halloween decorations
//! that pulse, the Hologram glitch, the Diamond gloss and the Galaxy starfield.
//! The bricks themselves are drawn by `visuals::dress_bricks`; this runs right after
//! it and adds children, so a re-dressed brick gets its decorations again.
//! Spring's grass and eggs aren't staged yet, so Spring shows its textures only.
//! Looks only: nothing here touches gameplay.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{BrickPack, GameConfig};
use crate::gameplay::{Brick, TowerSkin};
use crate::spells::{Grassy, Stone, Vined};
use crate::visuals::{CellSprite, dress_bricks};

pub struct PacksPlugin;

impl Plugin for PacksPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                decorate_bricks.after(dress_bricks),
                blink_lights,
                grow_snow,
                pulse_decorations,
                glitch_holograms,
                gloss_diamonds,
                scroll_galaxy,
                twirl_stars,
            ),
        );
    }
}

/// A Christmas lights layer: on and off, offset per brick.
#[derive(Component)]
struct Blink {
    offset: f32,
}

/// A snow cap appearing within 0.3 s.
#[derive(Component)]
struct Snow {
    time: f32,
    full: Vec3,
}

/// A Halloween decoration that pulses to 0.8× every 5 to 15 s.
#[derive(Component)]
struct Pulse {
    wait: f32,
    time: f32,
    seed: u32,
}

/// A Hologram brick: glitches for 0.1 to 0.5 s every 6 to 15 s.
#[derive(Component)]
struct Glitch {
    wait: f32,
    left: f32,
    seed: u32,
}

/// A Diamond brick: a gloss flash every few seconds (?).
#[derive(Component)]
struct Gloss {
    wait: f32,
    time: f32,
    seed: u32,
}

/// One cell's window onto the shared Galaxy starfield.
#[derive(Component)]
struct GalaxyCell;

/// A star on a Galaxy brick; one twirls every 2 to 6 s.
#[derive(Component)]
struct Star {
    wait: f32,
    spin: f32,
    seed: u32,
}

/// The Galaxy border colour (docs/research/skins.md).
const GALAXY_BORDER: Color = Color::srgb(0.12, 0.03, 0.24);

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
        Or<(Changed<Brick>, Added<Stone>, Added<Vined>, Added<Grassy>)>,
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
        let shape = &config.pieces[brick.piece].name;
        let c = config.cell_size * brick.scale;
        let cells = &brick.cells;
        let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
        let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
        let size = (max - min + Vec2::ONE) * c;
        let center = (min + max) * 0.5 * c;
        let mut seed = (entity.to_bits() as u32).wrapping_mul(2_654_435_761) | 1;
        let mut brick_entity = commands.entity(entity);
        match skin.pack {
            BrickPack::Christmas => {
                if let Some(lights) = art.image(&format!("brick_christmas_lights_{shape}")) {
                    brick_entity.with_child((
                        Blink {
                            offset: roll(&mut seed) * 2.0,
                        },
                        Sprite {
                            image: lights,
                            custom_size: Some(size),
                            ..default()
                        },
                        Transform::from_translation(center.extend(0.05)),
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
                            time: 0.0,
                            seed,
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
                    wait: between(&mut seed, 6.0, 15.0),
                    left: 0.0,
                    seed,
                });
            }
            BrickPack::Diamond => {
                brick_entity.insert(Gloss {
                    wait: between(&mut seed, 2.0, 5.0),
                    time: -1.0,
                    seed,
                });
            }
            BrickPack::Galaxy => {
                // No texture: each cell is a window onto one shared starfield inside
                // a dark purple border, with a few stars on top.
                let background = art.image("brick_galaxy_background");
                for &cell in cells {
                    let at = cell * c;
                    brick_entity.with_child((
                        Sprite::from_color(GALAXY_BORDER, Vec2::splat(c)),
                        Transform::from_translation(at.extend(0.01)),
                    ));
                    let inner = match &background {
                        Some(image) => Sprite {
                            image: image.clone(),
                            custom_size: Some(Vec2::splat(c * 0.84)),
                            rect: Some(Rect::new(0.0, 0.0, 1.0, 1.0)),
                            ..default()
                        },
                        None => {
                            Sprite::from_color(Color::srgb(0.05, 0.02, 0.12), Vec2::splat(c * 0.84))
                        }
                    };
                    brick_entity.with_child((
                        GalaxyCell,
                        inner,
                        Transform::from_translation(at.extend(0.02)),
                    ));
                }
                if let Some(star) = art.image("brick_galaxy_star") {
                    for _ in 0..cells.len().div_ceil(2) {
                        let cell =
                            cells[(roll(&mut seed) * cells.len() as f32) as usize % cells.len()];
                        let jitter = Vec2::new(roll(&mut seed) - 0.5, roll(&mut seed) - 0.5) * 0.6;
                        brick_entity.with_child((
                            Star {
                                wait: between(&mut seed, 2.0, 6.0),
                                spin: 0.0,
                                seed,
                            },
                            Sprite {
                                image: star.clone(),
                                custom_size: Some(Vec2::splat(c * 0.35)),
                                ..default()
                            },
                            Transform::from_translation(((cell + jitter) * c).extend(0.03)),
                        ));
                    }
                }
            }
            _ => {}
        }
    }
}

fn blink_lights(time: Res<Time>, mut lights: Query<(&Blink, &mut Visibility)>) {
    let t = time.elapsed_secs();
    for (blink, mut visibility) in &mut lights {
        // On and off each half second (?), offset per brick.
        let on = ((t + blink.offset) % 1.0) < 0.5;
        visibility.set_if_neq(if on {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        });
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

fn pulse_decorations(time: Res<Time>, mut items: Query<(&mut Pulse, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut pulse, mut transform) in &mut items {
        pulse.wait -= dt;
        if pulse.wait > 0.0 {
            continue;
        }
        // Down to 0.8× and back over 0.4 s (?), then wait again.
        pulse.time += dt;
        let s = (pulse.time / 0.4).min(1.0);
        let scale = 1.0 - 0.2 * (s * std::f32::consts::PI).sin();
        transform.scale = Vec3::splat(scale);
        if s >= 1.0 {
            pulse.time = 0.0;
            let mut seed = pulse.seed;
            pulse.wait = between(&mut seed, 5.0, 15.0);
            pulse.seed = seed;
        }
    }
}

/// The glitch shakes and tints the brick's sprite while it lasts (the original's
/// shader distortion, approximated, ?).
fn glitch_holograms(
    time: Res<Time>,
    mut bricks: Query<(&mut Glitch, &Children)>,
    mut sprites: Query<(&mut Sprite, &mut Transform), With<CellSprite>>,
) {
    let dt = time.delta_secs();
    for (mut glitch, children) in &mut bricks {
        let mut seed = glitch.seed;
        if glitch.left > 0.0 {
            glitch.left -= dt;
        } else {
            glitch.wait -= dt;
            if glitch.wait <= 0.0 {
                glitch.left = between(&mut seed, 0.1, 0.5);
                glitch.wait = between(&mut seed, 6.0, 15.0);
            }
        }
        let on = glitch.left > 0.0;
        let shift = if on {
            (roll(&mut seed) - 0.5) * 6.0
        } else {
            0.0
        };
        glitch.seed = seed;
        for child in children.iter() {
            if let Ok((mut sprite, mut transform)) = sprites.get_mut(child) {
                // Hologram bricks are see-through and cyan-tinted, more so in a glitch.
                sprite.color = if on {
                    Color::srgba(0.5, 1.0, 1.0, 0.55)
                } else {
                    Color::srgba(0.8, 1.0, 1.0, 0.85)
                };
                transform.translation.x = shift;
            }
        }
    }
}

/// A short bright flash every few seconds stands in for the gloss sweep (?).
fn gloss_diamonds(
    time: Res<Time>,
    mut bricks: Query<(&mut Gloss, &Children)>,
    mut sprites: Query<&mut Sprite, With<CellSprite>>,
) {
    let dt = time.delta_secs();
    for (mut gloss, children) in &mut bricks {
        let mut seed = gloss.seed;
        let glow = if gloss.time >= 0.0 {
            gloss.time += dt;
            let s = (gloss.time / 0.5).min(1.0);
            if s >= 1.0 {
                gloss.time = -1.0;
                gloss.wait = between(&mut seed, 2.0, 5.0);
            }
            (s * std::f32::consts::PI).sin() * 0.45
        } else {
            gloss.wait -= dt;
            if gloss.wait <= 0.0 {
                gloss.time = 0.0;
            }
            0.0
        };
        gloss.seed = seed;
        for child in children.iter() {
            if let Ok(mut sprite) = sprites.get_mut(child) {
                sprite.color = Color::WHITE.mix(&Color::srgb(1.3, 1.3, 1.3), glow);
            }
        }
    }
}

/// Every Galaxy cell shows the part of one big scrolling starfield behind it, so all
/// bricks share the same sky.
fn scroll_galaxy(
    time: Res<Time>,
    config: Res<GameConfig>,
    images: Res<Assets<Image>>,
    mut cells: Query<(&mut Sprite, &GlobalTransform), With<GalaxyCell>>,
) {
    let t = time.elapsed_secs();
    for (mut sprite, at) in &mut cells {
        let Some(size) = images.get(&sprite.image).map(|i| i.size_f32()) else {
            continue;
        };
        // One field cell maps to 64 texture pixels (?), drifting slowly.
        let scale = 64.0 / config.cell_size;
        let p = at.translation().truncate() * scale + Vec2::new(t * 6.0, t * 2.0);
        let side = 64.0 * 0.84;
        let x = p.x.rem_euclid((size.x - side).max(1.0));
        let y = (-p.y).rem_euclid((size.y - side).max(1.0));
        sprite.rect = Some(Rect::new(x, y, x + side, y + side));
    }
}

fn twirl_stars(time: Res<Time>, mut stars: Query<(&mut Star, &mut Transform)>) {
    let dt = time.delta_secs();
    for (mut star, mut transform) in &mut stars {
        let mut seed = star.seed;
        if star.spin > 0.0 {
            star.spin -= dt;
            transform.rotate_z(std::f32::consts::TAU * dt);
        } else {
            star.wait -= dt;
            if star.wait <= 0.0 {
                star.spin = 1.0;
                star.wait = between(&mut seed, 2.0, 6.0);
            }
        }
        star.seed = seed;
        // A gentle twinkle.
        let twinkle = 0.85 + 0.15 * (time.elapsed_secs() * 3.0 + seed as f32 * 0.001).sin();
        transform.scale = Vec3::splat(twinkle);
    }
}
