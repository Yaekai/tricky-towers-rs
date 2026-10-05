//! Small particle effects from the gap scan (docs/research/gap-scan.md 3-4):
//! - a white poof where every new brick appears (flash, puffs, star sparks);
//! - impact dust where a brick lands;
//! - when a brick is lost: a lilac mist burst (Race, Survival) or desert dust (Puzzle),
//!   and the big red lost cross where it left the screen, spinning and shrinking;
//! - the WAVE N message when a Survival curse wave starts.
//!
//! Sizes are in cells (the original's units). The readout's speeds (60 to 150) would
//! throw puffs dozens of cells, so they are taken as tenths of a cell a second (?).
//! Looks only.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{ActiveBrick, BrickLost, Match, Phase, Placed, Tower, TowerState};
use crate::visuals::ViewSize;

pub struct ParticlesPlugin;

impl Plugin for ParticlesPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                spawn_poofs,
                landing_dust,
                lost_bricks,
                wave_messages,
                move_particles,
                spin_crosses,
                fade_wave_messages,
            ),
        );
    }
}

/// One particle: velocity (pixels/s), gravity (pixels/s², up positive), age and life,
/// size from start to end, and whether it fades out.
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    gravity: f32,
    time: f32,
    life: f32,
    size: (f32, f32),
    fade: bool,
}

/// The lost cross: spins a full turn (eased in, after 0.2 s) and shrinks to 30% over
/// 0.8 s.
#[derive(Component)]
struct LostCross {
    time: f32,
    size: f32,
}

/// The WAVE N message, shown for a moment (?).
#[derive(Component)]
struct WaveMessage {
    time: f32,
}

fn roll(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 10_000) as f32 / 10_000.0
}

/// What a burst looks like: picture, colour, count, speed, size, life, gravity (all in
/// cells), spread (radians around `up`, or a full circle) and draw depth.
struct Burst {
    image: &'static str,
    color: Color,
    count: u32,
    speed: f32,
    size: f32,
    life: f32,
    gravity: f32,
    spread: Option<f32>,
    z: f32,
}

