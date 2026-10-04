//! Visuals: bricks, platforms, goal and spell lines, a drop guide, a preview
//! of the next brick, a camera that keeps every tower in view, and a text HUD.
//! Bricks, platforms and backgrounds use the original's art when it has been
//! extracted from your own install (see [`crate::art`]); otherwise everything
//! is drawn as plain coloured shapes.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::art::{Animated, Art};
use crate::config::{GameConfig, ModeKind, SpellKind};
use crate::gameplay::piece_def;
use crate::gameplay::{
    ActiveBrick, Brick, BrickBag, FloorBlock, FloorPart, MAX_TOWERS, Match, Phase, Platform, Tower,
    TowerState, TowerStatus,
};
use crate::spells::{Bubbled, Grassy, LifeOrb, SpellLog, Stone, Vined, Zapped};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .init_resource::<Art>()
            .init_resource::<ViewSize>()
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    dress_bricks,
                    dress_floor.after(dress_bricks),
                    flicker_effects,
                    dress_bubbles,
                    dress_life_orbs,
                    track_view_size.before(fit_backgrounds).before(frame_towers),
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
                    update_panels,
                    place_panels.after(update_panels).after(frame_towers),
                    update_lines,
                    update_mist,
                ),
            );
    }
}

/// The water strip at the depth where dropped bricks are lost. Modes whose water band
/// is turned off (Survival, Puzzle) have no water at all.
#[derive(Component)]
struct WaterStrip;

/// Size of the view in logical pixels: the window's, or the offscreen picture's.
#[derive(Resource, Clone, Copy, Debug)]
pub struct ViewSize(pub Vec2);

impl Default for ViewSize {
    fn default() -> Self {
        Self(Vec2::new(1280.0, 720.0))
    }
}

fn track_view_size(windows: Query<&Window>, mut view: ResMut<ViewSize>) {
    if let Ok(window) = windows.single() {
        let size = Vec2::new(window.width(), window.height());
        if view.0 != size {
            view.0 = size;
        }
    }
}

#[derive(Component)]
struct Hud;

/// A player panel, kept beside its tower's base by `place_panels`.
#[derive(Component)]
struct PanelLane(usize);

