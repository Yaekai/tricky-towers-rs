//! Visuals: bricks, platforms, goal and spell lines, a drop guide, a preview
//! of the next brick, a camera that keeps every tower in view, and a text HUD.
//! Bricks, platforms and backgrounds use the original's art when it has been
//! extracted from your own install (see [`crate::art`]); otherwise everything
//! is drawn as plain coloured shapes.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::art::Art;
use crate::config::{GameConfig, ModeKind};
use crate::gameplay::piece_def;
use crate::gameplay::{
    ActiveBrick, Brick, BrickBag, MAX_TOWERS, Match, Phase, Platform, Tower, TowerState,
    TowerStatus,
};
use crate::spells::{Bubbled, Grassy, LifeOrb, SpellLog, Stone, Vined, Zapped};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .init_resource::<Art>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    dress_bricks,
                    flicker_effects,
                    dress_life_orbs,
                    spawn_backgrounds,
                    fit_backgrounds,
                    scroll_lane_layers,
                    fit_strips,
                    redress_on_art_change,
                    dress_platforms,
                    update_drop_guide,
                    update_next_preview,
                    frame_towers,
                    update_hud,
                    update_lines,
                    update_mist,
                ),
            );
    }
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct DropGuide;

/// A tower's spell line.
#[derive(Component)]
struct SpellLine {
    tower: Entity,
}

/// The race finish line or puzzle laser over a tower; both can come down.
#[derive(Component)]
struct FinishLine {
    tower: Entity,
}

/// Fog over a tower hit by Mist.
#[derive(Component)]
struct MistCloud {
    tower: Entity,
}

#[derive(Component)]
struct NextPreview {
    piece: Option<usize>,
}

/// Child sprite of a brick, one per cell.
#[derive(Component)]
struct CellSprite;

