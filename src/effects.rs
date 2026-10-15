//! Effects drawn over the play: the Slow balloon on its string, the Ghost skulls
//! that carry a brick off (both only when their art is staged), the splash
//! of a brick lost in the water, and the blasts, shakes and sounds of a losing
//! tower falling apart.

use bevy::prelude::*;

use crate::art::Art;
use crate::collapse::{CollapseEvent, CollapseKind};
use crate::config::GameConfig;
use crate::gameplay::{ActiveBrick, Brick, BrickLost};
use crate::spells::Ghosted;
use crate::visuals::FramedCamera;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraShake>().add_systems(
            Update,
            (
                float_balloons,
                attach_skulls,
                fly_skulls,
                splash,
                fly_droplets,
                show_collapse,
                fly_particles,
                rumble_base_moves,
                stop_rumble,
                level_up_sounds,
                grow_tufts,
                unshake_camera.before(FramedCamera),
                shake_camera.after(FramedCamera).after(show_collapse),
            ),
        );
    }
}

/// A balloon tied to a slowed brick, staying upright above it.
#[derive(Component)]
struct Balloon {
    brick: Entity,
    string: Entity,
}

/// How far above the brick's centre the balloon floats, in cells.
const BALLOON_LIFT: f32 = 2.5;

#[allow(clippy::type_complexity)]
fn float_balloons(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<(Entity, &ActiveBrick, &Transform), Without<Balloon>>,
    mut balloons: Query<(Entity, &Balloon, &mut Transform), Without<ActiveBrick>>,
    mut strings: Query<(&mut Transform, &mut Sprite), (Without<Balloon>, Without<ActiveBrick>)>,
) {
    let c = config.cell_size;
    for (entity, balloon, mut transform) in &mut balloons {
        let Ok((_, control, brick)) = bricks.get(balloon.brick) else {
            commands.entity(balloon.string).despawn();
            commands.entity(entity).despawn();
            continue;
        };
        if !control.mods.slow {
            commands.entity(balloon.string).despawn();
            commands.entity(entity).despawn();
            continue;
        }
        let top = brick.translation + Vec3::Y * c * BALLOON_LIFT;
        transform.translation = top.with_z(5.0);
        if let Ok((mut line, mut sprite)) = strings.get_mut(balloon.string) {
            // A 3-pixel line from the brick up to the balloon.
            let from = brick.translation.truncate();
            let to = top.truncate() - Vec2::Y * c * 0.5;
            line.translation = ((from + to) * 0.5).extend(4.9);
            sprite.custom_size = Some(Vec2::new(3.0, from.distance(to)));
        }
    }
    let Some(image) = art.image("balloon") else {
        return;
    };
    for (entity, control, _) in &bricks {
        if !control.mods.slow || balloons.iter().any(|(_, b, _)| b.brick == entity) {
            continue;
        }
        let string = commands
            .spawn((
                Sprite::from_color(Color::srgb(0.95, 0.95, 0.9), Vec2::new(3.0, 1.0)),
                Transform::default(),
            ))
            .id();
        let (sprite, anchor, transform) = sized(&art, "balloon", image.clone(), c);
        commands.spawn((
            Balloon {
                brick: entity,
                string,
            },
            sprite,
            anchor,
            transform,
        ));
    }
}

/// The sprite for a staged name, sized by its pixels per unit.
fn sized(
    art: &Art,
    name: &str,
    image: Handle<Image>,
    cell: f32,
) -> (Sprite, bevy::sprite::Anchor, Transform) {
    let entry = art.entries.get(name);
    let (px, py) = entry.and_then(|e| e.pivot).unwrap_or((0.5, 0.5));
    let ppu = entry
        .and_then(|e| e.pixels_per_unit)
        .unwrap_or(100.0)
        .max(1.0);
    (
        Sprite::from_image(image),
        bevy::sprite::Anchor(Vec2::new(px - 0.5, py - 0.5)),
        Transform::from_scale(Vec3::splat(cell / ppu)),
    )
}

/// The picture names for the Ghost skull that takes grip `n`: the named skulls, then
/// the staged skull heads (`fx_skull0<1|2>_head`, the head alone; the jaw isn't drawn).
fn skull_names(n: usize) -> [String; 3] {
    [
        format!("ghost_skull_{}", n % 2 + 1),
        "ghost_skull".into(),
        format!("fx_skull0{}_head", n % 2 + 1),
    ]
}

