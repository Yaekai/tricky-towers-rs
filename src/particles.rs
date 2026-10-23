//! Small particle effects from the gap scan (docs/research/gap-scan.md 3-4):
//! - a white poof where every new brick appears (flash, puffs, star sparks);
//! - impact dust where a brick lands;
//! - gravel trickling from every petrified brick in view when a brick is placed
//!   on its tower (GravelEffect);
//! - when a brick is lost: a lilac mist burst (Race, Survival) or desert dust (Puzzle),
//!   and the big red lost cross where it left the screen, spinning and shrinking;
//! - the WAVE N message when a Survival curse wave starts.
//!
//! Sizes and speeds are in cells (the original's units). Most bursts start fast and
//! are clamped (each prefab's Limit Velocity: a top speed and the share of the excess
//! taken off every frame), so a 60 cells/s puff settles within a cell or two.
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
                gravel,
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
    /// Limit Velocity: top speed (pixels/s) and dampen per 60 Hz frame.
    clamp: (f32, f32),
}

/// The lost cross: spins a full turn (eased in, after 0.2 s) and shrinks to 30% over
/// 0.8 s.
#[derive(Component)]
struct LostCross {
    time: f32,
    size: f32,
    /// BlinkingCrossEffect: from where the brick left the view to the heart the lost life
    /// empties, over 0.8 s (in-quad; the axis the brick didn't leave by starts 0.3 s late),
    /// turning a full turn its side's way from 0.2 s.
    from: Vec2,
    tower: usize,
    heart: Option<u32>,
    side: f32,
    bottom: bool,
}

/// A front cloud of the WAVE N message, drifting by its own keys (`moments.wave_clouds`).
#[derive(Component)]
struct WaveCloud(usize);

/// The WAVE N message, sliding through (`moments.wave_x`, `wave_scale`).
#[derive(Component)]
struct WaveMessage {
    time: f32,
}

pub(crate) fn roll(seed: &mut u32) -> f32 {
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
    /// Gravity is picked between `gravity` and `gravity_to` for each particle (a Force
    /// module between two curves, as GRAVEL's).
    gravity: f32,
    gravity_to: f32,
    spread: Option<f32>,
    z: f32,
    /// Limit Velocity (cells/s, dampen); `NO_CLAMP` when the prefab has none.
    clamp: (f32, f32),
}

const NO_CLAMP: (f32, f32) = (f32::MAX, 0.0);

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
                gravity: (b.gravity + (b.gravity_to - b.gravity) * roll(seed)) * c,
                time: 0.0,
                life: b.life,
                size: (size, size * 0.6),
                fade: true,
                clamp: (b.clamp.0 * c, b.clamp.1),
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
                clamp: NO_CLAMP,
                size: 5.0,
                life: 0.07,
                gravity: 0.0,
                gravity_to: 0.0,
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
                speed: 60.0,
                clamp: (0.5, 0.28),
                size: 1.8,
                life: 0.5,
                gravity: 0.0,
                gravity_to: 0.0,
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
                speed: 60.0,
                clamp: (0.5, 0.3),
                size: 0.6,
                life: 0.5,
                gravity: 0.0,
                gravity_to: 0.0,
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
    mut roofs: MessageReader<crate::gameplay::RoofPlaced>,
) {
    let c = config.cell_size;
    // The roof set down raises the same dust (ROOF_PLACED's BrickImpactDustEffect).
    let landings: Vec<(Entity, Vec2)> = bricks
        .iter()
        .map(|(entity, transform, brick)| {
            let low = brick.cells.iter().fold(f32::MAX, |m, p| m.min(p.y));
            (entity, transform.translation.truncate() + Vec2::Y * (low - 0.5) * c * brick.scale)
        })
        .chain(roofs.read().map(|r| (r.tower, r.at)))
        .collect();
    for (entity, at) in landings {
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
                clamp: (0.5, 1.0),
                size: 0.5,
                life: 0.8,
                gravity: 0.0,
                gravity_to: 0.0,
                spread: Some(2.6),
                z: 7.0,
            },
        );
    }
}

