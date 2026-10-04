//! Effects drawn over the play: the Slow balloon on its string, the Ghost skulls
//! that carry a brick off (both only when their art is staged), and the splash
//! of a brick lost in the water.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::gameplay::{ActiveBrick, Brick, BrickLost};
use crate::spells::Ghosted;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                float_balloons,
                attach_skulls,
                fly_skulls,
                splash,
                fly_droplets,
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
            let name = format!("ghost_skull_{}", n % 2 + 1);
            let Some(image) = art.image(&name).or_else(|| art.image("ghost_skull")) else {
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

/// A droplet of a splash, flying and fading.
#[derive(Component)]
struct Droplet {
    velocity: Vec2,
    time: f32,
}

/// Staged splash art, removed when its animation ends.
#[derive(Component)]
struct SplashArt {
    time: f32,
    length: f32,
}

/// A splash where each lost brick hits the water.
fn splash(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    time: Res<Time>,
    mut lost: MessageReader<BrickLost>,
) {
    let c = config.cell_size;
    let look = &config.splash;
    let water = -config.cells(config.kill_depth);
    for (n, brick) in lost.read().enumerate() {
        let at = Vec2::new(brick.at.x, water);
        if let Some(image) = art.image("splash") {
            let sprite = Sprite::from_image(image);
            let ppu = art
                .entries
                .get("splash")
                .and_then(|e| e.pixels_per_unit)
                .unwrap_or(100.0);
            let mut entity = commands.spawn((
                sprite,
                bevy::sprite::Anchor::BOTTOM_CENTER,
                Transform::from_translation(at.extend(6.0))
                    .with_scale(Vec3::splat(c / ppu.max(1.0))),
                SplashArt {
                    time: 0.0,
                    length: art.length("splash").unwrap_or(look.life),
                },
            ));
            if art.is_animation("splash") {
                entity.insert(crate::art::Animated::new("splash"));
            }
            continue;
        }
        // No art: droplets thrown up in a fan, varied by a cheap hash.
        let seed = (time.elapsed_secs() * 1000.0) as u32 ^ (n as u32 * 7919);
        for d in 0..look.drops {
            let k =
                ((seed.wrapping_mul(2654435761).wrapping_add(d * 40503)) % 1000) as f32 / 1000.0;
            let side = d as f32 / (look.drops.max(2) - 1) as f32 * 2.0 - 1.0;
            let angle = (side * look.spread).to_radians();
            let speed = look.speed.0 + (look.speed.1 - look.speed.0) * k;
            commands.spawn((
                Sprite::from_color(
                    Color::srgba(0.85, 0.93, 1.0, 0.9),
                    Vec2::splat(config.cells(look.size)),
                ),
                Transform::from_translation(at.extend(6.0)),
                Droplet {
                    velocity: Vec2::new(angle.sin(), angle.cos()) * config.cells(speed),
                    time: 0.0,
                },
            ));
        }
    }
}

fn fly_droplets(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    mut droplets: Query<(Entity, &mut Droplet, &mut Transform, &mut Sprite)>,
    mut arts: Query<(Entity, &mut SplashArt)>,
) {
    let dt = time.delta_secs();
    let life = config.splash.life.max(0.05);
    for (entity, mut drop, mut transform, mut sprite) in &mut droplets {
        drop.time += dt;
        if drop.time >= life {
            commands.entity(entity).despawn();
            continue;
        }
        drop.velocity.y -= config.cells(config.gravity) * dt;
        transform.translation += (drop.velocity * dt).extend(0.0);
        sprite.color = sprite.color.with_alpha(0.9 * (1.0 - drop.time / life));
    }
    for (entity, mut splash) in &mut arts {
        splash.time += dt;
        if splash.time >= splash.length {
            commands.entity(entity).despawn();
        }
    }
}