/// A skull flying in to grab a ghosted brick at one point of its outline.
#[derive(Component)]
struct Skull {
    /// Where on the brick it holds on, in the brick's own space.
    grip: Vec2,
    /// Seconds it has been flying in.
    time: f32,
}

/// Seconds a skull takes to reach its grip.
const SKULL_FLIGHT: f32 = 0.35;

/// Up to 3 skulls fly in to grab points on the outline of each ghosted brick.
fn attach_skulls(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    ghosted: Query<(Entity, &Brick), Added<Ghosted>>,
) {
    let c = config.cell_size;
    for (entity, brick) in &ghosted {
        // Grip the outermost cells: the leftmost, the rightmost and the topmost.
        let pick = |key: fn(Vec2) -> f32| {
            brick
                .cells
                .iter()
                .copied()
                .max_by(|a, b| key(*a).total_cmp(&key(*b)))
                .unwrap_or_default()
        };
        let grips = [
            pick(|v| -v.x) + Vec2::new(-0.5, 0.0),
            pick(|v| v.x) + Vec2::new(0.5, 0.0),
            pick(|v| v.y) + Vec2::new(0.0, 0.5),
        ];
        for (n, grip) in grips.into_iter().enumerate() {
            let names = skull_names(n);
            let Some((name, image)) = names
                .iter()
                .find_map(|name| art.image(name).map(|image| (name.clone(), image)))
            else {
                continue;
            };
            let (sprite, anchor, transform) = sized(&art, &name, image, c);
            let grip = grip * c * brick.scale;
            commands.entity(entity).with_child((
                Skull { grip, time: 0.0 },
                sprite,
                anchor,
                transform.with_translation((grip * 4.0).extend(6.0)),
            ));
        }
    }
}

fn fly_skulls(
    mut commands: Commands,
    time: Res<Time>,
    ghosted: Query<(), With<Ghosted>>,
    mut skulls: Query<(Entity, &ChildOf, &mut Skull, &mut Transform)>,
) {
    for (entity, parent, mut skull, mut transform) in &mut skulls {
        if !ghosted.contains(parent.parent()) {
            commands.entity(entity).despawn();
            continue;
        }
        skull.time += time.delta_secs();
        let t = (skull.time / SKULL_FLIGHT).min(1.0);
        let ease = 1.0 - (1.0f32 - t).powi(3);
        let from = skull.grip * 4.0;
        transform.translation = from.lerp(skull.grip, ease).extend(6.0);
    }
}

/// A piece of a splash: a front picture that grows in and vanishes, or a drop
/// thrown up that falls and shrinks.
#[derive(Component)]
struct SplashPiece {
    velocity: Vec2,
    size: f32,
    time: f32,
    front: bool,
}

/// A tiny repeatable random number in 0..1 from a seed.
fn roll(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 10_000) as f32 / 10_000.0
}

