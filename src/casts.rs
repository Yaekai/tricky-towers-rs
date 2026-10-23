//! How casting looks, from the PC's readout of the original: a dark cast sends a
//! beam up from the caster, a beam down onto each victim's brick and a wave over
//! the victim's lane; a light cast throws lightning from the wizard to the brick.
//! Each wizard carries clouds holding its spells, which pop when one is used.
//! Large makes the brick in hand grow with a camera shake. Timings live in
//! [`GameConfig::casts`]; pictures are used when staged (docs/art-layout.md),
//! plain shapes stand in for them otherwise.

use bevy::platform::collections::HashMap;
use bevy::prelude::*;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind, SpellKind};
use crate::effects::CameraShake;
use crate::gameplay::{ActiveBrick, Brick, Match, Phase, Tower, TowerSkin, TowerState};
use crate::spells::SpellLog;
use crate::wizards::Wizard;

pub struct CastsPlugin;

impl Plugin for CastsPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                start_casts,
                run_beams,
                run_skin_beams,
                run_impacts,
                stream_beams,
                run_streams,
                run_orbs,
                run_lightning,
                track_clouds,
                animate_clouds,
                flick_hints,
                grow_large_bricks,
                quiet_large_rumbles,
                lock_bricks,
            )
                .chain(),
        );
    }
}

const DARK_FALLBACK: Color = Color::srgba(0.45, 0.15, 0.65, 0.85);
const LIGHT_FALLBACK: Color = Color::srgba(1.0, 0.95, 0.6, 0.95);

/// Where a tower's wizard floats, in world pixels, and whether it faces left
/// (alone, it floats on the left of the tower). Falls back to the configured spot
/// when the wizard's art isn't staged.
fn wizard_spot(
    config: &GameConfig,
    tower_entity: Entity,
    tower: &Tower,
    state: &TowerState,
    solo: bool,
    wizards: &Query<(&Wizard, &GlobalTransform)>,
) -> (Vec2, bool) {
    if let Some((_, at)) = wizards
        .iter()
        .find(|(w, _)| !w.enemy() && w.tower() == tower_entity)
    {
        return (at.translation().truncate(), solo);
    }
    let look = &config.wizards;
    let x = if solo {
        look.solo_offset_x
    } else {
        look.offset_x
    };
    (
        Vec2::new(
            tower.origin_x + config.cells(x),
            config.cells(state.height + look.lift),
        ),
        solo,
    )
}

/// One stage of a dark cast's beam, or the lane wave.
#[derive(Component)]
struct Beam {
    /// Particles owed to each stream (they come out a fraction at a time).
    owed: Vec<f32>,
    kind: BeamKind,
    /// The beam's foot (world pixels) and the lane centre it ends over.
    foot: Vec2,
    lane_x: f32,
    time: f32,
}

#[derive(Clone, Copy, PartialEq)]
enum BeamKind {
    Up,
    Down,
    Lane,
}

/// The lightning of a light cast, redrawn every frame from wizard to brick.
#[derive(Component)]
struct Lightning {
    from: Vec2,
    to: Vec2,
    time: f32,
    segments: Vec<Entity>,
    /// Pixels a unit where it is drawn, and the render layer (the cup winner's wizard
    /// draws its bolt over the popup).
    cell: f32,
    layers: Option<bevy::camera::visibility::RenderLayers>,
}

/// A lightning bolt from `from` to `to` (pixels), drawn `cell` pixels a unit.
pub(crate) fn spawn_bolt(
    commands: &mut Commands,
    from: Vec2,
    to: Vec2,
    cell: f32,
    layers: Option<bevy::camera::visibility::RenderLayers>,
) {
    commands.spawn((
        Lightning {
            from,
            to,
            time: 0.0,
            segments: Vec::new(),
            cell,
            layers,
        },
        Transform::default(),
        Visibility::default(),
    ));
}

#[allow(clippy::too_many_arguments)]
fn start_casts(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    log: Res<SpellLog>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    skins: Query<&TowerSkin>,
    wizards: Query<(&Wizard, &GlobalTransform)>,
    bricks: Query<&GlobalTransform, With<ActiveBrick>>,
    mut seen: Local<u64>,
) {
    let fresh = (log.1 - (*seen).min(log.1)) as usize;
    *seen = log.1;
    if fresh == 0 || game.phase != Phase::Playing {
        return;
    }
    let solo = towers.iter().count() == 1;
    let find = |index: usize| towers.iter().find(|(_, t, _)| t.index == index);
    let brick_at = |state: &TowerState, tower: &Tower| {
        state
            .active
            .and_then(|e| bricks.get(e).ok())
            .map(|g| g.translation().truncate())
            .unwrap_or(Vec2::new(
                tower.origin_x,
                config.cells(state.stack_top + config.spawn_gap),
            ))
    };
    // A dark cast logs one entry per victim; one beam goes up per caster.
    let mut beamed_up: Vec<usize> = Vec::new();
    let mut lit: Vec<usize> = Vec::new();
    for cast in log.0.iter().rev().take(fresh) {
        let Some((caster_entity, caster, caster_state)) = find(cast.caster) else {
            // A trial's enemy wizard has no tower: only the beam down onto the victim
            // (where the original draws the enemy wizard isn't read yet, ?).
            if cast.caster == crate::spells::ENEMY_WIZARD
                && let Some((_, victim, victim_state)) = find(cast.target)
            {
                let foot = brick_at(victim_state, victim);
                spawn_victim_beams(&mut commands, &config, &art, foot, victim.origin_x, 0);
            }
            continue;
        };
        // Each wizard has its own beam (the original's BEAM_<wizard>).
        let wizard = skins.get(caster_entity).map_or(caster.index, |s| s.wizard);
        let (from, _) = wizard_spot(&config, caster_entity, caster, caster_state, solo, &wizards);
        if cast.spell.is_dark() {
            if !beamed_up.contains(&cast.caster) {
                beamed_up.push(cast.caster);
                let beam = spawn_beam(
                    &mut commands,
                    &config,
                    &art,
                    BeamKind::Up,
                    from,
                    caster.origin_x,
                    wizard,
                );
                commands
                    .entity(beam)
                    .insert((Visibility::Hidden, DelayedBy(config.casts.dark_start)));
            }
            if let Some((_, victim, victim_state)) = find(cast.target) {
                let foot = brick_at(victim_state, victim);
                spawn_victim_beams(&mut commands, &config, &art, foot, victim.origin_x, wizard);
                spawn_curse_orb(&mut commands, &config, &art, foot);
            }
        } else if !lit.contains(&cast.caster) {
            lit.push(cast.caster);
            let to = brick_at(caster_state, caster);
            spawn_bolt(&mut commands, from, to, config.cell_size, None);
        }
    }
}

/// The lane overlay opens over this long, draws at this strength and sits behind
/// the towers (in front of the lane's scenery).
const LANE_OPEN: f32 = 1.0 / 6.0;
const LANE_TINT: f32 = 0.5;
const LANE_GLOW: f32 = 0.392;
const LANE_Z: f32 = -3.0;

/// Seconds before a beam starts.
#[derive(Component)]
struct DelayedBy(f32);

fn beam_image(art: &Art, kind: BeamKind, mode: ModeKind, wizard: usize) -> Option<Handle<Image>> {
    match kind {
        BeamKind::Lane if mode.look() == ModeKind::Survival => art
            .image("cast_lane_beam_survival")
            .or_else(|| art.image("cast_lane_beam")),
        BeamKind::Lane => art.image("cast_lane_beam"),
        _ => art
            .image(&format!("cast_beam_{}", wizard + 1))
            .or_else(|| art.image("cast_beam")),
    }
}

fn spawn_beam(
    commands: &mut Commands,
    config: &GameConfig,
    art: &Art,
    kind: BeamKind,
    foot: Vec2,
    lane_x: f32,
    wizard: usize,
) -> Entity {
    let skin = (kind != BeamKind::Lane && wizard >= 4)
        .then(|| wizard - 4)
        .filter(|&i| i < config.casts.skin_beam.colours.len());
    let image = beam_image(art, kind, config.mode, wizard).filter(|_| skin.is_none());
    let sprite = match image {
        Some(image) => Sprite {
            image,
            custom_size: Some(Vec2::ZERO),
            ..default()
        },
        None if skin.is_some() => Sprite::from_color(Color::NONE, Vec2::ZERO),
        None => Sprite::from_color(DARK_FALLBACK, Vec2::ZERO),
    };
    if let Some(look) = skin {
        let seed = (foot.x.to_bits() ^ foot.y.to_bits().rotate_left(7)) | 1;
        return commands
            .spawn((
                Beam {
                    owed: Vec::new(),
                    kind,
                    foot,
                    lane_x,
                    time: 0.0,
                },
                SkinBeam { look, built: false, spark_in: 0.0, owed: 0.0, seed },
                sprite,
                Transform::from_translation(foot.extend(16.0)),
            ))
            .id();
    }
    let mut beam = commands.spawn((
        Beam {
            owed: Vec::new(),
            kind,
            foot,
            lane_x,
            time: 0.0,
        },
        sprite,
        Transform::from_translation(foot.extend(16.0)),
    ));
    // The spinning blobs at the beam's foot (the glow and rings are only in the impact).
    if kind != BeamKind::Lane {
        let blobs = &config.casts.beam_blobs;
        let parts = blobs
            .get(wizard)
            .or(blobs.first())
            .cloned()
            .unwrap_or_default();
        beam.with_children(|parent| {
            for (i, (name, scale, angle, alpha, spin)) in parts.into_iter().enumerate() {
                let Some((mut sprite, anchor, transform)) =
                    crate::visuals::art_sprite(art, &name, config.cell_size)
                else {
                    continue;
                };
                sprite.color = Color::WHITE.with_alpha(alpha);
                parent.spawn((
                    BeamBlob {
                        size: transform.scale.x * scale,
                        spin,
                    },
                    sprite,
                    anchor,
                    Transform::from_xyz(0.0, 0.0, 0.1 + i as f32 * 0.01)
                        .with_rotation(Quat::from_rotation_z(angle.to_radians()))
                        .with_scale(Vec3::ZERO),
                ));
            }
        });
    }
    beam.id()
}

