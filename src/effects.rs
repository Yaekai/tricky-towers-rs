//! Spell effects drawn over bricks: the Slow balloon on its string and the Ghost
//! skulls that carry a brick off. Drawn only when their art is staged.

use bevy::prelude::*;

use crate::art::Art;
use crate::config::GameConfig;
use crate::gameplay::{ActiveBrick, Brick};
use crate::spells::Ghosted;

pub struct EffectsPlugin;

impl Plugin for EffectsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, (float_balloons, attach_skulls, fly_skulls));
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