/// A splash where each lost brick hits the water, with one of the splash sounds.
#[allow(clippy::too_many_arguments)]
fn splash(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    time: Res<Time>,
    mut lost: MessageReader<BrickLost>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut drop_look: Local<Option<(Handle<Mesh>, Handle<ColorMaterial>)>>,
) {
    let look = &config.splash;
    // Every drop is the same white disc, scaled.
    let (disc, white) = drop_look
        .get_or_insert_with(|| (meshes.add(Circle::new(0.5)), materials.add(Color::WHITE)))
        .clone();
    for (n, brick) in lost.read().enumerate() {
        let mut seed = (time.elapsed_secs() * 1000.0) as u32 ^ (n as u32 * 7919) | 1;
        let mut r = || roll(&mut seed);
        let at = brick.at - Vec2::Y * config.cells(look.drop_below * r());
        // One of the splash sounds.
        let sounds: Vec<_> = (1..=4)
            .filter_map(|i| art.sounds.get(&format!("sfx_splash_{i}")))
            .collect();
        if !sounds.is_empty() {
            let sound = sounds[(r() * sounds.len() as f32) as usize % sounds.len()].clone();
            commands.spawn((AudioPlayer(sound), PlaybackSettings::DESPAWN));
        }
        // The front pictures along a line, each picked at random.
        let pictures: Vec<_> = (1..=3)
            .filter_map(|i| art.image(&format!("splash_{i}")))
            .chain(art.image("splash"))
            .collect();
        for i in 0..look.fronts {
            let Some(picture) = (!pictures.is_empty())
                .then(|| pictures[(r() * pictures.len() as f32) as usize % pictures.len()].clone())
            else {
                break;
            };
            let along = if look.fronts > 1 {
                i as f32 / (look.fronts - 1) as f32 - 0.5
            } else {
                0.0
            };
            let size = look.front_size.0 + (look.front_size.1 - look.front_size.0) * r();
            commands.spawn((
                Sprite {
                    image: picture,
                    custom_size: Some(Vec2::ZERO),
                    ..default()
                },
                Transform::from_translation(
                    (at + Vec2::X * config.cells(along * look.front_line)).extend(7.0),
                ),
                SplashPiece {
                    velocity: Vec2::ZERO,
                    size: config.cells(size),
                    time: 0.0,
                    front: true,
                },
            ));
        }
        // White drops thrown up behind them.
        for _ in 0..look.drops {
            let angle = (r() * 2.0 - 1.0) * look.drop_cone.to_radians();
            let speed = look.drop_speed.0 + (look.drop_speed.1 - look.drop_speed.0) * r();
            let (dr, da) = (look.drop_disc * r().sqrt(), r() * std::f32::consts::TAU);
            let start = at + Vec2::new(da.cos(), da.sin()) * config.cells(dr);
            let size = look.drop_size.0 + (look.drop_size.1 - look.drop_size.0) * r();
            commands.spawn((
                Mesh2d(disc.clone()),
                MeshMaterial2d(white.clone()),
                Transform::from_translation(start.extend(5.5))
                    .with_scale(Vec3::splat(config.cells(size))),
                SplashPiece {
                    velocity: Vec2::new(angle.sin(), angle.cos()) * config.cells(speed),
                    size: config.cells(size),
                    time: 0.0,
                    front: false,
                },
            ));
        }
    }
}

/// Where a curve of (time, value) points is at `t`, straight between points.
fn along_curve(curve: &[(f32, f32)], t: f32) -> f32 {
    let Some(&(first_t, first)) = curve.first() else {
        return 1.0;
    };
    if t <= first_t {
        return first;
    }
    for pair in curve.windows(2) {
        let ((t0, v0), (t1, v1)) = (pair[0], pair[1]);
        if t <= t1 {
            return v0 + (v1 - v0) * ((t - t0) / (t1 - t0).max(f32::EPSILON));
        }
    }
    curve.last().map_or(1.0, |&(_, v)| v)
}

fn fly_droplets(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut pieces: Query<(
        Entity,
        &mut SplashPiece,
        &mut Transform,
        Option<&mut Sprite>,
    )>,
) {
    let dt = time.delta_secs();
    let look = &config.splash;
    let front_end = look.front_curve.last().map_or(1.0, |&(t, _)| t);
    for (entity, mut piece, mut transform, sprite) in &mut pieces {
        piece.time += dt;
        let share = if piece.front {
            along_curve(&look.front_curve, piece.time)
        } else {
            let (hold, gone) = look.drop_shrink;
            piece.velocity.y -= config.cells(look.drop_gravity) * dt;
            transform.translation += (piece.velocity * dt).extend(0.0);
            1.0 - ((piece.time - hold) / (gone - hold).max(0.01)).clamp(0.0, 1.0)
        };
        let end = if piece.front {
            front_end
        } else {
            look.drop_shrink.1
        };
        if piece.time >= end {
            commands.entity(entity).despawn();
            continue;
        }
        match sprite {
            Some(mut sprite) => sprite.custom_size = Some(Vec2::splat(piece.size * share)),
            None => transform.scale = Vec3::splat(piece.size * share),
        }
    }
}

/// Camera shakes still running, and the offset applied this frame.
#[derive(Resource, Default)]
pub(crate) struct CameraShake {
    /// Reach in pixels (sideways, up) and seconds left.
    pub(crate) kicks: Vec<(Vec2, f32)>,
    /// Random jumps of up to this many pixels each way, every frame, while this brick is
    /// still in hand (Large's BrickCameraShakeEffect stops when the brick touches anything).
    pub(crate) held: Vec<(Entity, f32)>,
    applied: Vec2,
    seed: u32,
}