/// A dark cast's victim: the beam down onto its brick and the lane overlay over its lane.
fn spawn_victim_beams(
    commands: &mut Commands,
    config: &GameConfig,
    art: &Art,
    foot: Vec2,
    lane_x: f32,
    wizard: usize,
) {
    let beam = spawn_beam(commands, config, art, BeamKind::Down, foot, lane_x, wizard);
    commands
        .entity(beam)
        .insert((Visibility::Hidden, DelayedBy(config.casts.dark_fall)));
    // The impact on the victim's brick (BeamDownIn): rays, two rings and a glow.
    let glows = &config.casts.beam_glow;
    let tint = glows
        .get(wizard)
        .or(glows.first())
        .map_or(Color::WHITE, |&(r, g, b, _)| Color::srgb(r, g, b));
    for (part, name) in IMPACT_PARTS.iter().enumerate() {
        if let Some((mut sprite, anchor, transform)) =
            crate::visuals::art_sprite(art, name, config.cell_size)
        {
            sprite.color = tint.with_alpha(0.0);
            commands.spawn((
                Impact {
                    part,
                    size: transform.scale.x,
                    time: -config.casts.dark_fall,
                },
                sprite,
                anchor,
                Transform::from_translation(foot.extend(16.5 + part as f32 * 0.01))
                    .with_scale(Vec3::ZERO),
            ));
        }
    }
    let lane = spawn_beam(commands, config, art, BeamKind::Lane, foot, lane_x, wizard);
    commands
        .entity(lane)
        .insert((Visibility::Hidden, DelayedBy(config.casts.dark_start)));
}

/// A particle given off by a beam: its velocity (pixels a second), life, age, full size and
/// whether it swells and shrinks in place (a droplet) or fades in and out (a streak or wave).
#[derive(Component)]
struct StreamBit {
    /// The beam it came from: its particles go with it (they live in the beam's object).
    beam: Entity,
    velocity: Vec2,
    life: f32,
    age: f32,
    size: Vec2,
    pulse: bool,
}

/// Gives off each beam's particles while it stands (from the moment its column has grown).
#[allow(clippy::too_many_arguments)]
fn stream_beams(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut beams: Query<(Entity, &mut Beam, Option<&DelayedBy>)>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut adds: Option<ResMut<Assets<crate::scene_material::AdditiveSprite>>>,
    mut quad: Local<Option<Handle<Mesh>>>,
) {
    let (Some(meshes), Some(adds)) = (meshes.as_mut(), adds.as_mut()) else {
        return;
    };
    let quad = quad
        .get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0)))
        .clone();
    let look = &config.casts;
    let dt = time.delta_secs();
    let c = config.cell_size;
    for (entity, mut beam, delay) in &mut beams {
        let streams = match beam.kind {
            BeamKind::Up => &look.beam_streams_up,
            BeamKind::Down => &look.beam_streams_down,
            BeamKind::Lane => continue,
        };
        let grow = match beam.kind {
            BeamKind::Up => look.beam_up.0,
            _ => look.beam_down.0,
        };
        if delay.is_some() || beam.time < grow {
            continue;
        }
        if beam.owed.len() != streams.len() {
            beam.owed = vec![0.0; streams.len()];
        }
        let mut seed =
            (time.elapsed_secs().to_bits() ^ entity.index_u32().wrapping_mul(2_654_435_761)) | 1;
        let mut unit = || {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            (seed % 10_000) as f32 / 10_000.0
        };
        for (i, stream) in streams.iter().enumerate() {
            beam.owed[i] += stream.rate * dt;
            let Some(image) = art.image(&stream.picture) else {
                continue;
            };
            while beam.owed[i] >= 1.0 {
                beam.owed[i] -= 1.0;
                let mut at = beam.foot + Vec2::new(stream.at.0, stream.at.1) * c;
                at.x += (unit() - 0.5) * stream.spread * c;
                if stream.ring > 0.0 {
                    at += Vec2::from_angle(unit() * std::f32::consts::TAU) * stream.ring * c;
                }
                let speed = stream.speed.0 + (stream.speed.1 - stream.speed.0) * unit();
                let way = if stream.down { -1.0 } else { 1.0 };
                let (a, b) = stream.tints;
                let k = unit();
                let tint = Color::srgb_u8(a.0, a.1, a.2).mix(&Color::srgb_u8(b.0, b.1, b.2), k);
                let pulse = stream.ring > 0.0;
                let size =
                    Vec2::new(stream.size.0, stream.size.1) * c * if pulse { unit() } else { 1.0 };
                let tint = Vec4::from_array(tint.to_linear().to_f32_array());
                commands.spawn((
                    StreamBit {
                        beam: entity,
                        velocity: Vec2::Y * way * speed * c,
                        life: stream.life,
                        age: 0.0,
                        size,
                        pulse,
                    },
                    Mesh2d(quad.clone()),
                    MeshMaterial2d(adds.add(crate::scene_material::AdditiveSprite {
                        tint: tint.with_w(0.0),
                        picture: image.clone(),
                    })),
                    Transform::from_translation(at.extend(16.2)).with_scale(Vec3::ZERO),
                ));
            }
        }
    }
}