#[allow(clippy::too_many_arguments)]
fn burst(commands: &mut Commands, art: &Art, c: f32, at: Vec2, seed: &mut u32, b: Burst) {
    let image = art
        .image(b.image)
        .or_else(|| art.image("mist_puff"))
        .or_else(|| art.image("explosion_flash"));
    for _ in 0..b.count {
        let angle = match b.spread {
            Some(spread) => std::f32::consts::FRAC_PI_2 + (roll(seed) - 0.5) * spread,
            None => roll(seed) * std::f32::consts::TAU,
        };
        let speed = b.speed * (0.6 + 0.4 * roll(seed)) * c;
        let size = b.size * c;
        let sprite = match &image {
            Some(image) => Sprite {
                image: image.clone(),
                color: b.color,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            None => Sprite::from_color(b.color.with_alpha(0.7), Vec2::splat(size * 0.5)),
        };
        commands.spawn((
            Particle {
                velocity: Vec2::from_angle(angle) * speed,
                gravity: b.gravity * c,
                time: 0.0,
                life: b.life,
                size: (size, size * 0.6),
                fade: true,
            },
            sprite,
            Transform::from_translation(at.extend(b.z)),
        ));
    }
}

/// A white flash, eight puffs and five star sparks where a new brick appears.
fn spawn_poofs(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<(Entity, &Transform), Added<ActiveBrick>>,
) {
    let c = config.cell_size;
    for (entity, transform) in &bricks {
        let at = transform.translation.truncate();
        let mut seed = (entity.to_bits() as u32) | 1;
        burst(
            &mut commands,
            &art,
            c,
            at,
            &mut seed,
            Burst {
                image: "explosion_flash",
                color: Color::WHITE,
                count: 1,
                speed: 0.0,
                size: 5.0,
                life: 0.07,
                gravity: 0.0,
                spread: None,
                z: 8.0,
            },
        );
        burst(
            &mut commands,
            &art,
            c,
            at,
            &mut seed,
            Burst {
                image: "mist_puff",
                color: Color::WHITE,
                count: 8,
                speed: 60.0 / 10.0,
                size: 1.8,
                life: 0.5,
                gravity: 0.0,
                spread: None,
                z: 8.0,
            },
        );
        burst(
            &mut commands,
            &art,
            c,
            at,
            &mut seed,
            Burst {
                image: "spark_star",
                color: Color::WHITE,
                count: 5,
                speed: 60.0 / 10.0,
                size: 0.6,
                life: 0.5,
                gravity: 0.0,
                spread: None,
                z: 8.1,
            },
        );
    }
}

/// Three small dust puffs under a brick as it lands.
fn landing_dust(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<(Entity, &Transform, &crate::gameplay::Brick), Added<Placed>>,
) {
    let c = config.cell_size;
    for (entity, transform, brick) in &bricks {
        let low = brick.cells.iter().fold(f32::MAX, |m, p| m.min(p.y));
        let at = transform.translation.truncate() + Vec2::Y * (low - 0.5) * c * brick.scale;
        let mut seed = (entity.to_bits() as u32).wrapping_mul(7919) | 1;
        burst(
            &mut commands,
            &art,
            c,
            at,
            &mut seed,
            Burst {
                image: "brick_impact_dust",
                color: Color::WHITE,
                count: 3,
                speed: 4.0,
                size: 0.5,
                life: 0.8,
                gravity: 0.0,
                spread: Some(2.6),
                z: 7.0,
            },
        );
    }
}

/// A lost brick: mist (Race, Survival) or desert dust (Puzzle) where it went, and the lost
/// cross at the edge of the screen.
#[allow(clippy::too_many_arguments)]
fn lost_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    view_size: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), With<Camera2d>>,
    mut lost: MessageReader<BrickLost>,
) {
    let c = config.cell_size;
    let (centre, size) = match camera.single() {
        Ok((t, Projection::Orthographic(o))) => (t.translation.truncate(), view_size.0 * o.scale),
        _ => (Vec2::ZERO, view_size.0),
    };
    for (n, brick) in lost.read().enumerate() {
        if game.phase != Phase::Playing {
            continue;
        }
        let mut seed = (brick.at.x.to_bits() ^ (n as u32 * 7919)) | 1;
        // Where it left the screen: 2 cells in from the side, 0.3 up from the bottom.
        let half = size * 0.5;
        let edge = Vec2::new(
            brick
                .at
                .x
                .clamp(centre.x - half.x + 2.0 * c, centre.x + half.x - 2.0 * c),
            brick.at.y.max(centre.y - half.y + 0.3 * c),
        );
        if game.mode.rules() == ModeKind::Puzzle {
            let dust = Color::srgb(1.0, 0.70, 0.44);
            burst(
                &mut commands,
                &art,
                c,
                edge,
                &mut seed,
                Burst {
                    image: "mist_puff",
                    color: dust.with_alpha(0.8),
                    count: 4,
                    speed: 80.0 / 10.0,
                    size: 5.0,
                    life: 3.0,
                    gravity: 0.0,
                    spread: None,
                    z: 9.0,
                },
            );
            burst(
                &mut commands,
                &art,
                c,
                edge,
                &mut seed,
                Burst {
                    image: "spark_star",
                    color: Color::srgb(1.0, 0.45, 0.2),
                    count: 5,
                    speed: 150.0 / 10.0,
                    size: 0.6,
                    life: 0.7,
                    gravity: 0.0,
                    spread: None,
                    z: 9.1,
                },
            );
        } else {
            burst(
                &mut commands,
                &art,
                c,
                edge,
                &mut seed,
                Burst {
                    image: "explosion_flash",
                    color: Color::WHITE,
                    count: 1,
                    speed: 0.0,
                    size: 4.0,
                    life: 0.15,
                    gravity: 0.0,
                    spread: None,
                    z: 9.2,
                },
            );
            burst(
                &mut commands,
                &art,
                c,
                edge,
                &mut seed,
                Burst {
                    image: "mist_puff",
                    color: Color::WHITE,
                    count: 8,
                    speed: 100.0 / 10.0,
                    size: 1.0,
                    life: 0.5,
                    gravity: 0.0,
                    spread: None,
                    z: 9.1,
                },
            );
            burst(
                &mut commands,
                &art,
                c,
                edge,
                &mut seed,
                Burst {
                    image: "mist_puff",
                    color: Color::WHITE,
                    count: 8,
                    speed: 50.0 / 10.0,
                    size: 0.8,
                    life: 2.0,
                    gravity: 2.0,
                    spread: Some(1.2),
                    z: 9.1,
                },
            );
            burst(
                &mut commands,
                &art,
                c,
                edge,
                &mut seed,
                Burst {
                    image: "mist_puff",
                    color: Color::srgba(0.94, 0.91, 1.0, 0.7),
                    count: 6,
                    speed: 15.0 / 10.0,
                    size: 4.0,
                    life: 2.0,
                    gravity: 0.5,
                    spread: Some(1.6),
                    z: 9.0,
                },
            );
        }
        // The lost cross.
        let size = c * 2.0;
        let sprite = match art.image("lost_cross") {
            Some(image) => Sprite {
                image,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            None => Sprite::from_color(Color::srgb(0.9, 0.15, 0.15), Vec2::new(size, size * 0.2)),
        };
        commands.spawn((
            LostCross { time: 0.0, size },
            sprite,
            Transform::from_translation(edge.extend(9.5)),
        ));
    }
}

fn move_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut particles: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut p, mut transform, mut sprite) in &mut particles {
        p.time += dt;
        if p.time >= p.life {
            commands.entity(entity).despawn();
            continue;
        }
        p.velocity.y += p.gravity * dt;
        transform.translation += (p.velocity * dt).extend(0.0);
        let s = p.time / p.life;
        let size = p.size.0 + (p.size.1 - p.size.0) * s;
        if sprite.custom_size.is_some() {
            sprite.custom_size = Some(Vec2::splat(size));
        }
        if p.fade {
            let alpha = sprite.color.alpha();
            sprite.color.set_alpha((alpha).min(1.0 - s));
        }
    }
}