/// A piece of a blown-up brick.
#[derive(Component)]
struct Particle {
    velocity: Vec2,
    spin: f32,
    gravity: f32,
    /// Speed limit and the share of the excess taken off each frame.
    clamp: Option<(f32, f32)>,
    shrink: ParticleShrink,
    life: f32,
    time: f32,
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            velocity: Vec2::ZERO,
            spin: 0.0,
            gravity: 0.0,
            clamp: None,
            shrink: ParticleShrink::Fade,
            life: 1.0,
            time: 0.0,
        }
    }
}

/// How a particle changes over its life: fading out, or one of the explosion's size curves.
#[derive(Clone, Copy)]
enum ParticleShrink {
    Fade,
    Puff,
    Debris,
}

/// The looping rumble sound, stopped when the rumble ends.
#[derive(Component)]
struct Rumble {
    left: f32,
}

/// How long a shard flies before it is gone, in seconds.

/// A Puzzle base moving up after a lost brick rumbles while it moves (TowerMoveController's
/// looping SFX_RUMBLE).
fn rumble_base_moves(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    towers: Query<(Entity, &crate::gameplay::TowerState)>,
    mut heard: Local<Vec<(Entity, u32)>>,
) {
    for (tower, state) in &towers {
        let was = heard.iter().find(|h| h.0 == tower).map_or(0, |h| h.1);
        if state.base_moves > was
            && let Some(sound) = art.sounds.get("sfx_rumble")
        {
            commands.spawn((
                AudioPlayer(sound.clone()),
                PlaybackSettings::LOOP,
                Rumble { left: config.puzzle.base_rise_time },
            ));
        }
        heard.retain(|h| h.0 != tower);
        heard.push((tower, state.base_moves));
    }
}

/// Blasts, shakes and sounds for the towers falling apart at the end of a match.
fn show_collapse(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    time: Res<Time>,
    mut events: MessageReader<CollapseEvent>,
    mut shake: ResMut<CameraShake>,
) {
    let c = config.cell_size;
    let look = &config.moments;
    let play = |commands: &mut Commands, name: &str, settings: PlaybackSettings| {
        art.sounds
            .get(name)
            .map(|sound| commands.spawn((AudioPlayer(sound.clone()), settings)).id())
    };
    for (n, event) in events.read().enumerate() {
        match &event.kind {
            // Only the controller rumbles here (src/input.rs).
            CollapseKind::Begin => {}
            CollapseKind::Blast { at, .. } => {
                let (x, y, seconds) = look.blast_shake;
                shake.kicks.push((Vec2::new(x, y) * c, seconds));
                play(&mut commands, "sfx_explosion", PlaybackSettings::DESPAWN);
                let mut seed = (time.elapsed_secs() * 1000.0) as u32 ^ (n as u32 * 7919) | 1;
                // The brick hides and the original's EXPLOSION_PARTICLE plays where it was
                // (BrickDisappearEffect): a flash, puffs of smoke and falling debris.
                let blast = &look.explosion;
                let at = at.extend(7.0);
                let spawn = |commands: &mut Commands, name: &str, part: Particle, size: f32, seed: &mut u32| {
                    let Some(image) = art.image(name) else { return };
                    let turn = roll(seed) * std::f32::consts::TAU;
                    commands.spawn((
                        part,
                        Sprite {
                            image,
                            custom_size: Some(Vec2::splat(size)),
                            ..default()
                        },
                        Transform::from_translation(at).with_rotation(Quat::from_rotation_z(turn)),
                    ));
                };
                let flash = Particle { life: blast.flash.1, ..default() };
                spawn(&mut commands, "explosion_flash", flash, blast.flash.0 * c, &mut seed);
                for _ in 0..blast.puffs.0 {
                    let way = Vec2::from_angle(roll(&mut seed) * std::f32::consts::TAU);
                    let part = Particle {
                        velocity: way * roll(&mut seed) * blast.puffs.2 * c,
                        spin: (roll(&mut seed) - 0.5) * 2.0,
                        clamp: Some((blast.puff_clamp.0 * c, blast.puff_clamp.1)),
                        shrink: ParticleShrink::Puff,
                        life: blast.puffs.3,
                        ..default()
                    };
                    let size = roll(&mut seed) * blast.puffs.1 * c;
                    spawn(&mut commands, "explosion_puff", part, size, &mut seed);
                }
                for _ in 0..blast.debris.0 {
                    let way = Vec2::from_angle(roll(&mut seed) * std::f32::consts::TAU);
                    let part = Particle {
                        velocity: way * roll(&mut seed) * blast.debris.2 * c,
                        spin: (roll(&mut seed) - 0.5) * 6.0,
                        gravity: blast.debris_gravity * c,
                        shrink: ParticleShrink::Debris,
                        life: blast.debris.3,
                        ..default()
                    };
                    let size = roll(&mut seed) * blast.debris.1 * c;
                    spawn(&mut commands, "explosion_debris", part, size, &mut seed);
                }
            }
            CollapseKind::Rumble(seconds) => {
                let (x, y, _) = look.rumble;
                shake.kicks.push((Vec2::new(x, y) * c, *seconds));
                if let Some(sound) = play(&mut commands, "sfx_rumble", PlaybackSettings::LOOP) {
                    commands.entity(sound).insert(Rumble { left: *seconds });
                }
            }
            CollapseKind::Thud => {
                play(&mut commands, "sfx_brick_heavy", PlaybackSettings::DESPAWN);
            }
        }
    }
}