/// The row of player panels along the bottom: portrait, hearts and spell slots.
#[derive(Component)]
struct PanelRow;

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
    commands.spawn((
        PanelRow,
        Node {
            position_type: PositionType::Absolute,
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        },
    ));
    commands.spawn((Camera2d, Transform::from_xyz(0.0, config.cells(6.0), 0.0)));
    // A translucent blue column under the brick, like the original.
    let guide = match art.image("drop_guide") {
        Some(image) => Sprite {
            image,
            color: DROP_GUIDE,
            ..default()
        },
        None => Sprite::from_color(DROP_GUIDE.with_alpha(0.2), Vec2::ONE),
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
        commands.spawn((WaterStrip, water));
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

const DROP_GUIDE: Color = Color::srgba(133.0 / 255.0, 177.0 / 255.0, 1.0, 0.75);
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

/// Draws the Puzzle floor with the original's pieces when they are staged: a column
/// piece hanging from each block's top-left corner with tiles stacked behind it, or
/// the Pro pillar and its ledges. Unstaged, the floor stays plain cells.
#[allow(clippy::type_complexity)]
fn dress_floor(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    blocks: Query<(Entity, &FloorBlock, &Brick), Or<(Added<FloorBlock>, Changed<Brick>)>>,
) {
    let c = config.cell_size;
    for (entity, block, brick) in &blocks {
        let name = match block.part {
            FloorPart::Column(0) => "puzzle_piece_1".to_string(),
            FloorPart::Column(_) => "puzzle_piece_3".to_string(),
            FloorPart::Pillar => "puzzle_pro_center".to_string(),
            FloorPart::Ledge(n) => format!("puzzle_pro_ledge_{}", n + 1),
            FloorPart::TopLedge => "puzzle_pro_ledge_top".to_string(),
        };
        let Some((sprite, anchor, mut transform)) = art_sprite(&art, &name, c) else {
            continue;
        };
        let half = brick.cells.len() as f32 * 0.5;
        let mut node = commands.entity(entity);
        node.despawn_children();
        if matches!(block.part, FloorPart::Column(_) | FloorPart::Pillar) {
            // Pivot on the top-left, a hair left of and above the block's corner.
            transform.translation = Vec3::new(c * (-0.5 - 0.03), c * (half + 0.03), 0.0);
        }
        node.with_child((sprite, anchor, transform));
        if matches!(block.part, FloorPart::Column(_)) {
            // Tiles behind the column every 2 cells down, and a cap on the stack.
            for k in 0..6 {
                if let Some((sprite, anchor, mut t)) = art_sprite(&art, "puzzle_piece_base", c) {
                    t.translation = Vec3::new(0.0, c * (half - 1.0 - 2.0 * k as f32), -0.1);
                    node.with_child((sprite, anchor, t));
                }
            }
            if let Some((sprite, anchor, mut t)) = art_sprite(&art, "puzzle_piece_base_top", c) {
                t.translation = Vec3::new(0.0, c * (half + 0.37), -0.1);
                node.with_child((sprite, anchor, t));
            }
        }
    }
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
        // Vines and grass are drawn over the sprite when staged, else tint it.
        let vine_art = art.image("vine_overlay").filter(|_| art_drawn && vined);
        let grass_art = art.image("grass_overlay").filter(|_| art_drawn && grassy);
        if vined && vine_art.is_none() {
            color = color.mix(&VINES, if art_drawn { 0.25 } else { 0.4 });
        }
        if grassy && grass_art.is_none() {
            color = color.mix(&GRASS, if art_drawn { 0.3 } else { 0.5 });
        }
        let mut entity = commands.entity(entity);
        entity.insert(Visibility::default()).despawn_children();
        match image {
            Some(image) => {
                let min = brick.cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
                let max = brick.cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
                let size = (max - min + Vec2::ONE) * cell;
                let center = (min + max) * 0.5 * cell;
                entity.with_child((
                    CellSprite,
                    Sprite {
                        image,
                        color,
                        custom_size: Some(size),
                        ..default()
                    },
                    Transform::from_translation(center.extend(0.0)),
                ));
                for (z, overlay) in [(0.1, vine_art), (0.2, grass_art)] {
                    if let Some(overlay) = overlay {
                        entity.with_child((
                            Sprite {
                                image: overlay,
                                custom_size: Some(size),
                                ..default()
                            },
                            Transform::from_translation(center.extend(z)),
                        ));
                    }
                }
            }
            None => {
                entity.with_children(|parent| cell_sprites(parent, &brick.cells, cell, color));
            }
        }
    }
}

/// The bubble drawn around a brick lifted by the Bubble spell.
#[derive(Component)]
struct BubbleSprite;