fn setup_scene(mut commands: Commands, config: Res<GameConfig>, art: Res<Art>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, config.cells(6.0), 0.0)));
    let guide = match art.image("drop_guide") {
        Some(image) => Sprite { image, ..default() },
        None => Sprite::from_color(Color::srgba(0.7, 0.85, 1.0, 0.035), Vec2::ONE),
    };
    commands.spawn((
        DropGuide,
        guide,
        Transform::from_xyz(0.0, 0.0, -2.0),
        Visibility::Hidden,
    ));
    // Water under every lane, its top at the depth where dropped bricks are lost.
    let depth = Vec3::new(0.0, -config.cells(config.kill_depth), -800.0);
    let width = config.cells(config.tower_spacing * (MAX_TOWERS as f32 + 2.0));
    if let Some(water) = art_strip(&art, "water", config.cell_size, width, depth) {
        commands.spawn(water);
    }
    commands.spawn((
        NextPreview { piece: None },
        Transform::default(),
        Visibility::Hidden,
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn cell_sprites(parent: &mut ChildSpawnerCommands, cells: &[Vec2], cell: f32, color: Color) {
    for &p in cells {
        parent.spawn((
            CellSprite,
            Sprite::from_color(color, Vec2::splat(cell * 0.94)),
            Transform::from_translation((p * cell).extend(0.0)),
        ));
    }
}

const STONE: Color = Color::srgb(0.45, 0.45, 0.48);
const VINES: Color = Color::srgb(0.2, 0.7, 0.25);
const GRASS: Color = Color::srgb(0.35, 0.8, 0.2);

/// A sprite from the extracted art, anchored at its pivot and scaled from its
/// pixels to cells.
/// An art sprite repeated sideways to a set width (lines, water). Its height is
/// the image's own, at `pixels_per_unit`.
#[derive(Component)]
struct Strip {
    width: f32,
}

/// An art sprite for a strip `width` world units wide, or `None` without the art.
fn art_strip(art: &Art, name: &str, cell: f32, width: f32, at: Vec3) -> Option<impl Bundle> {
    let (sprite, anchor, transform) = art_sprite(art, name, cell)?;
    Some((
        Strip { width },
        sprite,
        anchor,
        transform.with_translation(at),
    ))
}

/// Sizes strips once their image is loaded: the image repeats at its own size.
fn fit_strips(images: Res<Assets<Image>>, mut strips: Query<(&Strip, &mut Sprite, &Transform)>) {
    for (strip, mut sprite, transform) in &mut strips {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        if sprite.custom_size.is_some() {
            continue;
        }
        let size = image.size_f32();
        let scale = transform.scale.x.max(f32::EPSILON);
        sprite.custom_size = Some(Vec2::new(strip.width / scale, size.y));
        sprite.image_mode = SpriteImageMode::Tiled {
            tile_x: true,
            tile_y: false,
            stretch_value: 1.0,
        };
    }
}

fn art_sprite(art: &Art, name: &str, cell: f32) -> Option<(Sprite, Anchor, Transform)> {
    let image = art.image(name)?;
    let entry = art.entries.get(name)?;
    let (px, py) = entry.pivot.unwrap_or((0.5, 0.5));
    let scale = cell / entry.pixels_per_unit.unwrap_or(100.0).max(1.0);
    Some((
        Sprite::from_image(image),
        Anchor(Vec2::new(px - 0.5, py - 0.5)),
        Transform::from_scale(Vec3::splat(scale)),
    ))
}

/// Draws each brick: the original's sprite when one is staged (untinted, with its
/// stone or ice version when the brick is petrified or iced), else coloured cells.
/// Runs again when a brick changes, turns to stone, or gets vines or grass.
#[allow(clippy::type_complexity)]
fn dress_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<
        (Entity, &Brick, Has<Stone>, Has<Vined>, Has<Grassy>),
        Or<(Changed<Brick>, Added<Stone>, Added<Vined>, Added<Grassy>)>,
    >,
) {
    for (entity, brick, stone, vined, grassy) in &bricks {
        let cell = config.cell_size * brick.scale;
        let def = piece_def(&config, brick.piece);
        let key = if brick.piece < config.pieces.len() {
            format!("brick_{}", def.name)
        } else {
            format!("mystery_{}", def.name.to_lowercase())
        };
        let whole = brick.cells.len() == def.cells.len();
        let variant = |suffix: &str| art.image(&format!("{key}_{suffix}"));
        // The plain sprite stands in for a missing stone or ice version, tinted.
        let (image, mut color) = match (whole, stone, brick.ice) {
            (false, ..) => (None, brick.color),
            (true, true, _) => match variant("stone") {
                Some(image) => (Some(image), Color::WHITE),
                None => (art.image(&key), Color::WHITE.mix(&STONE, 0.75)),
            },
            (true, false, true) => match variant("ice") {
                Some(image) => (Some(image), Color::WHITE),
                None => (art.image(&key), brick.color),
            },
            (true, false, false) => (art.image(&key), Color::WHITE),
        };
        if image.is_none() {
            color = if stone {
                brick.color.mix(&STONE, 0.75)
            } else {
                brick.color
            };
        }
        let art_drawn = image.is_some();
        if vined {
            color = color.mix(&VINES, if art_drawn { 0.25 } else { 0.4 });
        }
        if grassy {
            color = color.mix(&GRASS, if art_drawn { 0.3 } else { 0.5 });
        }
        let mut entity = commands.entity(entity);
        entity.insert(Visibility::default()).despawn_children();
        match image {
            Some(image) => {
                let min = brick.cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                let max = brick.cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                let size = (max - min + Vec2::ONE) * cell;
                entity.with_child((
                    CellSprite,
                    Sprite {
                        image,
                        color,
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_translation(((min + max) * 0.5 * cell).extend(0.0)),
                ));
            }
            None => {
                entity.with_children(|parent| cell_sprites(parent, &brick.cells, cell, color));
            }
        }
    }
}

/// Undo's zap flickers the brick; a bubbled brick is drawn pale.
fn flicker_effects(
    time: Res<Time>,
    zapped: Query<&Children, With<Zapped>>,
    bubbled: Query<&Children, With<Bubbled>>,
    mut sprites: Query<&mut Sprite, With<CellSprite>>,
) {
    let on = (time.elapsed_secs() * 20.0).sin() > 0.0;
    for children in &zapped {
        for &child in children {
            if let Ok(mut sprite) = sprites.get_mut(child) {
                sprite.color = sprite.color.with_alpha(if on { 1.0 } else { 0.2 });
            }
        }
    }
    for children in &bubbled {
        for &child in children {
            if let Ok(mut sprite) = sprites.get_mut(child) {
                sprite.color = sprite.color.with_alpha(0.6);
            }
        }
    }
}

/// Draws Extra Life's heart orb.
fn dress_life_orbs(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    orbs: Query<Entity, Added<LifeOrb>>,
) {
    for entity in &orbs {
        let mut orb = commands.entity(entity);
        orb.insert(Visibility::default());
        match art_sprite(&art, "life_orb", config.cell_size) {
            Some(parts) => {
                orb.with_child(parts);
            }
            None => {
                orb.insert(Sprite::from_color(
                    Color::srgb(1.0, 0.3, 0.4),
                    Vec2::splat(config.cell_size * 0.8),
                ));
            }
        }
    }
}

/// A background layer that fills the view (manifest entries without `position`).
#[derive(Component)]
struct Background;

/// A background layer placed in a tower's lane (manifest entries with `position`).
/// `parallax` 1 moves with the tower; 0 stays fixed on screen.
#[derive(Component)]
struct LaneLayer {
    offset: Vec2,
    parallax: f32,
}

/// Background layer names for this mode, back to front: `background_<mode>_N`
/// when staged for the mode, else `background_N`.
fn background_names(art: &Art, mode: ModeKind) -> Vec<String> {
    let mode = match mode {
        ModeKind::Race => "race",
        ModeKind::Survival => "survival",
        ModeKind::Puzzle => "puzzle",
    };
    let layers = |prefix: &str| {
        let mut found: Vec<(f32, u32, String)> = art
            .entries
            .iter()
            .filter(|(name, _)| art.images.contains_key(*name))
            .filter_map(|(name, entry)| {
                let n = name.strip_prefix(prefix)?.parse::<u32>().ok()?;
                Some((entry.z.unwrap_or(n as f32), n, name.clone()))
            })
            .collect();
        found.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        found.into_iter().map(|(.., name)| name).collect::<Vec<_>>()
    };
    let own = layers(&format!("background_{mode}_"));
    if own.is_empty() {
        layers("background_")
    } else {
        own
    }
}

fn layer_z(art: &Art, name: &str, order: usize) -> f32 {
    let z = art
        .entries
        .get(name)
        .and_then(|e| e.z)
        .unwrap_or(order as f32);
    -900.0 + z
}

fn spawn_backgrounds(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    camera: Query<Entity, With<Camera2d>>,
    existing: Query<Entity, With<Background>>,
    mut shown: Local<Option<ModeKind>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    if *shown == Some(game.mode) && !art.is_changed() {
        return;
    }
    *shown = Some(game.mode);
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    for (order, name) in background_names(&art, game.mode).iter().enumerate() {
        let Some(entry) = art.entries.get(name) else {
            continue;
        };
        if entry.position.is_some() {
            continue;
        }
        let (px, py) = entry.pivot.unwrap_or((0.5, 0.5));
        commands.entity(camera).with_child((
            Background,
            Sprite::from_image(art.images[name].clone()),
            Anchor(Vec2::new(px - 0.5, py - 0.5)),
            Transform::from_xyz(0.0, 0.0, layer_z(&art, name, order)),
        ));
    }
}

fn fit_backgrounds(
    windows: Query<&Window>,
    images: Res<Assets<Image>>,
    camera: Query<&Projection, With<Camera2d>>,
    mut layers: Query<(&Sprite, &mut Transform), With<Background>>,
) {
    let (Ok(window), Ok(Projection::Orthographic(ortho))) = (windows.single(), camera.single())
    else {
        return;
    };
    for (sprite, mut transform) in &mut layers {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let size = image.size_f32();
        let cover = (window.width() / size.x).max(window.height() / size.y);
        transform.scale = Vec3::splat(cover * ortho.scale);
    }
}

/// Spawns the positioned background layers into a new platform's lane.
fn spawn_lane_layers(parent: &mut ChildSpawnerCommands, art: &Art, mode: ModeKind, cell: f32) {
    for (order, name) in background_names(art, mode).iter().enumerate() {
        let Some(entry) = art.entries.get(name) else {
            continue;
        };
        let Some((x, y)) = entry.position else {
            continue;
        };
        let Some((sprite, anchor, mut transform)) = art_sprite(art, name, cell) else {
            continue;
        };
        transform.scale *= entry.scale.unwrap_or(1.0);
        // The platform's centre is half a cell below its top, where positions start.
        let offset = Vec2::new(x, y + 0.5) * cell;
        transform.translation = offset.extend(layer_z(art, name, order));
        parent.spawn((
            LaneLayer {
                offset,
                parallax: entry.parallax.unwrap_or(1.0),
            },
            sprite,
            anchor,
            transform,
        ));
    }
}

/// Moves lane layers with less than full parallax towards the camera.
fn scroll_lane_layers(
    camera: Query<&Transform, (With<Camera2d>, Without<LaneLayer>)>,
    platforms: Query<&GlobalTransform, With<Platform>>,
    mut layers: Query<(&LaneLayer, &ChildOf, &mut Transform)>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    for (layer, child_of, mut transform) in &mut layers {
        let Ok(platform) = platforms.get(child_of.parent()) else {
            continue;
        };
        let lag = camera.translation.truncate() - platform.translation().truncate();
        let at = layer.offset + lag * (1.0 - layer.parallax);
        transform.translation.x = at.x;
        transform.translation.y = at.y;
    }
}

/// Redraws bricks and platforms when art drops out (an image failed to load).
fn redress_on_art_change(
    mut commands: Commands,
    art: Res<Art>,
    mut bricks: Query<&mut Brick>,
    platforms: Query<(Entity, &Platform)>,
) {
    if !art.is_changed() || art.is_added() {
        return;
    }
    for mut brick in &mut bricks {
        brick.set_changed();
    }
    for (entity, platform) in &platforms {
        let tower = platform.tower;
        commands
            .entity(entity)
            .despawn_children()
            .remove::<(Platform, Sprite)>()
            .insert(Platform { tower });
    }
}

/// Draws each new platform and its goal line (race finish or puzzle laser).
fn dress_platforms(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    platforms: Query<(Entity, &Platform), Added<Platform>>,
) {
    let c = config.cell_size;
    let art_name = match game.mode {
        ModeKind::Race => "platform_race",
        ModeKind::Survival => "platform_survival",
        ModeKind::Puzzle => "platform_puzzle",
    };
    for (entity, platform) in &platforms {
        let mut base = commands.entity(entity);
        match art_sprite(&art, art_name, c) {
            // A child, so the sprite's scale never touches the platform's collider.
            // Its pivot sits on the platform's top, half a cell above the centre.
            Some((sprite, anchor, mut transform)) => {
                transform.translation.y = c * 0.5;
                base.insert(Visibility::default())
                    .with_child((sprite, anchor, transform));
            }
            None => {
                base.insert(Sprite::from_color(
                    Color::srgb(0.55, 0.50, 0.45),
                    Vec2::new(config.cells(config.platform_width), c),
                ));
            }
        }
        base.with_children(|parent| {
            spawn_lane_layers(parent, &art, game.mode, c);
            let lane = config.cells(config.platform_width + 4.0);
            let line = match game.mode {
                ModeKind::Race => Some((
                    config.race.finish_height,
                    "finish_line",
                    Color::srgba(1.0, 1.0, 1.0, 0.6),
                )),
                ModeKind::Puzzle => Some((
                    config.puzzle.line_height,
                    "laser",
                    Color::srgba(1.0, 0.2, 0.2, 0.8),
                )),
                ModeKind::Survival => None,
            };
            if let Some((height, name, color)) = line {
                // The platform's centre is half a cell below its top.
                let mut line = parent.spawn((
                    FinishLine {
                        tower: platform.tower,
                    },
                    Transform::from_xyz(0.0, config.cells(height) + c * 0.5, -1.0),
                    Visibility::default(),
                ));
                match art_strip(&art, name, c, lane, Vec3::ZERO) {
                    Some(strip) => line.with_child(strip),
                    None => line.with_child(Sprite::from_color(color, Vec2::new(lane, 3.0))),
                };
            }
            if config.magic.enabled {
                // A line with an orb at each end: one light, one dark.
                let width = config.cells(config.platform_width + 2.0);
                let mut line = parent.spawn((
                    SpellLine {
                        tower: platform.tower,
                    },
                    Transform::from_xyz(0.0, 0.0, -1.5),
                    Visibility::Hidden,
                ));
                match art_strip(&art, "spell_line", c, width, Vec3::ZERO) {
                    Some(strip) => line.with_child(strip),
                    None => line.with_child(Sprite::from_color(
                        Color::srgba(1.0, 0.95, 0.7, 0.25),
                        Vec2::new(width, 2.0),
                    )),
                };
                line.with_children(|line| {
                    let orbs = [
                        (-1.0, "spell_orb_light", Color::srgb(1.0, 0.95, 0.55)),
                        (1.0, "spell_orb_dark", Color::srgb(0.65, 0.3, 0.95)),
                    ];
                    for (side, name, color) in orbs {
                        let x = side * config.cells(config.platform_width * 0.5 + 1.5);
                        match art_sprite(&art, name, c) {
                            Some((sprite, anchor, transform)) => {
                                line.spawn((
                                    sprite,
                                    anchor,
                                    transform.with_translation(Vec3::new(x, 0.0, 0.5)),
                                ));
                            }
                            None => {
                                line.spawn((
                                    Sprite::from_color(color, Vec2::splat(c * 0.6)),
                                    Transform::from_xyz(x, 0.0, 0.5).with_rotation(
                                        Quat::from_rotation_z(std::f32::consts::FRAC_PI_4),
                                    ),
                                ));
                            }
                        }
                    }
                });
            }
            parent.spawn((
                MistCloud {
                    tower: platform.tower,
                },
                Sprite::from_color(
                    Color::srgba(0.85, 0.88, 0.95, 0.0),
                    Vec2::new(
                        config.cells(config.platform_width + 6.0),
                        config.cells(40.0),
                    ),
                ),
                Transform::from_xyz(0.0, config.cells(20.0), 5.0),
            ));
        });
    }
}

/// A faint beam under your falling brick showing where it will land.
fn update_drop_guide(
    towers: Query<(&Tower, &TowerState)>,
    bricks: Query<(&ActiveBrick, &ColliderAabb)>,
    mut guide: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<DropGuide>>,
) {
    let Ok((mut transform, mut sprite, mut visibility)) = guide.single_mut() else {
        return;
    };
    let active = towers
        .iter()
        .find(|(tower, _)| tower.is_first_player())
        .and_then(|(_, state)| state.active)
        .and_then(|entity| bricks.get(entity).ok());
    let Some((active, aabb)) = active else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(distance) = active.drop_distance else {
        *visibility = Visibility::Hidden;
        return;
    };
    let width = aabb.max.x - aabb.min.x;
    let x = (aabb.min.x + aabb.max.x) * 0.5;
    transform.translation = Vec3::new(x, aabb.min.y - distance * 0.5, -2.0);
    sprite.custom_size = Some(Vec2::new(width, distance.max(1.0)));
    *visibility = Visibility::Visible;
}

/// Shows your next brick beside your spawn point.
fn update_next_preview(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<(&Tower, &TowerState, &mut BrickBag)>,
    mut preview: Query<(Entity, &mut NextPreview, &mut Transform, &mut Visibility)>,
) {
    let Ok((entity, mut preview, mut transform, mut visibility)) = preview.single_mut() else {
        return;
    };
    let Some((tower, state, mut bag)) = towers.iter_mut().find(|(t, _, _)| t.is_first_player())
    else {
        *visibility = Visibility::Hidden;
        return;
    };
    let finished_puzzle = game.mode == ModeKind::Puzzle
        && state.bricks_placed + state.bricks_dropped + 1 >= config.puzzle.bricks;
    if game.phase != Phase::Playing || finished_puzzle {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;
    let floor = if game.mode == ModeKind::Puzzle {
        state.stack_top.max(state.laser(&config))
    } else {
        state.stack_top
    };
    transform.translation = Vec3::new(
        tower.origin_x + config.cells(config.platform_width * 0.5 + 3.0),
        config.cells(floor + config.spawn_gap),
        0.0,
    );
    let next = bag.peek();
    if preview.piece == Some(next) {
        return;
    }
    preview.piece = Some(next);
    let def = &config.pieces[next];
    let color = Color::srgba(def.color[0], def.color[1], def.color[2], 0.6);
    let cells = def.centered_cells();
    commands
        .entity(entity)
        .despawn_children()
        .with_children(|parent| cell_sprites(parent, &cells, config.cell_size * 0.6, color));
}

/// Keeps every tower, from the platforms to the top of the highest one, in view.
fn frame_towers(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    windows: Query<&Window>,
    towers: Query<(&Tower, &TowerState)>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (Ok(window), Ok((mut transform, mut projection))) = (windows.single(), camera.single_mut())
    else {
        return;
    };
    if towers.is_empty() {
        return;
    }
    let (mut min_x, mut max_x, mut top) = (f32::MAX, f32::MIN, 0.0f32);
    for (tower, state) in &towers {
        min_x = min_x.min(tower.origin_x);
        max_x = max_x.max(tower.origin_x);
        top = top.max(state.stack_top);
    }
    // Follow the tower tops; the race finish only comes into view as towers near it.
    let goal = match game.mode {
        ModeKind::Puzzle => config.puzzle.line_height + config.puzzle.line_random,
        _ => 0.0,
    };
    let view_top = (top.max(goal) + config.spawn_gap + 3.0).max(16.0);
    // Just past the water line, so the whole base and what hangs under it stay in view.
    let below = config.kill_depth + 1.5;
    let height = config.cells(view_top + below);
    let width = max_x - min_x + config.cells(config.platform_width + 10.0);
    let scale = (width / window.width())
        .max(height / window.height())
        .max(0.5);
    let target = Vec2::new((min_x + max_x) * 0.5, height * 0.5 - config.cells(below));

    let t = 1.0 - (-3.0 * time.delta_secs()).exp();
    let current = transform.translation.truncate();
    transform.translation += ((target - current) * t).extend(0.0);
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale += (scale - ortho.scale) * t;
    }
}

fn update_hud(
    config: Res<GameConfig>,
    game: Res<Match>,
    log: Res<SpellLog>,
    towers: Query<(&Tower, &TowerState)>,
    mut hud: Query<&mut Text, With<Hud>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let mut sorted: Vec<_> = towers.iter().collect();
    sorted.sort_by_key(|(t, _)| t.index);
    let Some((_, you)) = sorted.iter().find(|(t, _)| t.is_first_player()) else {
        text.0.clear();
        return;
    };

    let goal = match game.mode {
        ModeKind::Race => {
            let hold = if you.hold_timer > 0.0 {
                format!(
                    "  hold {:.1}/{:.0}s",
                    you.hold_timer, config.race.hold_seconds
                )
            } else {
                String::new()
            };
            format!(
                "RACE  height {:.1} / {:.0}{hold}",
                you.height, game.finish_height
            )
        }
        ModeKind::Survival => format!(
            "SURVIVAL  bricks {} / {}  lives {}",
            you.bricks_used(),
            config.survival.bricks_to_place,
            you.lives.unwrap_or(0)
        ),
        ModeKind::Puzzle => format!(
            "PUZZLE  bricks {} / {}  stay under the laser ({:.1})",
            you.bricks_used(),
            config.puzzle.bricks,
            you.laser(&config)
        ),
    };
    let spell = if config.magic.enabled {
        let light = you.light_spell.map_or("-", |s| s.name());
        let dark = you.dark_spell.as_ref().map_or("-".into(), |c| c.name());
        let line = you
            .spell_line
            .map(|h| format!("   spell line at {h:.1}"))
            .unwrap_or_default();
        format!("light [Q]: {light}   dark [E]: {dark}{line}")
    } else {
        String::new()
    };
    let mist = if you.mist_timer > 0.0 { "   MIST!" } else { "" };
    let mut lines = vec![
        goal,
        format!("dropped {}   {spell}{mist}", you.bricks_dropped),
    ];

    let name = |tower: &Tower| match tower.player {
        Some(p) => format!("Player {}", p + 1),
        None => format!("Wizard {}", tower.index),
    };
    let names: Vec<String> = sorted.iter().map(|(t, _)| name(t)).collect();
    for (tower, state) in sorted.iter().filter(|(t, _)| !t.is_first_player()) {
        let status = match state.status {
            TowerStatus::Building => String::new(),
            TowerStatus::Won => " (won)".into(),
            TowerStatus::Out => " (out)".into(),
        };
        let lives = state
            .lives
            .map(|l| format!(" lives {l}"))
            .unwrap_or_default();
        lines.push(format!(
            "{}: height {:.1}{lives}{status}",
            name(tower),
            state.height
        ));
    }
    if let Some(last) = log.0.last() {
        let who = |i: usize| {
            if i == 0 {
                "You".to_string()
            } else {
                names.get(i).cloned().unwrap_or_default()
            }
        };
        let on = if last.caster == last.target {
            String::new()
        } else {
            format!(" on {}", who(last.target).to_lowercase())
        };
        lines.push(format!(
            "{} cast {}{on}",
            who(last.caster),
            last.spell.name()
        ));
    }
    lines.push("Left/Right move  Shift+Left/Right nudge  Up/Z rotate  Down drop  Q light  E dark  R restart  Esc menu".into());
    if game.phase == Phase::Over {
        lines.push(match game.winner {
            Some(0) => "YOU WIN! Press R to play again".into(),
            Some(i) => format!(
                "{} wins. Press R to try again",
                names.get(i).cloned().unwrap_or_default()
            ),
            None => match game.mode {
                ModeKind::Puzzle => "PUZZLE FAILED. Press R to try again".into(),
                _ => "OUT OF LIVES. Press R to try again".into(),
            },
        });
    }
    text.0 = lines.join("\n");
}

/// Moves each tower's spell line and the race finish line to where they are now.
fn update_lines(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<&TowerState>,
    mut lines: Query<(&SpellLine, &mut Transform, &mut Visibility), Without<FinishLine>>,
    mut finish: Query<(&FinishLine, &mut Transform)>,
) {
    let c = config.cell_size;
    for (line, mut transform) in &mut finish {
        let height = match game.mode {
            ModeKind::Puzzle => towers
                .get(line.tower)
                .map_or(config.puzzle.line_height, |s| s.laser(&config)),
            _ => game.finish_height,
        };
        transform.translation.y = config.cells(height) + c * 0.5;
    }
    for (line, mut transform, mut visibility) in &mut lines {
        let height = towers.get(line.tower).ok().and_then(|s| s.spell_line);
        // Lines right under the finish are hidden, like the original.
        let near_finish = game.mode == ModeKind::Race
            && height.is_some_and(|h| h > game.finish_height - config.magic.line_hide_near_finish);
        match height {
            Some(h) if !near_finish => {
                transform.translation.y = config.cells(h) + c * 0.5;
                visibility.set_if_neq(Visibility::Inherited);
            }
            _ => {
                visibility.set_if_neq(Visibility::Hidden);
            }
        }
    }
}

/// Fog over the top of a tower while Mist lasts, fading out at the end.
fn update_mist(
    config: Res<GameConfig>,
    towers: Query<&TowerState>,
    mut clouds: Query<(&MistCloud, &mut Sprite, &mut Transform)>,
) {
    let c = config.cell_size;
    for (cloud, mut sprite, mut transform) in &mut clouds {
        let Ok(state) = towers.get(cloud.tower) else {
            continue;
        };
        let height = config.magic.mist_height;
        transform.translation.y = config.cells(state.height + height * 0.5) + c * 0.5;
        sprite.custom_size = Some(Vec2::new(
            config.cells(config.platform_width + 6.0),
            config.cells(height * 2.0),
        ));
        let fade = (state.mist_timer / 1.5).min(1.0);
        sprite.color = sprite.color.with_alpha(fade * 0.92);
    }
}