fn spin_crosses(
    mut commands: Commands,
    time: Res<Time>,
    mut crosses: Query<(Entity, &mut LostCross, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut cross, mut transform, mut sprite) in &mut crosses {
        cross.time += time.delta_secs();
        let t = cross.time;
        if t >= 1.2 {
            commands.entity(entity).despawn();
            continue;
        }
        // A full turn eased in after 0.2 s, shrinking to 30% over 0.8 s.
        let spin = ((t - 0.2) / 0.8).clamp(0.0, 1.0);
        transform.rotation = Quat::from_rotation_z(-std::f32::consts::TAU * spin * spin);
        let scale = 1.0 - 0.7 * (t / 0.8).min(1.0);
        if let Some(size) = sprite.custom_size.as_mut() {
            *size = size.normalize_or_zero() * cross.size * scale * std::f32::consts::SQRT_2;
        }
        transform.scale = Vec3::ONE;
        if t > 0.8 {
            sprite.color.set_alpha(1.0 - (t - 0.8) / 0.4);
        }
    }
}

/// When a Survival curse wave starts: WAVE and its number on a row of cloud pieces.
#[allow(clippy::type_complexity)]
fn wave_messages(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    towers: Query<(&Tower, &TowerState)>,
    mut seen: Local<(u64, bool, u32)>,
) {
    if game.phase == Phase::Menu || seen.0 != game.round {
        *seen = (game.round, false, 0);
    }
    let wave = towers
        .iter()
        .any(|(t, s)| t.is_first_player() && s.wave_left > 0.0);
    if wave && !seen.1 {
        seen.2 += 1;
        let number = seen.2;
        let mut root = commands.spawn((
            WaveMessage { time: 0.0 },
            Node {
                position_type: PositionType::Absolute,
                top: Val::Percent(30.0),
                width: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            GlobalZIndex(18),
        ));
        root.with_children(|row| {
            for i in 1..=4 {
                if let Some(cloud) = art.image(&format!("wave_cloud_{i}")) {
                    row.spawn((
                        ImageNode::new(cloud),
                        Node {
                            width: Val::Vh(12.0),
                            height: Val::Vh(9.0),
                            margin: UiRect::horizontal(Val::Vh(-1.5)),
                            ..default()
                        },
                    ));
                }
            }
            row.spawn((
                Node {
                    position_type: PositionType::Absolute,
                    ..default()
                },
                Text::new(format!("WAVE {number}")),
                TextFont {
                    font_size: FontSize::Vh(7.0),
                    ..default()
                },
                TextColor(Color::WHITE),
            ));
        });
    }
    seen.1 = wave;
}

fn fade_wave_messages(
    mut commands: Commands,
    time: Res<Time>,
    mut messages: Query<(Entity, &mut WaveMessage)>,
) {
    for (entity, mut message) in &mut messages {
        message.time += time.delta_secs();
        // Shown for 2 s (?).
        if message.time >= 2.0 {
            commands.entity(entity).despawn();
        }
    }
}