fn dress_bubbles(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    added: Query<(Entity, &Brick), Added<Bubbled>>,
    mut popped: RemovedComponents<Bubbled>,
    children: Query<&Children>,
    bubbles: Query<(), With<BubbleSprite>>,
) {
    if let Some(image) = art.image("bubble") {
        for (entity, brick) in &added {
            let max = brick
                .cells
                .iter()
                .fold(0.0f32, |a, b| a.max(b.abs().max_element()));
            let size = (max + 1.0) * 2.0 * config.cell_size * brick.scale;
            commands.entity(entity).with_child((
                BubbleSprite,
                Sprite {
                    image: image.clone(),
                    custom_size: Some(Vec2::splat(size)),
                    ..default()
                },
                Transform::from_xyz(0.0, 0.0, 0.5),
            ));
        }
    }
    for entity in popped.read() {
        for child in children.iter_descendants(entity) {
            if bubbles.contains(child) {
                commands.entity(child).despawn();
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
/// `parallax` 1 moves with the tower; 0 stays at the same height on screen.
#[derive(Component)]
struct LaneLayer {
    offset: Vec2,
    parallax: f32,
}

/// The name a mode's art goes by in the manifest. Endless Race is dressed as a race.
fn mode_key(mode: ModeKind) -> &'static str {
    match mode.look() {
        ModeKind::Survival => "survival",
        ModeKind::Puzzle => "puzzle",
        _ => "race",
    }
}

/// Background layer names for this mode, back to front: `background_<mode>_N`
/// when staged for the mode, else `background_N`.
fn background_names(art: &Art, mode: ModeKind) -> Vec<String> {
    let endless = mode == ModeKind::Endless;
    let mode = mode_key(mode);
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
    // Endless Race has its own versions of the race islands, when staged.
    let own = Some(layers("background_endless_"))
        .filter(|l| endless && !l.is_empty())
        .unwrap_or_else(|| layers(&format!("background_{mode}_")));
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

/// A background layer pinned to the screen (manifest entries with `screen`).
#[derive(Component)]
struct ScreenLayer {
    anchor: Vec2,
    position: Vec2,
    view_height: f32,
    pixels_per_unit: f32,
    scale: f32,
    parallax: f32,
}

/// The sky gradient behind every other layer.
#[derive(Component)]
struct Sky;

/// The background scene's water band along the bottom of the screen.
#[derive(Component)]
struct WaterBand {
    height: f32,
    view_height: f32,
    parallax: f32,
}

/// A tall gradient image from `colors`, top to bottom.
fn gradient_image(colors: &[Color]) -> Image {
    use bevy::asset::RenderAssetUsages;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    const ROWS: usize = 256;
    let mut data = Vec::with_capacity(ROWS * 4);
    for row in 0..ROWS {
        // Image rows run top to bottom, like `colors`.
        let t = row as f32 / (ROWS - 1) as f32 * (colors.len() - 1) as f32;
        let i = (t.floor() as usize).min(colors.len() - 1);
        let next = (i + 1).min(colors.len() - 1);
        let color = colors[i].mix(&colors[next], t - i as f32);
        data.extend_from_slice(&color.to_srgba().to_u8_array());
    }
    Image::new(
        Extent3d {
            width: 1,
            height: ROWS as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::RENDER_WORLD,
    )
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn spawn_backgrounds(
    mut commands: Commands,
    art: Res<Art>,
    game: Res<Match>,
    mut images: ResMut<Assets<Image>>,
    camera: Query<Entity, With<Camera2d>>,
    existing: Query<
        Entity,
        Or<(
            With<Background>,
            With<ScreenLayer>,
            With<Sky>,
            With<WaterBand>,
        )>,
    >,
    mut water: Query<&mut Visibility, With<WaterStrip>>,
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
    let mode_name = mode_key(game.mode);
    // A mode's own sky and water band win over the shared ones.
    let for_mode = |name: &str| {
        art.entries
            .get(&format!("{name}_{mode_name}"))
            .or_else(|| art.entries.get(name))
    };
    if let Some(sky) = for_mode("sky") {
        let colors = sky.colors();
        if !colors.is_empty() {
            commands.entity(camera).with_child((
                Sky,
                Sprite::from_image(images.add(gradient_image(&colors))),
                Transform::from_xyz(0.0, 0.0, -950.0),
            ));
        }
    }
    let band = for_mode("water_band");
    let water_off = band.is_some_and(|b| b.colors().is_empty());
    for mut visibility in &mut water {
        visibility.set_if_neq(if water_off {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    if let Some(band) = band {
        let colors = band.colors();
        if !colors.is_empty() {
            commands.entity(camera).with_child((
                WaterBand {
                    height: band.position.map_or(6.0, |p| p.1),
                    view_height: band.view_height.unwrap_or(10.0).max(0.01),
                    parallax: band.parallax.unwrap_or(0.0),
                },
                Sprite::from_image(images.add(gradient_image(&colors))),
                Anchor::BOTTOM_CENTER,
                Transform::from_xyz(0.0, 0.0, -900.0 + band.z.unwrap_or(-0.5)),
            ));
        }
    }
    for (order, name) in background_names(&art, game.mode).iter().enumerate() {
        let Some(entry) = art.entries.get(name) else {
            continue;
        };
        let (px, py) = entry.pivot.unwrap_or((0.5, 0.5));
        let base = (
            Sprite::from_image(art.images[name].clone()),
            Anchor(Vec2::new(px - 0.5, py - 0.5)),
            Transform::from_xyz(0.0, 0.0, layer_z(&art, name, order)),
        );
        match (entry.screen, entry.position) {
            (Some((ax, ay)), position) => {
                let (x, y) = position.unwrap_or_default();
                commands.entity(camera).with_child((
                    ScreenLayer {
                        anchor: Vec2::new(ax, ay),
                        position: Vec2::new(x, y),
                        view_height: entry.view_height.unwrap_or(10.0).max(0.01),
                        pixels_per_unit: entry.pixels_per_unit.unwrap_or(100.0).max(1.0),
                        scale: entry.scale.unwrap_or(1.0),
                        parallax: entry.parallax.unwrap_or(0.0),
                    },
                    base,
                ));
            }
            // Drawn in each lane by `spawn_lane_layers`.
            (None, Some(_)) => {}
            (None, None) => {
                commands.entity(camera).with_child((Background, base));
            }
        }
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn fit_backgrounds(
    view_size: Res<ViewSize>,
    images: Res<Assets<Image>>,
    camera: Query<(&Projection, &Transform), (With<Camera2d>, Without<Background>)>,
    mut layers: Query<(&Sprite, &mut Transform), (With<Background>, Without<Camera2d>)>,
    mut sky: Query<
        &mut Sprite,
        (
            With<Sky>,
            Without<Background>,
            Without<ScreenLayer>,
            Without<Camera2d>,
        ),
    >,
    mut screen: Query<
        (&ScreenLayer, &mut Transform),
        (Without<Background>, Without<Camera2d>, Without<Sky>),
    >,
    mut bands: Query<
        (&WaterBand, &mut Sprite, &mut Transform),
        (
            Without<Background>,
            Without<Camera2d>,
            Without<Sky>,
            Without<ScreenLayer>,
        ),
    >,
    mut lowest: Local<Option<f32>>,
) {
    let Ok((Projection::Orthographic(ortho), camera)) = camera.single() else {
        return;
    };
    let window = view_size.0;
    let view = window * ortho.scale;
    for (sprite, mut transform) in &mut layers {
        let Some(image) = images.get(&sprite.image) else {
            continue;
        };
        let size = image.size_f32();
        let cover = (window.x / size.x).max(window.y / size.y);
        transform.scale = Vec3::splat(cover * ortho.scale);
    }
    for mut sprite in &mut sky {
        sprite.custom_size = Some(view);
    }
    // How far the camera has climbed above the lowest it has been.
    let low = lowest.get_or_insert(camera.translation.y);
    *low = low.min(camera.translation.y);
    let climb = camera.translation.y - *low;
    for (layer, mut transform) in &mut screen {
        let unit = view.y / layer.view_height;
        let at = (layer.anchor - Vec2::splat(0.5)) * view + layer.position * unit
            - Vec2::Y * climb * layer.parallax;
        transform.translation.x = at.x;
        transform.translation.y = at.y;
        transform.scale = Vec3::splat(unit / layer.pixels_per_unit * layer.scale);
    }
    for (band, mut sprite, mut transform) in &mut bands {
        let unit = view.y / band.view_height;
        sprite.custom_size = Some(Vec2::new(view.x, band.height * unit));
        transform.translation.y = -view.y * 0.5 - climb * band.parallax;
    }
}

/// Spawns the positioned background layers into a new platform's lane.
fn spawn_lane_layers(parent: &mut ChildSpawnerCommands, art: &Art, mode: ModeKind, cell: f32) {
    for (order, name) in background_names(art, mode).iter().enumerate() {
        let Some(entry) = art.entries.get(name) else {
            continue;
        };
        let Some((x, y)) = entry.position.filter(|_| entry.screen.is_none()) else {
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
        // Vertical only: lanes stay side by side as the camera climbs.
        let lag = camera.translation.y - platform.translation().y;
        transform.translation.x = layer.offset.x;
        transform.translation.y = layer.offset.y + lag * (1.0 - layer.parallax);
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
    let art_name = format!("platform_{}", mode_key(game.mode));
    let art_name = art_name.as_str();
    for (entity, platform) in &platforms {
        let mut base = commands.entity(entity);
        // In Puzzle the floor blocks stand in for the base.
        let floor = game.mode == ModeKind::Puzzle
            && config.puzzle.floor != crate::config::PuzzleFloor::Plain;
        match art_sprite(&art, art_name, c).filter(|_| !floor) {
            // A child, so the sprite's scale never touches the platform's collider.
            // Its pivot sits on the platform's top, half a cell above the centre.
            Some((sprite, anchor, mut transform)) => {
                transform.translation.y = c * 0.5;
                base.insert(Visibility::default())
                    .with_child((sprite, anchor, transform));
            }
            None if floor => {
                base.insert(Visibility::default());
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
                // The pacer starts below the base and rises from there.
                ModeKind::Endless => Some((
                    config.endless.intro_from,
                    "pacer_line",
                    Color::srgba(1.0, 0.55, 0.1, 0.85),
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
                match art_sprite(&art, "spell_line", c) {
                    // The original's line is a row of pickups in rainbow colours.
                    Some((sprite, anchor, transform)) => {
                        let entry = &art.entries["spell_line"];
                        let mut colors = entry.colors();
                        if colors.is_empty() {
                            colors = (0..9)
                                .map(|i| Color::hsl(i as f32 * 40.0, 0.9, 0.65))
                                .collect();
                        }
                        let count = entry.count.unwrap_or(16).max(1);
                        line.with_children(|line| {
                            for i in 0..count {
                                let x = width * ((i as f32 + 0.5) / count as f32 - 0.5);
                                line.spawn((
                                    Sprite {
                                        color: colors[i as usize % colors.len()],
                                        ..sprite.clone()
                                    },
                                    anchor,
                                    transform.with_translation(Vec3::new(x, 0.0, 0.0)),
                                ));
                            }
                        });
                    }
                    None => {
                        line.with_child(Sprite::from_color(
                            Color::srgba(1.0, 0.95, 0.7, 0.25),
                            Vec2::new(width, 2.0),
                        ));
                    }
                }
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
                // The staged fog picture if there is one; its alpha fades in and out.
                Sprite {
                    image: art.image("mist").unwrap_or_default(),
                    color: if art.image("mist").is_some() {
                        Color::WHITE.with_alpha(0.0)
                    } else {
                        Color::srgba(0.85, 0.88, 0.95, 0.0)
                    },
                    custom_size: Some(Vec2::new(
                        config.cells(config.platform_width + 6.0),
                        config.cells(40.0),
                    )),
                    ..default()
                },
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
    art: Res<Art>,
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
        // A few cells under the spawn point, clear of the text at the top.
        config.cells(floor + config.spawn_gap - 4.0),
        0.0,
    );
    let next = bag.peek();
    if preview.piece == Some(next) {
        return;
    }
    preview.piece = Some(next);
    let def = &config.pieces[next];
    let cells = def.centered_cells();
    let cell = config.cell_size * 0.8;
    let mut preview = commands.entity(entity);
    preview.despawn_children();
    match art.image(&format!("brick_{}", def.name)) {
        Some(image) => {
            let min = cells.iter().fold(Vec2::MAX, |a, &b| a.min(b));
            let max = cells.iter().fold(Vec2::MIN, |a, &b| a.max(b));
            preview.with_child((
                Sprite {
                    image,
                    custom_size: Some((max - min + Vec2::ONE) * cell),
                    ..default()
                },
                Transform::from_translation(((min + max) * 0.5 * cell).extend(0.0)),
            ));
        }
        None => {
            let color = Color::srgb(def.color[0], def.color[1], def.color[2]);
            preview.with_children(|parent| cell_sprites(parent, &cells, cell, color));
        }
    }
}

/// Keeps every tower, from the platforms to the top of the highest one, in view.
fn frame_towers(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    view_size: Res<ViewSize>,
    towers: Query<(&Tower, &TowerState)>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let Ok((mut transform, mut projection)) = camera.single_mut() else {
        return;
    };
    let window = view_size.0;
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
    let scale = (width / window.x).max(height / window.y).max(0.5);
    let target = Vec2::new((min_x + max_x) * 0.5, height * 0.5 - config.cells(below));

    let t = 1.0 - (-3.0 * time.delta_secs()).exp();
    let current = transform.translation.truncate();
    transform.translation += ((target - current) * t).extend(0.0);
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale += (scale - ortho.scale) * t;
    }
}

/// What one player's panel shows; the panels are rebuilt when any of it changes.
#[derive(PartialEq, Clone, Debug)]
struct PanelState {
    lane: usize,
    name: String,
    lives: Option<u32>,
    light: Option<SpellKind>,
    dark: Vec<SpellKind>,
    status: TowerStatus,
}

const PANEL_ICON: f32 = 44.0;

/// One panel per tower, in lane order: the wizard portrait (`wizard_<lane>`), hearts
/// for lives (`heart`), and the light and dark spell slots (`spell_icon_<Spell>` in
/// `spell_slot_light` / `spell_slot_dark`). Text stands in for missing art.
fn update_panels(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    towers: Query<(&Tower, &TowerState)>,
    row: Query<Entity, With<PanelRow>>,
    mut shown: Local<Option<Vec<PanelState>>>,
) {
    let Ok(row) = row.single() else {
        return;
    };
    let mut panels: Vec<PanelState> = towers
        .iter()
        .filter(|_| game.phase != Phase::Menu)
        .map(|(tower, state)| PanelState {
            lane: tower.index,
            name: match tower.player {
                Some(p) => format!("Player {}", p + 1),
                None => format!("Wizard {}", tower.index),
            },
            lives: state.lives,
            light: state.light_spell.filter(|_| config.magic.enabled),
            dark: state
                .dark_spell
                .as_ref()
                .filter(|_| config.magic.enabled)
                .map(|c| c.spells.clone())
                .unwrap_or_default(),
            status: state.status,
        })
        .collect();
    panels.sort_by_key(|p| p.lane);
    if shown.as_ref() == Some(&panels) && !art.is_changed() {
        return;
    }
    commands.entity(row).despawn_children();
    commands.entity(row).with_children(|row| {
        for panel in &panels {
            spawn_panel(row, &art, config.magic.enabled, panel);
        }
    });
    *shown = Some(panels);
}

/// Puts each panel beside its lane, just under the right end of the tower's base.
#[allow(clippy::too_many_arguments)]
fn place_panels(
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    images: Res<Assets<Image>>,
    view_size: Res<ViewSize>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    towers: Query<&Tower>,
    mut panels: Query<(&PanelLane, &mut Node, &ComputedNode)>,
) {
    let Ok((camera, eye)) = camera.single() else {
        return;
    };
    let window = view_size.0;
    // How far the base's art reaches below its top surface, in cells.
    let platform = format!("platform_{}", mode_key(game.mode));
    let platform = platform.as_str();
    let depth = art
        .entries
        .get(platform)
        .zip(art.image(platform).and_then(|h| images.get(&h)))
        .map_or(1.0, |(entry, image)| {
            let pivot_y = entry.pivot.map_or(0.5, |p| p.1);
            image.size_f32().y * pivot_y / entry.pixels_per_unit.unwrap_or(100.0).max(1.0)
        });
    for (lane, mut node, computed) in &mut panels {
        let Some(tower) = towers.iter().find(|t| t.index == lane.0) else {
            continue;
        };
        let corner = Vec3::new(
            tower.origin_x + config.cells(config.platform_width * 0.5),
            -config.cells(depth),
            0.0,
        );
        let Ok(at) = camera.world_to_viewport(eye, corner) else {
            continue;
        };
        // Right-aligned to the base's right end, hanging below the base. When the base
        // reaches below the bottom of the screen there is no room under it, so the
        // panel sits just right of the base instead, along the bottom edge.
        let size = computed.size() * computed.inverse_scale_factor();
        let bottom = window.y - size.y - 4.0;
        let x = if at.y <= bottom {
            at.x - size.x
        } else {
            at.x + 8.0
        };
        node.left = Val::Px(x.clamp(0.0, (window.x - size.x).max(0.0)));
        node.top = Val::Px(at.y.min(bottom).max(0.0));
    }
}

fn spawn_panel(row: &mut ChildSpawnerCommands, art: &Art, magic: bool, panel: &PanelState) {
    let label = |text: String, size: f32| {
        (
            Text::new(text),
            TextFont {
                font_size: FontSize::Px(size),
                ..default()
            },
        )
    };
    let icon = |image: Handle<Image>, size: f32| {
        (
            ImageNode::new(image),
            Node {
                width: Val::Px(size),
                height: Val::Px(size),
                ..default()
            },
        )
    };
    let faded = panel.status == TowerStatus::Out;
    row.spawn((
        PanelLane(panel.lane),
        Node {
            position_type: PositionType::Absolute,
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            row_gap: Val::Px(4.0),
            padding: UiRect::all(Val::Px(6.0)),
            ..default()
        },
        BackgroundColor(Color::srgba(0.0, 0.0, 0.0, if faded { 0.15 } else { 0.35 })),
    ))
    .with_children(|panel_ui| {
        panel_ui
            .spawn(Node {
                align_items: AlignItems::Center,
                column_gap: Val::Px(6.0),
                ..default()
            })
            .with_children(|top| {
                let portrait = format!("wizard_{}", panel.lane + 1);
                if let Some(image) = art.image(&portrait) {
                    let mut wizard = top.spawn(icon(image, 56.0));
                    if art.is_animation(&portrait) {
                        wizard.insert(Animated::new(portrait));
                    }
                }
                let status = match panel.status {
                    TowerStatus::Building => "",
                    TowerStatus::Won => " - won",
                    TowerStatus::Out => " - out",
                };
                top.spawn(label(format!("{}{status}", panel.name), 16.0));
            });
        if let Some(lives) = panel.lives {
            panel_ui
                .spawn(Node {
                    column_gap: Val::Px(2.0),
                    ..default()
                })
                .with_children(|hearts| match art.image("heart") {
                    Some(image) => {
                        for _ in 0..lives {
                            hearts.spawn(icon(image.clone(), 22.0));
                        }
                    }
                    None => {
                        hearts.spawn((
                            label("♥ ".repeat(lives as usize).trim_end().to_string(), 18.0),
                            TextColor(Color::srgb(1.0, 0.35, 0.45)),
                        ));
                    }
                });
        }
        if !magic {
            return;
        }
        panel_ui
            .spawn(Node {
                column_gap: Val::Px(8.0),
                ..default()
            })
            .with_children(|slots| {
                let slot_list = [
                    (
                        "spell_slot_light",
                        panel.light.into_iter().collect::<Vec<_>>(),
                        Color::srgb(1.0, 0.95, 0.55),
                    ),
                    (
                        "spell_slot_dark",
                        panel.dark.clone(),
                        Color::srgb(0.7, 0.45, 1.0),
                    ),
                ];
                for (frame, spells, color) in slot_list {
                    let mut slot = slots.spawn((
                        Node {
                            min_width: Val::Px(PANEL_ICON),
                            height: Val::Px(PANEL_ICON),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            padding: UiRect::horizontal(Val::Px(4.0)),
                            border: UiRect::all(Val::Px(2.0)),
                            ..default()
                        },
                        BorderColor::all(color),
                    ));
                    if let Some(image) = art.image(frame) {
                        slot.insert((ImageNode::new(image), BorderColor::all(Color::NONE)));
                    }
                    slot.with_children(|slot| {
                        for spell in spells {
                            match art.image(&format!("spell_icon_{spell:?}")) {
                                Some(image) => {
                                    slot.spawn(icon(image, PANEL_ICON - 8.0));
                                }
                                None => {
                                    slot.spawn((
                                        label(spell.name().into(), 13.0),
                                        TextColor(color),
                                    ));
                                }
                            }
                        }
                    });
                }
            });
    });
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
            "SURVIVAL  bricks {} / {}",
            you.bricks_used(),
            config.survival.bricks_to_place,
        ),
        ModeKind::Puzzle => format!(
            "PUZZLE  bricks {} / {}  stay under the laser ({:.1})",
            you.bricks_used(),
            config.puzzle.bricks,
            you.laser(&config)
        ),
        ModeKind::Endless => format!(
            "ENDLESS  score {}  best {}  pacer {:.1}",
            config.endless.score(you.height),
            config.endless.score(you.best_height),
            you.pacer
        ),
    };
    let spell = if config.magic.enabled {
        you.spell_line
            .map(|h| format!("spell line at {h:.1}"))
            .unwrap_or_default()
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
    if game.phase == Phase::Over {
        lines.push(match game.winner {
            Some(0) => "YOU WIN! Press R to play again".into(),
            Some(i) => format!(
                "{} wins. Press R to try again",
                names.get(i).cloned().unwrap_or_default()
            ),
            None => match game.mode {
                ModeKind::Puzzle => "PUZZLE FAILED. Press R to try again".into(),
                ModeKind::Endless => format!(
                    "CAUGHT! Score {}. Press R to try again",
                    config.endless.score(you.best_height)
                ),
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
            ModeKind::Endless => towers
                .get(line.tower)
                .map_or(config.endless.intro_from, |s| s.pacer),
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