/// Moves the beams' particles: streaks and waves fade in and out over their first and last
/// tenth, droplets swell and shrink (Droplets' size curve: 0.83 at 30%, full at 56%, 0.97 at
/// 92%, gone at the end).
fn run_streams(
    mut commands: Commands,
    time: Res<Time>,
    mut bits: Query<(
        Entity,
        &mut StreamBit,
        &mut Transform,
        &MeshMaterial2d<crate::scene_material::AdditiveSprite>,
    )>,
    beams: Query<(), With<Beam>>,
    mut adds: Option<ResMut<Assets<crate::scene_material::AdditiveSprite>>>,
) {
    let dt = time.delta_secs();
    for (entity, mut bit, mut transform, material) in &mut bits {
        bit.age += dt;
        let t = bit.age / bit.life.max(0.001);
        // A finished beam takes its particles with it (the white waves Max saw falling on
        // after a cast outlived it by up to 10 s).
        if t >= 1.0 || !beams.contains(bit.beam) {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation += (bit.velocity * dt).extend(0.0);
        let (size, alpha) = if bit.pulse {
            // Droplets' SizeModule curve with its slopes.
            let keys = [
                (0.0, 0.0, 0.0, 0.0),
                (0.3006, 0.8291, 1.235, 1.235),
                (0.556, 1.0, 0.0, 0.0),
                (0.9227, 0.9712, -0.7031, -0.7031),
                (1.0, 0.0, 0.0, 0.0),
            ];
            (bit.size * crate::config::curve(&keys, t), 1.0)
        } else {
            (bit.size, (t / 0.1).min(1.0).min((1.0 - t) / 0.1))
        };
        transform.scale = size.extend(1.0);
        if let Some(mut look) = adds.as_mut().and_then(|a| a.get_mut(&material.0))
            && look.tint.w != alpha
        {
            look.tint.w = alpha;
        }
    }
}

/// The pictures of a down beam's impact, back to front.
const IMPACT_PARTS: [&str; 4] = [
    "cast_circle_gradient",
    "cast_circle_thin",
    "cast_circle_thick",
    "cast_sunrays",
];

/// One picture of a down beam's impact and its seconds since the beam started (negative
/// while it waits).
#[derive(Component)]
struct Impact {
    part: usize,
    size: f32,
    time: f32,
}

/// BeamDownIn's keys for the impact, from 0.2 s: the glow and the thin ring grow to 2 by
/// 0.517 s fading out by 0.6 s (from 0.153 and 0.228); the thick ring grows from 0.267 s
/// to 1.2 at 0.633 s fading from 0.516; the rays grow to 1 by 0.633 s, fading from 0.498
/// after 0.4 s. Straight lines between keys (Unity's are smoothed).
fn run_impacts(
    mut commands: Commands,
    time: Res<Time>,
    mut parts: Query<(Entity, &mut Impact, &mut Transform, &mut Sprite)>,
) {
    let ramp = |t: f32, a: f32, b: f32| ((t - a) / (b - a)).clamp(0.0, 1.0);
    for (entity, mut impact, mut transform, mut sprite) in &mut parts {
        impact.time += time.delta_secs();
        let t = impact.time;
        if t >= 0.64 {
            commands.entity(entity).despawn();
            continue;
        }
        let (scale, alpha) = if t < 0.2 {
            (0.0, 0.0)
        } else {
            match impact.part {
                0 => (2.0 * ramp(t, 0.2, 0.517), 0.153 * (1.0 - ramp(t, 0.2, 0.6))),
                1 => (2.0 * ramp(t, 0.2, 0.517), 0.228 * (1.0 - ramp(t, 0.2, 0.6))),
                2 => (
                    1.2 * ramp(t, 0.267, 0.633),
                    0.516 * (1.0 - ramp(t, 0.267, 0.633)),
                ),
                _ => (ramp(t, 0.2, 0.633), 0.498 * (1.0 - ramp(t, 0.4, 0.633))),
            }
        };
        transform.scale = Vec3::new(scale * impact.size, scale * impact.size, 1.0);
        sprite.color.set_alpha(alpha);
    }
}

/// A blob at a beam's foot: its full size and spin (degrees a second). It stays at the
/// foot, not stretched with the beam.
#[derive(Component)]
struct BeamBlob {
    size: f32,
    spin: f32,
}

/// A lane's quad drawn from the grabbed background (post::BeamLane), under its sprite,
/// which then only keeps the size.
#[derive(Component)]
struct LaneQuad;

#[allow(clippy::type_complexity)]
fn run_beams(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    camera: Query<(&Transform, &Projection), (With<crate::visuals::MainCamera>, Without<Beam>)>,
    mut beams: Query<
        (
            Entity,
            &mut Beam,
            &mut Sprite,
            &mut Transform,
            &mut Visibility,
            Option<&mut DelayedBy>,
            Option<&Children>,
        ),
        Without<SkinBeam>,
    >,
    mut feet: Query<
        (&BeamBlob, &mut Transform),
        (Without<Beam>, Without<crate::visuals::MainCamera>),
    >,
    mut quads: Query<
        &mut Transform,
        (
            With<LaneQuad>,
            Without<BeamBlob>,
            Without<Beam>,
            Without<crate::visuals::MainCamera>,
        ),
    >,
    mut lane_looks: Option<ResMut<Assets<crate::post::BeamLane>>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    grab: Option<Res<crate::post::GrabTarget>>,
    mut images: Option<ResMut<Assets<Image>>>,
    mut glow: Local<Option<(Handle<Image>, Handle<Image>)>>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    // The lane wave covers the whole view's height.
    let (view_y, view_h) = camera
        .single()
        .map(|(t, p)| {
            let scale = match p {
                Projection::Orthographic(o) => o.scale,
                _ => 1.0,
            };
            (t.translation.y, 2000.0 * scale)
        })
        .unwrap_or((0.0, 2000.0));
    let width = config.cells(look.beam_width);
    for (entity, mut beam, mut sprite, mut transform, mut visibility, delay, children) in &mut beams
    {
        if let Some(mut delay) = delay {
            delay.0 -= dt;
            if delay.0 > 0.0 {
                continue;
            }
            commands.entity(entity).remove::<DelayedBy>();
            *visibility = Visibility::Inherited;
        }
        beam.time += dt;
        let (grow, last) = match beam.kind {
            BeamKind::Up => look.beam_up,
            BeamKind::Down => look.beam_down,
            BeamKind::Lane => look.lane_wave,
        };
        let t = beam.time;
        if beam.kind == BeamKind::Lane {
            // The overlay opens across the lane in 1/6 s, stays, then closes in
            // `last` (the original's BeamLaneIn / BeamLaneOut clips).
            if t >= grow + last {
                commands.entity(entity).despawn();
                continue;
            }
            let open = if t < grow {
                (t / LANE_OPEN).min(1.0)
            } else {
                1.0 - (t - grow) / last.max(0.01)
            };
            let lane = config.cells(config.lane_half_width * 2.0 + 2.0);
            sprite.custom_size = Some(Vec2::new(lane * open.clamp(0.0, 1.0), view_h));
            transform.translation = Vec2::new(beam.lane_x, view_y).extend(LANE_Z);
            // With the grab: Race's lane multiplies the background behind it by its
            // picture (BeamBackWorld1Mat, 80%), Survival's adds light to it (BeamLane02,
            // 39%), both read through the wobble.
            let survival = config.mode.look() == ModeKind::Survival;
            let quad = children.and_then(|c| c.iter().find(|e| quads.contains(*e)));
            match quad {
                Some(quad) => {
                    if let Ok(mut t) = quads.get_mut(quad) {
                        t.scale = sprite.custom_size.unwrap_or(Vec2::ZERO).extend(1.0);
                    }
                    continue;
                }
                None => {
                    if let (Some(looks), Some(meshes), Some(grab)) =
                        (lane_looks.as_mut(), meshes.as_mut(), grab.as_ref())
                    {
                        let (tint, mode) = if survival {
                            (Vec4::new(LANE_GLOW, LANE_GLOW, LANE_GLOW, 1.0), 1.0)
                        } else {
                            (Vec4::new(1.0, 1.0, 1.0, 0.8), 0.0)
                        };
                        let quad = commands
                            .spawn((
                                LaneQuad,
                                crate::post::NeedsGrab,
                                Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                                MeshMaterial2d(looks.add(crate::post::BeamLane {
                                    tint,
                                    mode: Vec4::new(mode, 0.0, 0.0, 0.0),
                                    picture: sprite.image.clone(),
                                    grab: grab.0.clone(),
                                })),
                                Transform::default(),
                            ))
                            .id();
                        commands.entity(entity).add_child(quad);
                        sprite.color = Color::NONE;
                        continue;
                    }
                }
            }
            // Without the grab, sprites only blend: the light one draws a whitened copy and
            // the other a see-through tint.
            let alpha = match (survival, art.image("cast_lane_beam").is_some()) {
                (true, _) => 1.0,
                (false, true) => LANE_TINT,
                (false, false) => 0.35,
            };
            if survival && let Some(images) = images.as_mut() {
                let source = sprite.image.clone();
                if glow.as_ref().is_none_or(|(from, _)| *from != source)
                    && glow.as_ref().is_none_or(|(_, to)| *to != source)
                    && let Some(copy) = images
                        .get(&source)
                        .and_then(|i| crate::visuals::glow_copy(i, LANE_GLOW))
                {
                    *glow = Some((source.clone(), images.add(copy)));
                }
                if let Some((from, to)) = glow.as_ref()
                    && *from == source
                {
                    sprite.image = to.clone();
                }
            }
            sprite.color.set_alpha(alpha);
            // On the background layer: behind the towers, not over them.
            transform.translation = Vec2::new(beam.lane_x, view_y).extend(LANE_Z);
            continue;
        }
        // Grows out of its foot, holds, then squeezes to nothing.
        let reach = config.cells(if beam.kind == BeamKind::Down {
            look.beam_reach.1
        } else {
            look.beam_reach.0
        });
        let length = reach * (t / grow).clamp(0.0, 1.0);
        let squeeze = ((t - last) / look.beam_squeeze.max(0.001)).clamp(0.0, 1.0);
        if squeeze >= 1.0 {
            commands.entity(entity).despawn();
            continue;
        }
        sprite.custom_size = Some(Vec2::new(width * (1.0 - 0.99 * squeeze), length));
        // Both beams stand on their foot: up from the wizard, down from the sky
        // onto the brick.
        transform.translation = (beam.foot + Vec2::Y * length * 0.5).extend(16.0);
        // The blobs grow in (the down beam's after its column has started) and spin.
        let wait = if beam.kind == BeamKind::Down {
            look.beam_blob_grow
        } else {
            0.0
        };
        let grown = ((t - wait) / look.beam_blob_grow.max(0.001)).clamp(0.0, 1.0);
        if let Some(children) = children {
            for child in children.iter() {
                if let Ok((blob, mut foot)) = feet.get_mut(child) {
                    foot.translation.y = -length * 0.5;
                    foot.rotate_z(blob.spin.to_radians() * dt);
                    let k = blob.size * grown * (1.0 - 0.99 * squeeze);
                    foot.scale = Vec3::new(k, k, 1.0);
                }
            }
        }
    }
}

/// An Animal or Indie wizard's beam (BeamAnimal0N / BeamIndie0N and their Down twins): two
/// scrolling columns, three spinning stars, sparks and star particles at its foot, and a
/// fading glow under a Down beam's brick. Index into `skin_beam.colours`.
#[derive(Component)]
struct SkinBeam {
    look: usize,
    built: bool,
    /// Seconds to the next spark; star particles owed.
    spark_in: f32,
    owed: f32,
    seed: u32,
}

/// The parts of a skin beam, children of its `Beam`.
#[derive(Component, Clone, Copy, PartialEq)]
enum SkinPart {
    /// 0 the back column, 1 the centre one.
    Column(usize),
    /// A star (front to back) and its base scale.
    Star(usize, f32),
    Circle(f32),
}

/// A spark: seconds since it started and the height it stretches to.
#[derive(Component)]
struct SkinSpark {
    time: f32,
    height: f32,
    base: f32,
}

/// A star particle: velocity (pixels/s), seconds lived, spin (radians/s).
#[derive(Component)]
struct SkinParticle {
    velocity: Vec2,
    time: f32,
    spin: f32,
}

fn skin_roll(seed: &mut u32) -> f32 {
    *seed ^= *seed << 13;
    *seed ^= *seed >> 17;
    *seed ^= *seed << 5;
    (*seed % 10_000) as f32 / 10_000.0
}

/// Piecewise-linear value of `keys` (time, value) at `t`.
fn keyed_lerp(keys: &[(f32, f32)], t: f32) -> f32 {
    let mut last = keys[0];
    if t <= last.0 {
        return last.1;
    }
    for &key in &keys[1..] {
        if t <= key.0 {
            let k = (t - last.0) / (key.0 - last.0).max(1e-4);
            return last.1 + (key.1 - last.1) * k;
        }
        last = key;
    }
    last.1
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn run_skin_beams(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut beams: Query<(Entity, &mut Beam, &mut SkinBeam, &mut Visibility, Option<&mut DelayedBy>)>,
    mut parts: Query<
        (
            &SkinPart,
            &ChildOf,
            &mut Transform,
            Option<&mut Sprite>,
            Option<&MeshMaterial2d<crate::scene_material::ScrollBeam>>,
        ),
        Without<Beam>,
    >,
    mut sparks: Query<(Entity, &mut SkinSpark, &mut Transform), (Without<Beam>, Without<SkinPart>)>,
    mut stars: Query<
        (Entity, &mut SkinParticle, &mut Transform),
        (Without<Beam>, Without<SkinPart>, Without<SkinSpark>),
    >,
    looks: Option<ResMut<Assets<crate::scene_material::ScrollBeam>>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
) {
    let look = &config.casts.skin_beam;
    let dt = time.delta_secs();
    let c = config.cell_size;
    let mut looks = looks;
    for (entity, mut beam, mut skin, mut visibility, delay) in &mut beams {
        if let Some(mut delay) = delay {
            delay.0 -= dt;
            if delay.0 > 0.0 {
                continue;
            }
            commands.entity(entity).remove::<DelayedBy>();
            *visibility = Visibility::Inherited;
        }
        let colours = &look.colours[skin.look];
        let down = beam.kind == BeamKind::Down;
        if !skin.built {
            skin.built = true;
            let tint = |rgb: [f32; 3]| Vec4::new(rgb[0], rgb[1], rgb[2], 1.0);
            commands.entity(entity).with_children(|beam| {
                for (i, (name, rgb)) in [
                    ("fx_cast_effect_animals_background", colours.back),
                    ("fx_cast_effect_animals_center_background", colours.centre),
                ]
                .into_iter()
                .enumerate()
                {
                    if let (Some(image), Some(looks), Some(meshes)) =
                        (art.image(name), looks.as_mut(), meshes.as_mut())
                    {
                        beam.spawn((
                            SkinPart::Column(i),
                            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                            MeshMaterial2d(looks.add(crate::scene_material::ScrollBeam {
                                tint: tint(rgb),
                                scroll: Vec4::new(look.length * look.repeat_per_cell, 0.0, 0.0, 0.0),
                                picture: image,
                            })),
                            Transform::from_xyz(0.0, 0.0, 0.01 * i as f32).with_scale(Vec3::ZERO),
                        ));
                    }
                }
                // Draw order (sorting order 5, 3, 0): front star over the sparks, the middle one
                // over the centre column, the back one with the back column.
                for (i, z) in [0.05, 0.03, 0.005].into_iter().enumerate() {
                    if let Some((mut sprite, anchor, transform)) =
                        crate::visuals::art_sprite(&art, "fx_cast_effect_animals01", c)
                    {
                        let rgb = colours.stars[i];
                        sprite.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
                        beam.spawn((
                            SkinPart::Star(i, transform.scale.x),
                            sprite,
                            anchor,
                            Transform::from_xyz(0.0, 0.0, z).with_scale(Vec3::ZERO),
                        ));
                    }
                }
                if down
                    && let Some((mut sprite, anchor, transform)) =
                        crate::visuals::art_sprite(&art, "cast_circle_gradient", c)
                {
                    let rgb = colours.stars[0];
                    sprite.color = Color::srgba(rgb[0], rgb[1], rgb[2], 0.0);
                    beam.spawn((
                        SkinPart::Circle(transform.scale.x),
                        sprite,
                        anchor,
                        Transform::from_xyz(0.0, 0.0, -0.001).with_scale(Vec3::ZERO),
                    ));
                }
            });
        }
        beam.time += dt;
        let t = beam.time;
        let last = if down { config.casts.beam_down.1 } else { config.casts.beam_up.1 };
        // The original destroys the beam outright when its cast is done.
        if t >= last {
            commands.entity(entity).despawn();
            continue;
        }
        // BeamAnimalIn / BeamAnimalDownIn: the beam's width and length scale, the stars'
        // scale, and when the sparks and particles start.
        let (sx, sy, star, sparks_at, particles_at) = if down {
            (
                keyed_lerp(&[(0.0, 0.1), (1.0 / 6.0, 0.1), (1.0 / 3.0, 0.5)], t),
                keyed_lerp(&[(0.0, 0.0), (1.0 / 6.0, 0.5)], t),
                keyed_lerp(&[(1.0 / 6.0, 0.0), (1.0 / 3.0, 1.0)], t),
                0.2333,
                1.0 / 3.0,
            )
        } else {
            (
                keyed_lerp(&[(0.0, 0.0), (1.0 / 6.0, 0.1), (1.0 / 3.0, 0.5)], t),
                keyed_lerp(&[(0.0, 0.0), (1.0 / 6.0, 0.5)], t),
                keyed_lerp(&[(0.0, 0.0), (1.0 / 6.0, 1.0)], t),
                1.0 / 6.0,
                1.0 / 6.0,
            )
        };
        let height = look.length * sy / 0.5;
        let centre = if down { look.length - height * 0.5 } else { height * 0.5 };
        for (part, parent, mut transform, sprite, material) in &mut parts {
            if parent.parent() != entity {
                continue;
            }
            match *part {
                SkinPart::Column(i) => {
                    let width = if i == 0 { look.widths.0 } else { look.widths.1 } * sx / 0.5;
                    transform.translation.y = c * centre;
                    transform.scale = Vec3::new(c * width, c * height, 1.0);
                    // BeamBackgroundBehaviour: the picture scrolls up at its speed.
                    let speed = if i == 0 { look.scroll.0 } else { look.scroll.1 };
                    if let (Some(material), Some(looks)) = (material, looks.as_mut())
                        && let Some(mut beam_look) = looks.get_mut(&material.0)
                    {
                        beam_look.scroll.y = -speed * t;
                    }
                }
                SkinPart::Star(i, base) => {
                    let (scale, spin) = look.stars[i];
                    transform.scale = Vec3::splat(base * scale * star);
                    transform.rotate_z(spin.to_radians() * dt);
                }
                SkinPart::Circle(base) => {
                    let (times, alphas, scales) = look.circle;
                    let keys = |v: (f32, f32, f32)| [(times.0, v.0), (times.1, v.1), (times.2, v.2)];
                    let on = t >= times.0;
                    transform.scale = Vec3::splat(if on { base * keyed_lerp(&keys(scales), t) } else { 0.0 });
                    if let Some(mut sprite) = sprite {
                        sprite.color.set_alpha(if on { keyed_lerp(&keys(alphas), t) } else { 0.0 });
                    }
                }
            }
        }
        let mut seed = skin.seed;
        // Sparks: one every 0 to 0.07 s, turned any way, out and back in 0.1 s each.
        if t >= sparks_at {
            skin.spark_in -= dt;
            while skin.spark_in <= 0.0 {
                let (lo, hi) = look.spark_every;
                skin.spark_in += lo + (hi - lo) * skin_roll(&mut seed) + 0.001;
                if let Some((mut sprite, anchor, transform)) =
                    crate::visuals::art_sprite(&art, "cast_spark", c)
                {
                    let rgb = colours.spark;
                    sprite.color = Color::srgb(rgb[0], rgb[1], rgb[2]);
                    let (lo, hi) = look.spark_height;
                    let spark = commands
                        .spawn((
                            SkinSpark {
                                time: 0.0,
                                height: lo + (hi - lo) * skin_roll(&mut seed),
                                base: transform.scale.x,
                            },
                            sprite,
                            anchor,
                            Transform::from_xyz(0.0, 0.0, 0.04)
                                .with_rotation(Quat::from_rotation_z(skin_roll(&mut seed) * std::f32::consts::TAU))
                                .with_scale(Vec3::ZERO),
                        ))
                        .id();
                    commands.entity(entity).add_child(spark);
                }
            }
        }
        // Star particles from a 1-cell circle, thrown out and slowed to a drift.
        if t >= particles_at {
            let (rate, speed, _, _, _, size) = look.particles;
            skin.owed += rate * dt;
            while skin.owed >= 1.0 {
                skin.owed -= 1.0;
                let angle = skin_roll(&mut seed) * std::f32::consts::TAU;
                let r = skin_roll(&mut seed).sqrt();
                let way = Vec2::from_angle(angle);
                let cream = Color::srgb(1.0, 0.988, 0.875);
                let tint = Color::WHITE.mix(&cream, skin_roll(&mut seed));
                let sprite = match art.image("spark_star") {
                    Some(image) => Sprite { image, color: tint, custom_size: Some(Vec2::splat(size * c)), ..default() },
                    None => Sprite::from_color(tint, Vec2::splat(size * c)),
                };
                let star = commands
                    .spawn((
                        SkinParticle {
                            velocity: way * speed * c,
                            time: 0.0,
                            spin: skin_roll(&mut seed) * std::f32::consts::TAU,
                        },
                        sprite,
                        Transform::from_translation((way * r * c).extend(0.06)),
                    ))
                    .id();
                commands.entity(entity).add_child(star);
            }
        }
        skin.seed = seed;
    }
    let (spark_scale_x, spark_time) = (look.spark_width, look.spark_time);
    for (spark, mut state, mut transform) in &mut sparks {
        state.time += dt;
        let k = state.time / spark_time.max(1e-4);
        if k >= 2.0 {
            commands.entity(spark).despawn();
            continue;
        }
        // Out-sine up, in-sine back.
        let y = if k < 1.0 {
            (k * std::f32::consts::FRAC_PI_2).sin()
        } else {
            1.0 - (1.0 - ((k - 1.0) * std::f32::consts::FRAC_PI_2).cos())
        };
        transform.scale = Vec3::new(state.base * spark_scale_x, state.base * state.height * y, 1.0);
    }
    let (_, _, limit, dampen, life, _) = look.particles;
    for (star, mut state, mut transform) in &mut stars {
        state.time += dt;
        if state.time >= life {
            commands.entity(star).despawn();
            continue;
        }
        // Limit Velocity: the speed over `limit` loses `dampen` of the excess each 60 Hz frame.
        let speed = state.velocity.length();
        let cap = limit * c;
        if speed > cap {
            let keep = (1.0 - dampen).powf(dt * 60.0);
            state.velocity *= (cap + (speed - cap) * keep) / speed;
        }
        transform.translation += (state.velocity * dt).extend(0.0);
        transform.rotate_z(state.spin * dt);
    }
}

/// Dark cast's curse orb: the spiky blob with the curse icon. It comes down inside the
/// victim's lane column from the top of the view onto the falling brick (the video).
#[derive(Component)]
struct CurseOrb {
    to: Vec2,
    time: f32,
}

fn spawn_curse_orb(commands: &mut Commands, config: &GameConfig, art: &Art, to: Vec2) {
    let size = config.cells(config.casts.dark_orb_size);
    let blob = match art.image("fx_cast_effect2_blob01") {
        Some(image) => Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        None => Sprite::from_color(DARK_FALLBACK, Vec2::splat(size)),
    };
    let orb = commands
        .spawn((
            CurseOrb { to, time: 0.0 },
            blob,
            Transform::from_translation(to.extend(17.0)),
            Visibility::Hidden,
        ))
        .id();
    if let Some(icon) = art.image("spell_orb_dark") {
        commands.entity(orb).with_children(|parent| {
            parent.spawn((
                Sprite {
                    image: icon,
                    custom_size: Some(Vec2::splat(size * 0.6)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.1),
            ));
        });
    }
}

fn run_orbs(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    view: Option<Res<crate::visuals::ViewSize>>,
    camera: Query<(&Transform, &Projection), (With<crate::visuals::MainCamera>, Without<CurseOrb>)>,
    mut orbs: Query<(Entity, &mut CurseOrb, &mut Transform, &mut Visibility)>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    let size = config.cells(look.dark_orb_size);
    let top = camera.single().map_or(0.0, |(eye, projection)| {
        let scale = match projection {
            Projection::Orthographic(ortho) => ortho.scale,
            _ => 1.0,
        };
        eye.translation.y + view.as_ref().map_or(0.0, |v| v.0.y) * 0.5 * scale + size * 0.5
    });
    for (entity, mut orb, mut transform, mut visibility) in &mut orbs {
        orb.time += dt;
        if orb.time < look.dark_start {
            continue;
        }
        if orb.time >= look.dark_orb_land {
            commands.entity(entity).despawn();
            continue;
        }
        *visibility = Visibility::Inherited;
        let fall = (orb.time - look.dark_fall) / (look.dark_orb_land - look.dark_fall).max(0.01);
        let from = Vec2::new(orb.to.x, top.max(orb.to.y));
        transform.translation = from.lerp(orb.to, fall.clamp(0.0, 1.0)).extend(17.0);
    }
}

/// Number of pieces a lightning bolt is drawn with.
const BOLT_PIECES: usize = 14;

fn run_lightning(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut bolts: Query<(Entity, &mut Lightning)>,
    mut pieces: Query<(&mut Transform, &mut Sprite), Without<Lightning>>,
) {
    let look = &config.casts;
    let image = art.image("cast_lightning");
    for (entity, mut bolt) in &mut bolts {
        bolt.time += time.delta_secs();
        if bolt.time >= look.lightning {
            for &piece in &bolt.segments {
                commands.entity(piece).despawn();
            }
            commands.entity(entity).despawn();
            continue;
        }
        if bolt.segments.is_empty() {
            bolt.segments = (0..BOLT_PIECES)
                .map(|_| {
                    let sprite = match &image {
                        Some(image) => Sprite {
                            image: image.clone(),
                            custom_size: Some(Vec2::ZERO),
                            ..default()
                        },
                        None => Sprite::from_color(LIGHT_FALLBACK, Vec2::ZERO),
                    };
                    let mut piece = commands.spawn((sprite, Transform::default()));
                    if let Some(layers) = &bolt.layers {
                        piece.insert(layers.clone());
                    }
                    piece.id()
                })
                .collect();
            continue;
        }
        // The original's wobble: sin(phase + 5s) × amplitude × sin(πs) along the bolt, the
        // phase running 0 → -5 over the cast and the amplitude 0 → peak at 1/6 of it → 0.
        let (from, to) = (bolt.from, bolt.to);
        let side = (to - from).perp().normalize_or(Vec2::X);
        let t = (bolt.time / look.lightning).clamp(0.0, 1.0);
        let phase = -5.0 * t;
        let rise = 1.0 / 2.4;
        let envelope = if t < rise {
            t / rise
        } else {
            (1.0 - t) / (1.0 - rise)
        };
        let wobble = bolt.cell * look.lightning_wobble * envelope;
        let point = |i: usize| {
            let s = i as f32 / BOLT_PIECES as f32;
            let swing = (s * std::f32::consts::PI).sin() * wobble * (s * 5.0 + phase).sin();
            from.lerp(to, s) + side * swing
        };
        let fade = 1.0 - bolt.time / look.lightning;
        for (i, &piece) in bolt.segments.iter().enumerate() {
            let Ok((mut transform, mut sprite)) = pieces.get_mut(piece) else {
                continue;
            };
            let (a, b) = (point(i), point(i + 1));
            let along = b - a;
            sprite.custom_size = Some(Vec2::new(along.length() + 2.0, bolt.cell * 0.35));
            sprite.color.set_alpha(fade);
            *transform = Transform::from_translation(((a + b) * 0.5).extend(16.5))
                .with_rotation(Quat::from_rotation_z(along.to_angle()));
        }
    }
}

/// A spell cloud beside a wizard, holding one spell.
#[derive(Component)]
struct SpellCloud {
    tower: Entity,
    dark: bool,
    stage: CloudStage,
    time: f32,
    /// Seconds since it appeared, for the pick-up blink.
    age: f32,
}

/// WizardSpellView's blink when a spell is picked up: 0.15 s in, the icon's opacity follows
/// cos(t x 2 pi x 5 / 1.5) / 2 + 1/2 for 1.5 s (five blinks), then stays full.
const PICKUP_BLINK_WAIT: f32 = 0.15;
const PICKUP_BLINK_TIME: f32 = 1.5;
const PICKUP_BLINKS: f32 = 5.0;

fn pickup_alpha(age: f32) -> f32 {
    let t = age - PICKUP_BLINK_WAIT;
    if t <= 0.0 || t > PICKUP_BLINK_TIME {
        1.0
    } else {
        (t * std::f32::consts::TAU * PICKUP_BLINKS / PICKUP_BLINK_TIME).cos() / 2.0 + 0.5
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
enum CloudStage {
    Show,
    Float,
    Used,
    Hide,
}

fn icon_spell(state: &TowerState, dark: bool) -> Option<SpellKind> {
    if dark {
        state
            .dark_spell
            .as_ref()
            .and_then(|c| c.spells.first().copied())
    } else {
        state.light_spell
    }
}

/// Starts a cloud when a tower gets a spell, and sends it off (Used after a cast,
/// Hide otherwise) when the spell goes.
#[allow(clippy::too_many_arguments)]
fn track_clouds(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    game: Res<Match>,
    log: Res<SpellLog>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    mut clouds: Query<(Entity, &mut SpellCloud)>,
    mut held: Local<HashMap<(Entity, bool), SpellKind>>,
    mut seen_casts: Local<u64>,
) {
    let fresh = (log.1 - (*seen_casts).min(log.1)) as usize;
    *seen_casts = log.1;
    if game.phase == Phase::Menu {
        for (entity, _) in &clouds {
            commands.entity(entity).despawn();
        }
        held.clear();
        return;
    }
    let cast_by = |index: usize, dark: bool| {
        log.0
            .iter()
            .rev()
            .take(fresh)
            .any(|c| c.caster == index && c.spell.is_dark() == dark)
    };
    held.retain(|(tower, _), _| towers.contains(*tower));
    for (tower_entity, tower, state) in &towers {
        for dark in [false, true] {
            let now = icon_spell(state, dark);
            let before = held.get(&(tower_entity, dark)).copied();
            if now == before {
                continue;
            }
            if before.is_some() {
                let used = cast_by(tower.index, dark);
                for (_, mut cloud) in &mut clouds {
                    if cloud.tower == tower_entity
                        && cloud.dark == dark
                        && matches!(cloud.stage, CloudStage::Show | CloudStage::Float)
                    {
                        cloud.stage = if used {
                            CloudStage::Used
                        } else {
                            CloudStage::Hide
                        };
                        cloud.time = 0.0;
                    }
                }
            }
            match now {
                Some(spell) => {
                    held.insert((tower_entity, dark), spell);
                    spawn_cloud(&mut commands, &config, &art, tower_entity, dark, spell);
                }
                None => {
                    held.remove(&(tower_entity, dark));
                }
            }
        }
    }
}

fn spawn_cloud(
    commands: &mut Commands,
    config: &GameConfig,
    art: &Art,
    tower: Entity,
    dark: bool,
    spell: SpellKind,
) {
    let look = &config.casts;
    let size = config.cells(look.cloud_size);
    let cloud = match art.image("spell_cloud") {
        Some(image) => Sprite {
            image,
            custom_size: Some(Vec2::splat(size)),
            ..default()
        },
        None => Sprite::from_color(Color::srgba(0.05, 0.03, 0.12, 0.7), Vec2::splat(size * 0.8)),
    };
    // The icon prefab sits in the cloud's SpellContainer at its own size (52 px a unit) and
    // by its pivot; icons aren't square (Grassy's is wide, its pivot high).
    let icon =
        match crate::visuals::art_sprite(art, &format!("spell_icon_{spell:?}"), config.cell_size) {
            Some((sprite, anchor, transform)) => {
                (sprite, anchor, transform.with_translation(Vec3::Z * 0.1))
            }
            None => (
                Sprite::from_color(
                    if dark { DARK_FALLBACK } else { LIGHT_FALLBACK },
                    Vec2::splat(size * look.cloud_icon * 0.7),
                ),
                bevy::sprite::Anchor::CENTER,
                Transform::from_xyz(0.0, 0.0, 0.1),
            ),
        };
    let side = if dark { "dark" } else { "light" };
    let hint = crate::visuals::art_sprite(
        art,
        &format!("spell_cloud_button_{side}_up"),
        config.cell_size,
    );
    commands
        .spawn((
            SpellCloud {
                tower,
                dark,
                stage: CloudStage::Show,
                time: 0.0,
                age: 0.0,
            },
            Transform::from_scale(Vec3::ZERO),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((cloud, Transform::default()));
            parent.spawn((icon, CloudIcon));
            // The button to press, 1.1 cells out to its side and 1.25 below the
            // bubble's centre, flicking up and down (SpellButtonIndicator*PC).
            if let Some((sprite, anchor, mut transform)) = hint {
                let x = if dark { 1.1 } else { -1.1 };
                transform.translation =
                    Vec3::new(x * config.cell_size, -1.25 * config.cell_size, 0.2);
                parent.spawn((sprite, anchor, transform, ButtonHint { dark, tower }));
            }
        });
}

/// A spell bubble's icon (it blinks while the spell is used).
#[derive(Component)]
struct CloudIcon;

/// A spell bubble's button hint, and whose bubble it is.
#[derive(Component)]
struct ButtonHint {
    dark: bool,
    tower: Entity,
}

/// Up for 0.2 s, down for 0.2 s (ButtonPressLight/DarkPC and XBox). Players on a
/// pad (players 2-4) see its X and B; the keyboard player and the AI wizards J and K.
fn flick_hints(
    time: Res<Time>,
    art: Res<Art>,
    towers: Query<&Tower>,
    pads: Query<&Gamepad>,
    mut hints: Query<(&ButtonHint, &mut Sprite)>,
) {
    let playstation = pads.iter().next().is_some_and(crate::input::is_playstation);
    let down = time.elapsed_secs() % 0.4 >= 0.2;
    for (hint, mut sprite) in &mut hints {
        let side = if hint.dark { "dark" } else { "light" };
        let state = if down { "down" } else { "up" };
        let pad = towers
            .get(hint.tower)
            .is_ok_and(|t| t.player.is_some_and(|p| p >= 1));
        let kind = match (pad, playstation) {
            (true, true) => "ps",
            (true, false) => "pad",
            _ => "button",
        };
        if let Some(image) = art
            .image(&format!("spell_cloud_{kind}_{side}_{state}"))
            .or_else(|| art.image(&format!("spell_cloud_button_{side}_{state}")))
            && sprite.image != image
        {
            sprite.image = image;
        }
    }
}

/// Places each cloud beside its wizard and plays Show, Floating, Used and Hide.
fn animate_clouds(
    mut commands: Commands,
    time: Res<Time>,
    config: Res<GameConfig>,
    towers: Query<(Entity, &Tower, &TowerState)>,
    wizards: Query<(&Wizard, &GlobalTransform)>,
    mut clouds: Query<(Entity, &mut SpellCloud, &mut Transform, Option<&Children>)>,
    mut parts: Query<&mut Visibility, Or<(With<CloudIcon>, With<ButtonHint>)>>,
    hints: Query<(), With<ButtonHint>>,
    mut icons: Query<&mut Sprite, With<CloudIcon>>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    let solo = towers.iter().count() == 1;
    for (entity, mut cloud, mut transform, children) in &mut clouds {
        let Ok((tower_entity, tower, state)) = towers.get(cloud.tower) else {
            commands.entity(entity).despawn();
            continue;
        };
        cloud.time += dt;
        cloud.age += dt;
        let (at, flip) = wizard_spot(&config, tower_entity, tower, state, solo, &wizards);
        let spots = look
            .cloud_spots
            .get(tower.index)
            .or(look.cloud_spots.first())
            .copied()
            .unwrap_or_default();
        let spot = Vec2::from(if cloud.dark { spots.1 } else { spots.0 });
        let spot = Vec2::new(if flip { -spot.x } else { spot.x }, spot.y);
        let t = cloud.time;
        let mut drift = Vec2::ZERO;
        use crate::config::curve;
        let pair = |x: &[(f32, f32, f32, f32)], y: &[(f32, f32, f32, f32)]| {
            Vec2::new(curve(x, t), curve(y, t))
        };
        let scale = match cloud.stage {
            CloudStage::Show => {
                if t >= look.cloud_show {
                    cloud.stage = CloudStage::Float;
                    cloud.time = 0.0;
                }
                // Hidden until the clip turns it on.
                if t < look.cloud_show_x.first().map_or(0.0, |k| k.0) {
                    Vec2::ZERO
                } else {
                    pair(&look.cloud_show_x, &look.cloud_show_y)
                }
            }
            CloudStage::Float => {
                let at = t % look.cloud_float.max(0.01);
                drift = Vec2::new(
                    curve(&look.cloud_float_x, at),
                    curve(&look.cloud_float_y, at),
                );
                Vec2::splat(curve(&look.cloud_float_scale, at))
            }
            CloudStage::Used => pair(&look.cloud_used_x, &look.cloud_used_y),
            CloudStage::Hide => pair(&look.cloud_hide_x, &look.cloud_hide_y),
        };
        let done = match cloud.stage {
            CloudStage::Used => t >= look.cloud_used,
            CloudStage::Hide => t >= look.cloud_hide,
            _ => false,
        };
        if done {
            commands.entity(entity).despawn();
            continue;
        }
        transform.translation = (at + config.cells(1.0) * (spot + drift)).extend(21.0);
        transform.scale = scale.max(Vec2::ZERO).extend(1.0);
        // While used, the spell icon blinks and the button hint hides.
        let used = cloud.stage == CloudStage::Used;
        let blink_off = used && look.cloud_used_blinks.iter().any(|&(a, b)| t >= a && t < b);
        let alpha = pickup_alpha(cloud.age);
        for child in children.into_iter().flatten() {
            if let Ok(mut sprite) = icons.get_mut(*child)
                && sprite.color.alpha() != alpha
            {
                sprite.color.set_alpha(alpha);
            }
            if let Ok(mut visibility) = parts.get_mut(*child) {
                let hidden = if hints.contains(*child) {
                    used
                } else {
                    blink_off
                };
                visibility.set_if_neq(if hidden {
                    Visibility::Hidden
                } else {
                    Visibility::Inherited
                });
            }
        }
    }
}

/// A Large brick's rumble, looping while that brick is in hand.
#[derive(Component)]
struct LargeRumble(Entity);

/// Stops a Large brick's rumble once the brick has landed (or gone).
fn quiet_large_rumbles(
    mut commands: Commands,
    rumbles: Query<(Entity, &LargeRumble)>,
    held: Query<(), With<ActiveBrick>>,
) {
    for (sound, rumble) in &rumbles {
        if !held.contains(rumble.0) {
            commands.entity(sound).despawn();
        }
    }
}

/// Grows the brick in hand when Large makes it bigger: its cells are drawn at
/// the new size at once, so they are scaled down to the old size and eased up.
/// Also shakes it while No Rotate refuses a turn.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn grow_large_bricks(
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut commands: Commands,
    mut shake: ResMut<CameraShake>,
    bricks: Query<(Entity, &Brick, &ActiveBrick, Option<&Children>)>,
    mut cells: Query<(&mut Transform, Option<&GrowBase>), Without<Brick>>,
    mut sizes: Local<HashMap<Entity, (f32, f32, f32)>>,
    mut shook: Local<bevy::platform::collections::HashSet<Entity>>,
) {
    let look = &config.casts;
    let dt = time.delta_secs();
    sizes.retain(|entity, _| bricks.contains(*entity));
    shook.retain(|entity| bricks.contains(*entity));
    for (entity, brick, control, children) in &bricks {
        let entry = sizes
            .entry(entity)
            .or_insert((brick.scale, brick.scale, 1.0));
        // A bigger brick than last seen: start growing from the old size.
        if brick.scale > entry.1 + 1e-4 {
            *entry = (entry.1, brick.scale, 0.0);
            if !shake.held.iter().any(|h| h.0 == entity) {
                shake.held.push((entity, config.cells(look.large_shake)));
            }
            // LargeBrickModifier: SFX_LARGE_BRICK as it grows, and SFX_RUMBLE looping while
            // the brick is steered (CurrentBrickAudioEffect, until it touches something).
            if let Some(sound) = art.sounds.get("sfx_enlarge_bricks") {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::DESPAWN));
            }
            if let Some(sound) = art.sounds.get("sfx_rumble") {
                commands.spawn((AudioPlayer(sound.clone()), PlaybackSettings::LOOP, LargeRumble(entity)));
            }
        } else if brick.scale < entry.1 - 1e-4 {
            *entry = (brick.scale, brick.scale, 1.0);
        }
        let (from, to, progress) = *entry;
        // No Rotate refusing a turn shakes the brick sideways; a curse wave's lift
        // makes its picture jump about.
        let shaking = control.refused > 0.0 || control.picture_shake > 0.0;
        let was_shaking = if shaking {
            !shook.insert(entity)
        } else {
            shook.remove(&entity)
        };
        // Only touch the cells while a grow or shake runs (the last step puts them
        // back).
        if progress >= 1.0 && !shaking && !was_shaking {
            continue;
        }
        let progress = (progress + dt / look.large_grow.max(0.001)).min(1.0);
        entry.2 = progress;
        let factor = (from + (to - from) * progress) / to;
        // A refused turn makes the picture jump like a wave's lift, smaller on a
        // big brick (the original's shake modifier).
        let jump = if control.picture_shake > 0.0 {
            control.picture_shake
        } else {
            config.cells(config.magic.no_rotate_jump) / brick.scale.max(1.0)
        };
        let wobble = if shaking {
            // A fresh random jump every frame, ±reach on each axis.
            let mut seed = (time.elapsed_secs().to_bits() ^ entity.index_u32()) | 1;
            let mut unit = || {
                seed ^= seed << 13;
                seed ^= seed >> 17;
                seed ^= seed << 5;
                (seed % 2001) as f32 / 1000.0 - 1.0
            };
            Vec2::new(unit(), unit()) * jump
        } else {
            Vec2::ZERO
        };
        let Some(children) = children else {
            continue;
        };
        for child in children.iter() {
            let Ok((mut transform, base)) = cells.get_mut(child) else {
                continue;
            };
            let base = match base {
                Some(base) => base.0,
                None => {
                    commands.entity(child).insert(GrowBase(*transform));
                    *transform
                }
            };
            transform.translation =
                (base.translation.truncate() * factor + wobble).extend(base.translation.z);
            transform.scale = base.scale * factor;
        }
    }
}

/// No Rotate's padlock and chains on a brick (NoRotateBrickModifier's NoRotate<shape> child):
/// it stays on the brick once placed; `age` is how far its pop-in has run.
#[derive(Component)]
pub(crate) struct NoRotateLock {
    age: f32,
}

/// The holder of a brick's lock pictures, and one of them (an index into its shape's parts).
#[derive(Component)]
struct LockHolder;
#[derive(Component)]
struct LockPart(usize);

/// One piece of a lock: its picture, where it sits on the brick (cells, from the brick's
/// middle), its turn in degrees, and when its chain link grows out (0 to full width, from
/// the link's own pivot); the padlock (first) pops instead.
type LockPiece = (&'static str, f32, f32, f32, f32, f32);

const LINK: &str = "trial_chain_link";
const SHORT: &str = "trial_chain_link_short";
const END: &str = "trial_chain_link_end";
const PADLOCK: LockPiece = ("trial_lock", 0.0, 0.0, 0.0, 0.0, 0.0);

/// The NoRotate<shape> prefabs and their clips, in `pieces` order (I, O, T, S, Z, J, L), then
/// the Mystery shapes.
/// Z has no clip of its own; it takes S's timings, mirrored like its prefab.
const LOCKS: [&[LockPiece]; 10] = [
    &[
        PADLOCK,
        (LINK, -0.38, 0.0, 0.0, 0.283, 0.383),
        (LINK, 0.38, 0.0, 180.0, 0.283, 0.383),
        (LINK, -0.88, 0.0, 0.0, 0.383, 0.483),
        (LINK, 0.88, 0.0, 180.0, 0.383, 0.483),
        (SHORT, 0.0, -0.3, 90.0, 0.5, 0.617),
        (SHORT, 0.0, 0.3, -90.0, 0.5, 0.617),
        (END, -1.38, 0.0, 0.0, 0.5, 0.617),
        (END, 1.38, 0.0, 180.0, 0.5, 0.617),
    ],
    &[
        PADLOCK,
        (END, 0.0, -0.3, 90.0, 0.267, 0.383),
        (END, 0.0, 0.3, -90.0, 0.267, 0.383),
        (END, -0.38, 0.0, 0.0, 0.267, 0.383),
        (END, 0.38, 0.0, 180.0, 0.267, 0.383),
    ],
    &[
        ("trial_lock", 0.0, 0.5, 0.0, 0.0, 0.0),
        (END, 0.0, 0.2, 90.0, 0.283, 0.383),
        (END, -0.38, 0.5, 0.0, 0.283, 0.383),
        (END, 0.38, 0.5, 180.0, 0.283, 0.383),
        (END, 0.0, -0.3, 90.0, 0.4, 0.517),
        (SHORT, 0.0, 0.8, -90.0, 0.4, 0.517),
        (END, -0.88, 0.5, 0.0, 0.4, 0.517),
        (END, 0.88, 0.5, 180.0, 0.4, 0.517),
    ],
    &[
        PADLOCK,
        (LINK, -0.38, -0.31, 15.0, 0.283, 0.383),
        (LINK, 0.38, 0.31, -165.0, 0.283, 0.383),
        (END, 0.0, -0.35, 90.0, 0.4, 0.517),
        (END, -0.88, -0.46, 15.0, 0.4, 0.517),
        (END, 0.88, 0.46, -165.0, 0.4, 0.517),
        (END, 0.0, 0.35, -90.0, 0.4, 0.517),
    ],
    &[
        PADLOCK,
        (LINK, 0.38, -0.31, 165.0, 0.283, 0.383),
        (LINK, -0.38, 0.31, -15.0, 0.283, 0.383),
        (END, 0.0, -0.35, 90.0, 0.4, 0.517),
        (END, 0.88, -0.46, 165.0, 0.4, 0.517),
        (END, -0.88, 0.46, -15.0, 0.4, 0.517),
        (END, 0.0, 0.35, -90.0, 0.4, 0.517),
    ],
    &[
        ("trial_lock", 1.0, 0.5, 0.0, 0.0, 0.0),
        (LINK, 1.0, 0.18, 90.0, 0.517, 0.617),
        (LINK, 0.62, 0.5, 0.0, 0.283, 0.383),
        (LINK, 0.12, 0.5, 0.0, 0.4, 0.5),
        (LINK, -0.38, 0.5, 0.0, 0.517, 0.617),
        (END, 1.0, -0.35, 90.0, 0.717, 0.817),
        (END, -0.88, 0.5, 0.0, 0.717, 0.817),
        (SHORT, 1.3, 0.5, 180.0, 0.717, 0.817),
        (SHORT, 0.98, 0.8, -90.0, 0.717, 0.817),
    ],
    &[
        ("trial_lock", -1.0, 0.5, 0.0, 0.0, 0.0),
        (LINK, -1.0, 0.18, 90.0, 0.533, 0.633),
        (LINK, -0.62, 0.5, 180.0, 0.283, 0.383),
        (LINK, -0.12, 0.5, 180.0, 0.383, 0.517),
        (LINK, 0.38, 0.5, 180.0, 0.533, 0.633),
        (END, -1.0, -0.35, 90.0, 0.65, 0.75),
        (END, 0.88, 0.5, 180.0, 0.65, 0.75),
        (SHORT, -1.3, 0.5, 0.0, 0.65, 0.75),
        (SHORT, -1.0, 0.8, -90.0, 0.65, 0.75),
    ],
    // The Mystery shapes (NoRotateMystery01-03, in `mystery_pieces` order).
    &[
        PADLOCK,
        (LINK, 0.0, -0.38, 90.0, 0.25, 0.35),
        (LINK, 0.0, 0.38, -90.0, 0.25, 0.35),
        (LINK, 0.0, -0.88, 90.0, 0.367, 0.467),
        (LINK, 0.0, 0.88, -90.0, 0.367, 0.467),
        (END, -0.38, 0.0, 0.0, 0.483, 0.583),
        (END, 0.38, 0.0, 180.0, 0.483, 0.583),
        (END, 0.0, -1.38, 90.0, 0.483, 0.583),
        (END, 0.0, 1.38, -90.0, 0.483, 0.583),
    ],
    &[
        PADLOCK,
        (LINK, 0.0, -0.38, 90.0, 0.25, 0.35),
        (LINK, -0.29, -0.24, 60.0, 0.25, 0.35),
        (LINK, -0.38, 0.0, 0.0, 0.25, 0.35),
        (LINK, 0.38, 0.0, 180.0, 0.25, 0.35),
        (LINK, 0.0, 0.38, -90.0, 0.25, 0.35),
        (LINK, 0.0, -0.88, 90.0, 0.367, 0.467),
        (LINK, -0.579, -0.74, 60.0, 0.367, 0.467),
        (END, -0.88, 0.0, 0.0, 0.483, 0.583),
        (END, 0.88, 0.0, 180.0, 0.483, 0.583),
        (LINK, 0.0, 0.88, -90.0, 0.367, 0.467),
        (END, 0.0, -1.38, 90.0, 0.483, 0.583),
        (END, -0.879, -1.24, 60.0, 0.483, 0.583),
        (END, 0.0, 1.38, -90.0, 0.483, 0.583),
    ],
    &[
        ("trial_lock", 0.0, 0.5, 0.0, 0.0, 0.0),
        (LINK, -0.29, 0.23, 30.0, 0.25, 0.35),
        (LINK, -0.38, 0.5, 0.0, 0.25, 0.35),
        (LINK, -0.88, 0.5, 0.0, 0.367, 0.467),
        (LINK, 0.38, 0.5, 180.0, 0.25, 0.35),
        (LINK, 0.88, 0.5, 180.0, 0.367, 0.467),
        (LINK, -0.74, -0.05, 30.0, 0.367, 0.467),
        (END, -1.38, 0.5, 0.0, 0.483, 0.583),
        (END, 1.38, 0.5, 180.0, 0.483, 0.583),
        (END, 0.0, 0.18, 90.0, 0.483, 0.583),
        (END, -1.23, -0.35, 30.0, 0.483, 0.583),
        (END, 0.0, 0.88, -90.0, 0.483, 0.583),
    ],
];

/// The padlock's pop (every NoRotate clip): size 0.4 to 1.4 by 0.067 s, held, 1 by 0.233 s;
/// turned 10 degrees to -10 by 0.067 s, held to 0.134 s, back to 0 by 0.25 s.
const LOCK_SIZE: [(f32, f32); 4] = [(0.0, 0.4), (0.067, 1.4), (0.133, 1.4), (0.233, 1.0)];
const LOCK_TURN: [(f32, f32); 4] = [(0.0, 10.0), (0.067, -10.0), (0.134, -10.0), (0.25, 0.0)];
/// NoRotateBrickModifier shakes the lock by up to 0.06 units while a refused turn shakes.
const LOCK_SHAKE: f32 = 0.06;

fn keyed(keys: &[(f32, f32)], t: f32) -> f32 {
    match keys.windows(2).find(|k| t <= k[1].0) {
        Some(k) => k[0].1 + (k[1].1 - k[0].1) * ((t - k[0].0) / (k[1].0 - k[0].0)).clamp(0.0, 1.0),
        None => keys.last().map_or(1.0, |k| k.1),
    }
}

/// No Rotate puts its padlock and chains on the brick (in hand and after it lands), popping
/// in by the clip; they come back whenever the brick's pictures are redrawn.
#[allow(clippy::type_complexity)]
fn lock_bricks(
    time: Res<Time>,
    config: Res<GameConfig>,
    art: Res<Art>,
    mut commands: Commands,
    mut bricks: Query<(
        Entity,
        &Brick,
        Option<&ActiveBrick>,
        Option<&mut NoRotateLock>,
        Option<&Children>,
    )>,
    holders: Query<(), With<LockHolder>>,
    mut parts: Query<(&LockPart, &mut Transform, &ChildOf), Without<LockHolder>>,
    mut holder_moves: Query<&mut Transform, (With<LockHolder>, Without<LockPart>)>,
    mut seed: Local<u32>,
) {
    let dt = time.delta_secs();
    let cell = config.cell_size;
    for (entity, brick, control, lock, children) in &mut bricks {
        let Some(def) = LOCKS.get(brick.piece) else {
            continue;
        };
        let Some(mut lock) = lock else {
            if control.is_some_and(|c| c.mods.no_rotate) {
                commands.entity(entity).insert(NoRotateLock { age: 0.0 });
            }
            continue;
        };
        lock.age += dt;
        let Some(holder) = children.and_then(|c| c.iter().find(|c| holders.contains(*c))) else {
            // Not drawn yet, or the brick's pictures were just redrawn.
            commands.entity(entity).with_children(|b| {
                b.spawn((
                    LockHolder,
                    Transform::from_xyz(0.0, 0.0, 3.0).with_scale(Vec3::splat(brick.scale)),
                    Visibility::default(),
                ))
                .with_children(|h| {
                    for (i, &(name, x, y, turn, ..)) in def.iter().enumerate() {
                        if let Some((sprite, anchor, mut t)) =
                            crate::visuals::art_sprite(&art, name, cell)
                        {
                            t.translation = Vec3::new(x * cell, y * cell, i as f32 * 0.01);
                            t.rotation = Quat::from_rotation_z(turn.to_radians());
                            t.scale = Vec3::ZERO;
                            h.spawn((sprite, anchor, t, LockPart(i)));
                        }
                    }
                });
            });
            continue;
        };
        // A refused turn shakes the lock about (in hand only).
        if let Ok(mut t) = holder_moves.get_mut(holder) {
            let shaking = control.is_some_and(|c| c.refused > 0.0);
            let reach = LOCK_SHAKE * cell;
            let mut roll = || {
                *seed = seed.wrapping_mul(1_103_515_245).wrapping_add(12_345) | 1;
                ((*seed >> 8) as f32 / (1u32 << 24) as f32 * 2.0 - 1.0) * reach
            };
            let jump = if shaking {
                Vec2::new(roll(), roll())
            } else {
                Vec2::ZERO
            };
            t.translation = jump.extend(3.0);
        }
        let age = lock.age;
        for (part, mut t, parent) in &mut parts {
            if parent.parent() != holder {
                continue;
            }
            let Some(&(name, _, _, turn, from, to)) = def.get(part.0) else {
                continue;
            };
            let base = art
                .entries
                .get(name)
                .and_then(|e| e.pixels_per_unit)
                .map_or(1.0, |ppu| cell / ppu.max(1.0));
            if part.0 == 0 {
                t.scale = Vec3::new(base, base, 1.0) * keyed(&LOCK_SIZE, age);
                t.rotation = Quat::from_rotation_z((turn + keyed(&LOCK_TURN, age)).to_radians());
            } else {
                let grow = ((age - from) / (to - from).max(0.001)).clamp(0.0, 1.0);
                t.scale = Vec3::new(base * grow, base, 1.0);
            }
        }
    }
}

/// A brick cell's transform before the Large grow scales it.
#[derive(Component)]
struct GrowBase(Transform);

#[cfg(test)]
mod tests {
    #[test]
    fn a_picked_up_spell_blinks_five_times_then_stays() {
        use super::pickup_alpha;
        assert_eq!(pickup_alpha(0.1), 1.0);
        assert!(pickup_alpha(0.15 + 0.15) < 0.01, "dark half way through the first blink");
        assert!((pickup_alpha(0.15 + 0.3) - 1.0).abs() < 1e-3);
        assert_eq!(pickup_alpha(2.0), 1.0);
    }

    use std::time::Duration;

    use bevy::time::TimeUpdateStrategy;

    use super::*;
    use crate::config::{Curse, ModeKind};
    use crate::gameplay::PlayerInput;

    fn app() -> App {
        let mut config = GameConfig {
            mode: ModeKind::Race,
            opponents: 1,
            ..default()
        };
        config.magic.enabled = true;
        let mut app = App::new();
        app.add_plugins((MinimalPlugins, TransformPlugin, AssetPlugin::default()))
            .insert_resource(TimeUpdateStrategy::ManualDuration(Duration::from_secs_f32(
                1.0 / 64.0,
            )))
            .insert_resource(config)
            .init_resource::<Art>()
            .init_resource::<CameraShake>()
            .add_plugins((crate::TowerCorePlugin, CastsPlugin));
        app.finish();
        app.cleanup();
        for _ in 0..10 {
            app.update();
        }
        app
    }

    fn you(app: &mut App) -> Entity {
        let world = app.world_mut();
        world
            .query::<(Entity, &Tower)>()
            .iter(world)
            .find(|(_, t)| t.human)
            .unwrap()
            .0
    }

    fn count<C: Component>(app: &mut App) -> usize {
        let world = app.world_mut();
        world.query::<&C>().iter(world).count()
    }

    fn step(app: &mut App, n: usize) {
        for _ in 0..n {
            app.update();
        }
    }

    #[test]
    fn clouds_show_spells_and_casts_draw_lightning_and_beams() {
        let mut app = app();
        let tower = you(&mut app);
        {
            let mut state = app.world_mut().get_mut::<TowerState>(tower).unwrap();
            state.light_spell = Some(SpellKind::Ivy);
            state.dark_spell = Some(Curse::new(&[SpellKind::Large], 1, None));
        }
        step(&mut app, 2);
        assert_eq!(count::<SpellCloud>(&mut app), 2, "one cloud per held spell");

        // Casting light throws the dark half away: lightning, one Used, one Hide.
        app.world_mut()
            .get_mut::<PlayerInput>(tower)
            .unwrap()
            .cast_light_pressed = true;
        step(&mut app, 3);
        assert_eq!(count::<Lightning>(&mut app), 1);
        let world = app.world_mut();
        let mut stages: Vec<(bool, CloudStage)> = world
            .query::<&SpellCloud>()
            .iter(world)
            .map(|c| (c.dark, c.stage))
            .collect();
        stages.sort_by_key(|s| s.0);
        assert_eq!(
            stages,
            vec![(false, CloudStage::Used), (true, CloudStage::Hide)]
        );
        // Everything is gone after its clip.
        step(&mut app, 100);
        assert_eq!(count::<SpellCloud>(&mut app), 0);
        assert_eq!(count::<Lightning>(&mut app), 0);

        // A dark cast: a beam up from you, then one down onto the opponent.
        {
            let mut state = app.world_mut().get_mut::<TowerState>(tower).unwrap();
            state.light_spell = Some(SpellKind::Ivy);
            state.dark_spell = Some(Curse::new(&[SpellKind::Large], 1, None));
        }
        step(&mut app, 2);
        app.world_mut()
            .get_mut::<PlayerInput>(tower)
            .unwrap()
            .cast_dark_pressed = true;
        step(&mut app, 3);
        let world = app.world_mut();
        let kinds: Vec<bool> = world
            .query::<&Beam>()
            .iter(world)
            .map(|b| b.kind == BeamKind::Up)
            .collect();
        assert!(kinds.contains(&true) && kinds.contains(&false), "{kinds:?}");
        // The down beam ends in a wave over the victim's lane (all over by ~2.5 s).
        let mut saw_wave = false;
        for _ in 0..200 {
            app.update();
            let world = app.world_mut();
            saw_wave |= world
                .query::<&Beam>()
                .iter(world)
                .any(|b| b.kind == BeamKind::Lane);
        }
        assert!(saw_wave);
        assert_eq!(count::<Beam>(&mut app), 0);
    }
}