fn fly_particles(
    mut commands: Commands,
    time: Res<Time>,
    mut parts: Query<(Entity, &mut Particle, &mut Transform, &mut Sprite)>,
) {
    let dt = time.delta_secs();
    for (entity, mut part, mut transform, mut sprite) in &mut parts {
        part.time += dt;
        let t = part.time / part.life.max(0.001);
        if t >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        part.velocity.y -= part.gravity * dt;
        // Limit Velocity over Lifetime: the speed past the limit is cut by `dampen` each
        // frame (taken at 60 a second here).
        if let Some((limit, dampen)) = part.clamp {
            let speed = part.velocity.length();
            if speed > limit {
                let keep = (1.0 - dampen).powf(dt * 60.0);
                part.velocity *= (limit + (speed - limit) * keep) / speed;
            }
        }
        transform.translation += (part.velocity * dt).extend(0.0);
        transform.rotate_z(part.spin * dt);
        let k = match part.shrink {
            ParticleShrink::Fade => {
                sprite.color.set_alpha(1.0 - t);
                1.0
            }
            // Size over Lifetime: 1, 0.76 at two thirds, 0 at the end.
            ParticleShrink::Puff if t < 0.67 => 1.0 - 0.24 * t / 0.67,
            ParticleShrink::Puff => 0.76 * (1.0 - (t - 0.67) / 0.33),
            // Full size until 89% of its life, then gone by the end.
            ParticleShrink::Debris => ((1.0 - t) / 0.11).min(1.0),
        };
        transform.scale = Vec3::new(k, k, 1.0);
    }
}

fn stop_rumble(mut commands: Commands, time: Res<Time>, mut rumbles: Query<(Entity, &mut Rumble)>) {
    for (entity, mut rumble) in &mut rumbles {
        rumble.left -= time.delta_secs();
        if rumble.left <= 0.0 {
            commands.entity(entity).despawn();
        }
    }
}

/// Takes last frame's shake off the camera before it is framed again.
fn unshake_camera(shake: Res<CameraShake>, mut camera: Query<&mut Transform, With<crate::visuals::MainCamera>>) {
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation -= shake.applied.extend(0.0);
    }
}

/// Puts this frame's shake on the camera: every running kick jitters it.
fn shake_camera(
    time: Res<Time>,
    mut shake: ResMut<CameraShake>,
    in_hand: Query<(), With<crate::gameplay::ActiveBrick>>,
    mut camera: Query<&mut Transform, With<crate::visuals::MainCamera>>,
) {
    let dt = time.delta_secs();
    shake.kicks.retain_mut(|(_, left)| {
        *left -= dt;
        *left > 0.0
    });
    shake.held.retain(|(brick, _)| in_hand.contains(*brick));
    let t = time.elapsed_secs();
    let mut offset = shake
        .kicks
        .iter()
        .map(|(reach, _)| Vec2::new(reach.x * (t * 61.0).sin(), reach.y * (t * 47.0).cos()))
        .sum::<Vec2>();
    let mut seed = shake.seed | 1;
    for (_, reach) in shake.held.clone() {
        offset += Vec2::new(roll(&mut seed) * 2.0 - 1.0, roll(&mut seed) * 2.0 - 1.0) * reach;
    }
    shake.seed = seed;
    shake.applied = Vec2::ZERO;
    if let Ok(mut transform) = camera.single_mut() {
        transform.translation += offset.extend(0.0);
        shake.applied = offset;
    }
}