/// When a brick is placed, each petrified brick of that tower in view sheds a few
/// dark-red bits across its width (GRAVEL: colour (0.22, 0.11, 0.11), size 0.5, 1 s; its
/// start speed points at the camera, and a world Force of 10 to 20 cells/s² pulls it down).
#[allow(clippy::type_complexity)]
fn gravel(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    view_size: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>,
    placed: Query<&crate::gameplay::Brick, Added<Placed>>,
    mut roofs: MessageReader<crate::gameplay::RoofPlaced>,
    stones: Query<(Entity, &Transform, &crate::gameplay::Brick), With<crate::spells::Stone>>,
) {
    let c = config.cell_size;
    let (centre, size) = match camera.single() {
        Ok((t, Projection::Orthographic(o))) => (t.translation.truncate(), view_size.0 * o.scale),
        _ => return,
    };
    // ROOF_PLACED sheds gravel too (its GravelEffect).
    let towers: Vec<Entity> = placed.iter().map(|b| b.tower).chain(roofs.read().map(|r| r.tower)).collect();
    for tower in towers {
        for (entity, transform, stone) in &stones {
            let at = transform.translation.truncate();
            if stone.tower != tower || (at.y - centre.y).abs() > size.y * 0.5 {
                continue;
            }
            let (lo, hi) = stone
                .cells
                .iter()
                .fold((f32::MAX, f32::MIN), |(lo, hi), p| {
                    (lo.min(p.x), hi.max(p.x))
                });
            let width = (hi - lo + 1.0) * stone.scale;
            let mut seed = (entity.to_bits() as u32).wrapping_mul(2_654_435_761) | 1;
            burst(
                &mut commands,
                &art,
                c,
                at,
                &mut seed,
                Burst {
                    image: "gravel",
                    color: Color::srgb(0.22, 0.11, 0.11),
                    count: 3,
                    speed: 0.0,
                    clamp: NO_CLAMP,
                    size: 0.5,
                    life: 1.0,
                    gravity: -10.0,
                    gravity_to: -20.0,
                    spread: Some(width),
                    z: 7.0,
                },
            );
        }
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
    camera: Query<(&Transform, &Projection), With<crate::visuals::MainCamera>>,
    mut lost: MessageReader<BrickLost>,
    towers: Query<(&Tower, &TowerState)>,
) {
    // The heart a lost life empties: the one at the lives now left.
    let lives: std::collections::HashMap<usize, u32> =
        towers.iter().filter_map(|(t, s)| Some((t.index, s.lives?))).collect();
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
        // Race listens to none of the lost-brick triggers: only the splash.
        if game.mode.rules() == ModeKind::Race {
            continue;
        }
        if game.mode.rules() == ModeKind::Puzzle {
            // BRICK_LOST_PUZZLE's desert dust, and again for LIFE_LOST_PUZZLE (trials).
            for _ in 0..1 + brick.life as u32 {
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
                        speed: 80.0,
                        clamp: (3.0, 0.2),
                        size: 5.0,
                        life: 3.0,
                        gravity: 0.0,
                        gravity_to: 0.0,
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
                        speed: 150.0,
                        clamp: (1.0, 0.4),
                        size: 0.6,
                        life: 0.7,
                        gravity: 0.0,
                        gravity_to: 0.0,
                        spread: None,
                        z: 9.1,
                    },
                );
            }
        } else {
            // BRICK_LOST: the small mist (BrickLostMistSmall) for every brick.
            for (speed, size, gravity, color, count, spread, z) in [
                (30.0, 0.8, 2.0, Color::WHITE, 8, 1.2, 9.1),
                (15.0, 4.0, 0.5, Color::srgba(0.94, 0.91, 1.0, 0.7), 6, 1.6, 9.0),
            ] {
                burst(
                    &mut commands,
                    &art,
                    c,
                    edge,
                    &mut seed,
                    Burst {
                        image: "mist_puff",
                        color,
                        count,
                        speed,
                        clamp: (2.0, 0.15),
                        size,
                        life: 2.0,
                        gravity,
                        gravity_to: gravity,
                        spread: Some(spread),
                        z,
                    },
                );
            }
        }
        // LIFE_LOST: the big mist (BrickLostMist) and the lost cross.
        if !brick.life {
            continue;
        }
        if game.mode.rules() != ModeKind::Puzzle {
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
                    clamp: NO_CLAMP,
                    size: 4.0,
                    life: 0.15,
                    gravity: 0.0,
                    gravity_to: 0.0,
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
                    speed: 100.0,
                    clamp: (0.5, 0.28),
                    size: 1.0,
                    life: 0.5,
                    gravity: 0.0,
                    gravity_to: 0.0,
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
                    speed: 50.0,
                    clamp: (2.0, 0.15),
                    size: 0.8,
                    life: 2.0,
                    gravity: 2.0,
                    gravity_to: 2.0,
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
                    speed: 15.0,
                    clamp: (2.0, 0.15),
                    size: 4.0,
                    life: 2.0,
                    gravity: 0.5,
                    gravity_to: 0.5,
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
            LostCross {
                time: 0.0,
                size,
                from: edge,
                tower: brick.tower,
                heart: lives.get(&brick.tower).copied(),
                side: if brick.at.x > centre.x { 1.0 } else { -1.0 },
                bottom: brick.at.x > centre.x - half.x && brick.at.x < centre.x + half.x,
            },
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
        let speed = p.velocity.length();
        if speed > p.clamp.0 {
            // Unity takes `dampen` of the excess off each frame (60 Hz assumed).
            let excess = (speed - p.clamp.0) * (1.0 - p.clamp.1).powf(dt * 60.0);
            let k = (p.clamp.0 + excess) / speed;
            p.velocity *= k;
        }
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
    hearts: Res<crate::hud::HeartSpots>,
    mut crosses: Query<(Entity, &mut LostCross, &mut Transform, &mut Sprite)>,
) {
    for (entity, mut cross, mut transform, mut sprite) in &mut crosses {
        cross.time += time.delta_secs();
        let t = cross.time;
        // With a heart to go to it is gone when it gets there.
        let target = cross.heart.and_then(|h| hearts.0.get(&(cross.tower, h)).copied());
        if t >= 1.2 || (target.is_some() && t >= 0.8) {
            commands.entity(entity).despawn();
            continue;
        }
        if let Some(to) = target {
            let (wait_x, wait_y) = if cross.bottom { (0.3, 0.0) } else { (0.0, 0.3) };
            let quad = |wait: f32| {
                let k = ((t - wait) / (0.8 - wait)).clamp(0.0, 1.0);
                k * k
            };
            let at = Vec2::new(
                cross.from.x + quad(wait_x) * (to.x - cross.from.x),
                cross.from.y + quad(wait_y) * (to.y - cross.from.y),
            );
            transform.translation = at.extend(transform.translation.z);
        }
        // A full turn eased in after 0.2 s, its side's way, shrinking to 30% over 0.8 s.
        let spin = ((t - 0.2) / 0.8).clamp(0.0, 1.0);
        transform.rotation = Quat::from_rotation_z(cross.side * std::f32::consts::TAU * spin * spin);
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
    config: Res<GameConfig>,
    images: Option<Res<Assets<Image>>>,
    game: Res<Match>,
    towers: Query<(&Tower, &TowerState)>,
    mut seen: Local<(u64, bool, u32)>,
    old: Query<Entity, With<WaveMessage>>,
) {
    // A new match starts the count again. (The pause menu is the Menu phase too: resetting
    // there made every paused frame start another banner, smeared over the pause screen.)
    if seen.0 != game.round {
        *seen = (game.round, false, 0);
        // A banner left from the last match (it waits while paused) goes.
        for entity in &old {
            commands.entity(entity).despawn();
        }
    }
    let wave = towers
        .iter()
        .any(|(t, s)| t.is_first_player() && s.wave_left > 0.0);
    if wave && !seen.1 {
        seen.2 += 1;
        let number = seen.2;
        // SharedAgonyLayer.WaveActivated: the gong, then the message.
        if let Some(gong) = art.sounds.get("sfx_gong_v2") {
            commands.spawn((AudioPlayer(gong.clone()), PlaybackSettings::DESPAWN));
        }
        // WaveMessage's layout in cells from the screen's middle (y up): two back clouds,
        // WAVE and the number in EnzoBlackRed front and back layers (the back 0.21 lower and
        // 1.11 tall), then three front clouds that drift (`moments.wave_clouds`). TMP world
        // text at 42 is 42 / 32.75 of WINNER!'s size.
        let cell = 100.0 / config.hud.camera_height.max(0.01);
        let place = |x: f32, y: f32, w: f32, h: f32| Node {
            position_type: PositionType::Absolute,
            left: Val::Vh((x - w / 2.0) * cell),
            top: Val::Vh((-y - h / 2.0) * cell),
            width: Val::Vh(w * cell),
            height: Val::Vh(h * cell),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            ..default()
        };
        let size = |name: &str| {
            art.entries.get(name).and_then(|e| {
                let ppu = e.pixels_per_unit.unwrap_or(100.0).max(1.0);
                art.image(name).and_then(|h| images.as_ref().and_then(|i| i.get(&h))).map(|i| i.size_f32() / ppu)
            })
        };
        let text_vh = config.moments.banner_cells * 42.0 / 32.75 * cell;
        let mut root = commands.spawn((
            WaveMessage { time: 0.0 },
            UiTransform::default(),
            Visibility::Hidden,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(50.0),
                ..default()
            },
            GlobalZIndex(10),
        ));
        root.with_children(|row| {
            let cloud = |row: &mut ChildSpawnerCommands, name: &str, x: f32, y: f32, drift: Option<usize>| {
                let (Some(image), Some(s)) = (art.image(name), size(name)) else { return };
                let mut node = row.spawn((ImageNode::new(image), place(x, y, s.x, s.y)));
                if let Some(i) = drift {
                    node.insert((WaveCloud(i), UiTransform::default()));
                }
            };
            cloud(row, "wave_cloud_4", -6.16, 7.5, None);
            cloud(row, "wave_cloud_3", 2.99, 6.21, None);
            for (words, x, y) in [("WAVE".to_string(), -6.16 + 1.93, 7.5 + 1.18), (number.to_string(), 3.0 + 2.09, 6.09 + 1.19)] {
                for back in [true, false] {
                    let tone = crate::results::Tone::Red;
                    let (look, drop, tall) = if back { (tone.back(), 0.21, 1.11) } else { (tone.front(), 0.0, 1.0) };
                    row.spawn(place(x, y - drop, 12.0, 4.0)).with_child((
                        Text::new(words.clone()),
                        // WaveMessage spaces -1.
                        crate::letters::LetterSpacing(-1.0),
                        look,
                        TextFont {
                            font_size: FontSize::Vh(text_vh),
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        TextLayout::justify(Justify::Center),
                        UiTransform::from_scale(Vec2::new(1.0, tall)),
                    ));
                }
            }
            cloud(row, "wave_cloud_2", -10.94, 6.09, Some(0));
            cloud(row, "wave_cloud_2", -10.94 + 8.86, 6.22, Some(1));
            cloud(row, "wave_cloud_1", -1.35 + 8.86, 5.28, Some(2));
        });
    }
    seen.1 = wave;
}

fn fade_wave_messages(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Match>,
    config: Res<GameConfig>,
    mut messages: Query<(Entity, &mut WaveMessage, &mut UiTransform, &mut Visibility, &Children)>,
    mut clouds: Query<(&WaveCloud, &mut UiTransform), Without<WaveMessage>>,
) {
    let moments = &config.moments;
    let length = moments.wave_x.last().map_or(0.0, |k| k.0);
    // A cell is a 41.54th of the view's height.
    let vh_per_cell = 100.0 / config.hud.camera_height.max(0.01);
    for (entity, mut message, mut transform, mut visibility, children) in &mut messages {
        // The banner waits with the match while the pause menu is up.
        if game.phase != Phase::Menu {
            message.time += time.delta_secs();
        }
        if message.time >= length {
            commands.entity(entity).despawn();
            continue;
        }
        let x = crate::config::curve(&moments.wave_x, message.time);
        transform.translation = Val2::new(Val::Vh(x * vh_per_cell), Val::Px(0.0));
        transform.scale = Vec2::new(crate::config::curve(&moments.wave_scale, message.time), 1.0);
        visibility.set_if_neq(Visibility::Inherited);
        for child in children.iter() {
            if let Ok((cloud, mut drift)) = clouds.get_mut(child)
                && let Some(keys) = config.moments.wave_clouds.get(cloud.0)
            {
                let start = keys.first().map_or(0.0, |k| k.1);
                let x = crate::config::curve(keys, message.time) - start;
                drift.translation = Val2::new(Val::Vh(x * vh_per_cell), Val::Px(0.0));
            }
        }
    }
}