/// Endless Puzzle level-ups: a laser move sound as the laser rises, a spawn sound
/// with each block (half of them grow a grass tuft), the laser start sound at the end.
fn level_up_sounds(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    towers: Query<(Entity, &crate::gameplay::TowerState)>,
    blocks: Query<Entity, Added<crate::gameplay::Obstacle>>,
    mut seen: Local<Vec<(Entity, bool)>>,
) {
    let play = |commands: &mut Commands, name: &str| {
        if let Some(sound) = art.sounds.get(name) {
            commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
        }
    };
    let roll = || (rand_seed() % 1000) as u32;
    let wait = config.endless_play.level_wait.0;
    for (tower, state) in &towers {
        let rising = state.level_up.is_some_and(|t| t >= wait);
        let was = seen.iter().find(|s| s.0 == tower).is_some_and(|s| s.1);
        if rising && !was {
            play(
                &mut commands,
                &format!("sfx_laser_move_0{}", roll() % 3 + 1),
            );
        }
        if !rising && was && state.level_up.is_none() {
            play(&mut commands, "sfx_spawn_laser");
        }
        seen.retain(|s| s.0 != tower);
        seen.push((tower, rising));
    }
    for block in &blocks {
        play(
            &mut commands,
            &format!("sfx_spawn_block_puzzle_0{}", roll() % 3 + 1),
        );
        if roll() % 2 == 0
            && let Some((mut sprite, anchor, mut transform)) = crate::visuals::art_sprite(
                &art,
                &format!("grass_tuft_{}", roll() % 7 + 1),
                config.cell_size,
            )
        {
            sprite.color = Color::srgb_u8(0x9D, 0xA2, 0x47);
            sprite.flip_x = roll() % 2 == 0;
            transform.translation = Vec3::new(0.0, config.cell_size * 0.5, 0.2);
            transform.scale = Vec3::ZERO;
            commands
                .entity(block)
                .with_child((sprite, anchor, transform, Tuft(0.0)));
        }
    }
}

/// A grass tuft on an Endless Puzzle block: waits 0.5 s, grows to 0.7x in 0.1 s,
/// then springs to full size over 0.4 s.
#[derive(Component)]
struct Tuft(f32);

fn grow_tufts(time: Res<Time>, mut tufts: Query<(&mut Tuft, &mut Transform)>) {
    for (mut tuft, mut transform) in &mut tufts {
        tuft.0 += time.delta_secs();
        let t = tuft.0 - 0.5;
        let s = if t < 0.0 {
            0.0
        } else if t < 0.1 {
            0.7 * t / 0.1
        } else {
            // Ease out elastic from 0.7 to 1.
            let k = ((t - 0.1) / 0.4).min(1.0);
            let e = if k >= 1.0 {
                1.0
            } else {
                1.0 + 2f32.powf(-10.0 * k) * ((k * 10.0 - 0.75) * std::f32::consts::TAU / 3.0).sin()
            };
            0.7 + 0.3 * e
        };
        transform.scale = Vec3::new(s, s, 1.0);
    }
}

/// A cheap seed for picking sound and look variants.
fn rand_seed() -> u64 {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEED: AtomicU64 = AtomicU64::new(0x9E37_79B9_7F4A_7C15);
    let mut s = SEED.load(Ordering::Relaxed);
    let r = crate::gameplay::xorshift(&mut s);
    SEED.store(s, Ordering::Relaxed);
    r
}


#[cfg(test)]
mod tests {
    use super::skull_names;

    #[test]
    fn each_grip_takes_its_skull_then_the_staged_heads() {
        assert_eq!(skull_names(0)[0], "ghost_skull_1");
        assert_eq!(skull_names(1)[2], "fx_skull02_head");
        assert_eq!(skull_names(2)[2], "fx_skull01_head");
    }
}
