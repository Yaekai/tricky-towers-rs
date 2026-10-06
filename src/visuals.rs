//! Visuals: bricks, platforms, goal and spell lines, a drop guide, a preview
//! of the next brick, a camera that keeps every tower in view, and a text HUD.
//! Bricks, platforms and backgrounds use the original's art when it has been
//! extracted from your own install (see [`crate::art`]); otherwise everything
//! is drawn as plain coloured shapes.

use avian2d::prelude::*;
use bevy::prelude::*;
use bevy::sprite::Anchor;

use crate::art::{Animated, Art};
use crate::config::{GameConfig, ModeKind};
use crate::gameplay::piece_def;
use crate::gameplay::{
    ActiveBrick, Brick, FloorBlock, FloorPart, Match, Phase, Platform, Tower, TowerState,
};
use crate::spells::{Bubbled, Grassy, LifeOrb, SpellLog, Stone, Vined, Zapped};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .init_resource::<Art>()
            .init_resource::<ViewSize>()
            .init_resource::<MenuLook>()
            .add_systems(Startup, setup_scene)
            .add_systems(Update, wobble_bubbles)
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
                    light_layers,
                    grey_wave_layers,
                    scroll_lane_layers,
                    fit_strips,
                    redress_on_art_change,
                    dress_platforms,
                    update_drop_guide.after(frame_towers),
                    frame_towers.in_set(FramedCamera),
                    update_hud,
                    update_lines,
                    show_trial_clock,
                    update_mist,
                ),
            );
    }
}

/// The mode whose background shows behind a menu (the Endless screen), if any.
#[derive(Resource, Default, PartialEq)]
pub struct MenuLook(pub Option<ModeKind>);

/// The water strip at the depth where dropped bricks are lost. Modes whose water band
/// is turned off (Survival, Puzzle) have no water at all.
#[derive(Component)]
struct WaterStrip;

/// After this set the camera is where it will be drawn from this frame.
#[derive(SystemSet, Debug, Clone, PartialEq, Eq, Hash)]
pub struct FramedCamera;

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

/// A tower's drop guide: a pale column from the top of the screen down to where
/// its falling brick will land.
#[derive(Component)]
struct DropGuide {
    tower: Entity,
}

/// A tower's spell line.
#[derive(Component)]
struct SpellLine {
    tower: Entity,
}

/// One diamond of a spell line.
#[derive(Component)]
struct SpellDiamond;

/// The race trial's clock on the finish line: the hourglass and the seconds left.
#[derive(Component)]
struct TrialClock;

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

/// Child sprite of a brick, one per cell.
#[derive(Component)]
pub(crate) struct CellSprite;

fn setup_scene(mut commands: Commands, config: Res<GameConfig>, art: Res<Art>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, config.cells(6.0), 0.0)));
    // Water under every lane, its top at the depth where dropped bricks are lost.
    let depth = Vec3::new(0.0, -config.cells(config.kill_depth), -800.0);
    let width = config.cells(config.lanes.screen_width * 3.0);
    if let Some(water) = art_strip(&art, "water", config.cell_size, width, depth) {
        commands.spawn((WaterStrip, water));
    }
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

pub(crate) fn cell_sprites(
    parent: &mut ChildSpawnerCommands,
    cells: &[Vec2],
    cell: f32,
    color: Color,
) {
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

pub(crate) fn art_sprite(art: &Art, name: &str, cell: f32) -> Option<(Sprite, Anchor, Transform)> {
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
pub(crate) fn dress_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    art: Res<Art>,
    bricks: Query<
        (Entity, &Brick, Has<Stone>, Has<Vined>, Has<Grassy>),
        Or<(Changed<Brick>, Added<Stone>, Added<Vined>, Added<Grassy>)>,
    >,
    skins: Query<&crate::gameplay::TowerSkin>,
) {
    for (entity, brick, stone, vined, grassy) in &bricks {
        let cell = config.cell_size * brick.scale;
        let def = piece_def(&config, brick.piece);
        let pack = skins.get(brick.tower).map(|s| s.pack).unwrap_or_default();
        // A normal brick takes its tower's brick pack art (default when not staged);
        // stone and ice keep their own looks.
        let key = if brick.piece < config.pieces.len() {
            let packed = pack.brick_art(&def.name);
            if stone || brick.ice || art.image(&packed).is_none() {
                format!("brick_{}", def.name)
            } else {
                packed
            }
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
        // Vines grow over the brick with an animation made for its shape, when staged.
        let vine_name = [
            format!("vine_overlay_{}", key.trim_start_matches("brick_")),
            "vine_overlay".into(),
        ]
        .into_iter()
        .find(|name| art.image(name).is_some())
        .unwrap_or_default();
        let vine_art = art.image(&vine_name).filter(|_| art_drawn && vined);
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
                for (z, overlay, name) in
                    [(0.1, vine_art, vine_name.as_str()), (0.2, grass_art, "")]
                {
                    if let Some(overlay) = overlay {
                        let mut child = entity.commands();
                        let id = child
                            .spawn((
                                Sprite {
                                    image: overlay,
                                    custom_size: Some(size),
                                    ..default()
                                },
                                Transform::from_translation(center.extend(z)),
                            ))
                            .id();
                        if art.is_animation(name) {
                            child.entity(id).insert(Animated::new(name));
                        }
                        entity.add_child(id);
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
    for (entity, brick) in &added {
        let max = brick
            .cells
            .iter()
            .fold(0.0f32, |a, b| a.max(b.abs().max_element()));
        let size = (max + 1.0) * 2.0 * config.cell_size * brick.scale;
        // The original has bubbles for bricks 2, 3 and 4 cells across, in 3 looks each.
        let across = ((max * 2.0 + 1.0) * brick.scale).round().clamp(2.0, 4.0) as u32;
        let look = entity.index_u32() % 3 + 1;
        let Some(image) = [
            format!("bubble_{across}_{look}"),
            format!("bubble_{across}"),
            "bubble".into(),
        ]
        .iter()
        .find_map(|name| art.image(name)) else {
            continue;
        };
        commands.entity(entity).with_child((
            BubbleSprite,
            Sprite {
                image,
                custom_size: Some(Vec2::splat(size)),
                ..default()
            },
            Transform::from_xyz(0.0, 0.0, 0.5),
        ));
    }
    for entity in popped.read() {
        for child in children.iter_descendants(entity) {
            if bubbles.contains(child) {
                commands.entity(child).despawn();
            }
        }
    }
}

/// Bubbles wobble a little, like the original's (±2.5% of their size).
fn wobble_bubbles(time: Res<Time>, mut bubbles: Query<&mut Transform, With<BubbleSprite>>) {
    let wobble = 1.0 + 0.025 * (time.elapsed_secs() * std::f32::consts::TAU).sin();
    for mut transform in &mut bubbles {
        transform.scale = Vec3::new(wobble, 2.0 - wobble, 1.0);
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
pub(crate) fn mode_key(mode: ModeKind) -> &'static str {
    match mode.look() {
        ModeKind::Survival => "survival",
        ModeKind::Puzzle => "puzzle",
        _ => "race",
    }
}

/// Background layer names for this mode, back to front: `background_<mode>_N`
/// when staged for the mode, else `background_N`.
/// The look a mode swaps to on Pro ("Special"): Race Pro's night, Puzzle Pro's eclipse.
pub(crate) fn pro_look(
    mode: ModeKind,
    difficulty: Option<crate::config::Difficulty>,
) -> Option<&'static str> {
    if difficulty != Some(crate::config::Difficulty::Pro) {
        return None;
    }
    match mode {
        ModeKind::Race => Some("race_pro"),
        ModeKind::Puzzle => Some("puzzle_pro"),
        _ => None,
    }
}

fn background_names(art: &Art, mode: ModeKind) -> Vec<String> {
    background_names_for(art, mode, None)
}

fn background_names_for(art: &Art, mode: ModeKind, pro: Option<&str>) -> Vec<String> {
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
    // Endless Race has its own versions of the race islands, and Pro its own look,
    // when staged.
    let own = Some(layers("background_endless_"))
        .filter(|l| endless && !l.is_empty())
        .or_else(|| {
            pro.map(|p| layers(&format!("background_{p}_")))
                .filter(|l| !l.is_empty())
        })
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
    /// Sideways drift in screen units a second, and how far it has gone.
    drift: f32,
    drifted: f32,
    /// The drift before a curse wave sped it up.
    base_drift: Option<f32>,
}

/// The brighter copy of a layer, shown as the lightning flashes.
#[derive(Component)]
struct LitLayer;

/// The sky gradient behind every other layer.
#[derive(Component)]
pub(crate) struct Sky;

/// The background scene's water band along the bottom of the screen.
#[derive(Component)]
struct WaterBand {
    height: f32,
    view_height: f32,
    parallax: f32,
}

/// A tall gradient image from `colors`, top to bottom.
pub(crate) fn gradient_image(colors: &[Color]) -> Image {
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
    menu_look: Res<MenuLook>,
    config: Res<GameConfig>,
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
    mut shown: Local<Option<(ModeKind, Option<&'static str>)>>,
) {
    let Ok(camera) = camera.single() else {
        return;
    };
    let mode = match menu_look.0 {
        Some(mode) if game.phase == Phase::Menu => mode,
        _ => game.mode,
    };
    let pro = pro_look(mode, config.difficulty);
    if *shown == Some((mode, pro)) && !art.is_changed() {
        return;
    }
    *shown = Some((mode, pro));
    for entity in &existing {
        commands.entity(entity).despawn();
    }
    let mode_name = mode_key(mode);
    // A Pro or Endless look's sky and water band win over the mode's, which win over the shared.
    // Endless Race has its own sunset sky.
    let endless = (mode == ModeKind::Endless).then_some("endless");
    let for_mode = |name: &str| {
        pro.or(endless)
            .and_then(|p| art.entries.get(&format!("{name}_{p}")))
            .or_else(|| art.entries.get(&format!("{name}_{mode_name}")))
            .or_else(|| art.entries.get(name))
    };
    // The readouts' sky colours stand in when the Pro sky isn't staged.
    let weather = &config.weather;
    let rgb = |c: &[[u8; 3]]| -> Vec<Color> {
        c.iter().map(|&[r, g, b]| Color::srgb_u8(r, g, b)).collect()
    };
    let pro_sky = match pro {
        Some("race_pro") => Some(rgb(&weather.race_pro_sky)),
        Some("puzzle_pro") => Some(rgb(&weather.puzzle_pro_sky)),
        _ => None,
    }
    .filter(|_| pro.is_some_and(|p| !art.entries.contains_key(&format!("sky_{p}"))));
    if let Some(colors) = pro_sky {
        commands.entity(camera).with_child((
            Sky,
            Sprite::from_image(images.add(gradient_image(&colors))),
            Transform::from_xyz(0.0, 0.0, -950.0),
        ));
    } else if let Some(sky) = for_mode("sky") {
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
    // Race Pro's water band from the readout when its own isn't staged.
    let pro_band = (pro == Some("race_pro") && !art.entries.contains_key("water_band_race_pro"))
        .then(|| {
            (
                rgb(&weather.race_pro_water),
                weather.race_pro_water_height,
                weather.race_pro_water_parallax,
                band.and_then(|b| b.view_height).unwrap_or(10.0),
            )
        });
    let water_off = pro_band.is_none() && band.is_some_and(|b| b.colors().is_empty());
    for mut visibility in &mut water {
        visibility.set_if_neq(if water_off {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
    }
    if let Some((colors, height, parallax, view_height)) = pro_band {
        commands.entity(camera).with_child((
            WaterBand {
                height,
                view_height: view_height.max(0.01),
                parallax,
            },
            Sprite::from_image(images.add(gradient_image(&colors))),
            Anchor::BOTTOM_CENTER,
            Transform::from_xyz(0.0, 0.0, -900.5),
        ));
    } else if let Some(band) = band {
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
    for (order, name) in background_names_for(&art, mode, pro).iter().enumerate() {
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
                // Each drifting layer picks its speed in the range, by its order.
                let drift = entry.drift.map_or(0.0, |(lo, hi)| {
                    let roll = ((order as f32 + 1.0) * 0.618_034).fract();
                    lo + (hi - lo) * roll
                });
                let lit = entry
                    .lit
                    .as_ref()
                    .and_then(|name| art.image(name))
                    .map(|image| Sprite {
                        image,
                        color: Color::WHITE.with_alpha(0.0),
                        ..default()
                    });
                let anchor = base.1;
                let mut layer = commands.spawn((
                    ScreenLayer {
                        anchor: Vec2::new(ax, ay),
                        position: Vec2::new(x, y),
                        view_height: entry.view_height.unwrap_or(10.0).max(0.01),
                        pixels_per_unit: entry.pixels_per_unit.unwrap_or(100.0).max(1.0),
                        scale: entry.scale.unwrap_or(1.0),
                        parallax: entry.parallax.unwrap_or(0.0),
                        drift,
                        drifted: 0.0,
                        base_drift: None,
                    },
                    base,
                    ChildOf(camera),
                ));
                if let Some(sprite) = lit {
                    layer.with_child((
                        LitLayer,
                        sprite,
                        anchor,
                        Transform::from_xyz(0.0, 0.0, 0.01),
                    ));
                }
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
    time: Res<Time>,
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
        (&mut ScreenLayer, &mut Transform),
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
    for (mut layer, mut transform) in &mut screen {
        let unit = view.y / layer.view_height;
        // A drifting layer wraps round the view with half a view to spare each side.
        let span = view.x / unit * 2.0;
        layer.drifted =
            (layer.drifted + layer.drift * time.delta_secs()).rem_euclid(span.max(0.01));
        let slide =
            Vec2::X * (layer.drifted - if layer.drift != 0.0 { span * 0.5 } else { 0.0 }) * unit;
        let at = (layer.anchor - Vec2::splat(0.5)) * view + layer.position * unit + slide
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

/// Survival's curse waves grey the background layers (clouds and islands) and speed
/// up the drifting clouds, shifting with the sky over `wave_shift` (gap-scan.md 2).
#[allow(clippy::type_complexity)]
fn grey_wave_layers(
    config: Res<GameConfig>,
    game: Res<Match>,
    mix: Option<Res<crate::weather::WaveMix>>,
    mut layers: Query<
        (&mut Sprite, Option<&mut ScreenLayer>),
        (Or<(With<ScreenLayer>, With<Background>)>, Without<LitLayer>),
    >,
) {
    if game.mode.look() != ModeKind::Survival {
        return;
    }
    let mix = mix.map_or(0.0, |m| m.0);
    let weather = &config.weather;
    let [r, g, b] = weather.survival_wave_grey;
    let tint = Color::WHITE.mix(&Color::srgb_u8(r, g, b), mix);
    for (mut sprite, layer) in &mut layers {
        if sprite.color != tint {
            sprite.color = tint;
        }
        if let Some(mut layer) = layer {
            let current = layer.drift;
            let base = *layer.base_drift.get_or_insert(current);
            let drift = base * (1.0 + (weather.survival_wave_drift - 1.0) * mix);
            if layer.drift != drift {
                layer.drift = drift;
            }
        }
    }
}

/// Layers lit by the lightning show their bright picture as the flash fades.
fn light_layers(
    flash: Option<Res<crate::weather::Flash>>,
    mut lit: Query<&mut Sprite, With<LitLayer>>,
) {
    let alpha = flash.map_or(0.0, |f| f.0);
    for mut sprite in &mut lit {
        if sprite.color.alpha() != alpha {
            sprite.color.set_alpha(alpha);
        }
    }
}

/// Spawns the positioned background layers into a new platform's lane.
pub(crate) fn spawn_lane_layers(
    parent: &mut ChildSpawnerCommands,
    art: &Art,
    mode: ModeKind,
    cell: f32,
) {
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

/// A trial's own floor: its pictures (mirrored ones flipped about their pivot, like
/// the original), or one plain block per box when none is staged.
fn dress_trial_floor(
    base: &mut EntityCommands,
    art: &Art,
    floor: &crate::config::TrialFloor,
    c: f32,
) {
    let pictures: Vec<_> = floor
        .art
        .iter()
        .filter_map(|(key, x, y, flip, turn)| {
            let (sprite, anchor, mut transform) = art_sprite(art, key, c)?;
            transform.translation = Vec3::new(x * c, y * c + c * 0.5, 0.1);
            transform.rotation = Quat::from_rotation_z(turn.to_radians());
            if *flip {
                transform.scale.x = -transform.scale.x;
            }
            Some((sprite, anchor, transform))
        })
        .collect();
    if pictures.is_empty() {
        base.with_children(|parent| {
            for b in &floor.boxes {
                let size = Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) * c;
                let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * c;
                parent.spawn((
                    Sprite::from_color(Color::srgb(0.62, 0.52, 0.40), size),
                    Transform::from_translation((centre + Vec2::Y * c * 0.5).extend(0.0)),
                ));
            }
        });
    }
    for picture in pictures {
        base.with_child(picture);
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
        // In Puzzle the floor blocks stand in for the base, and a trial's own floor
        // may replace it.
        let trial = config.trial_floor();
        let floor = (game.mode.rules() == ModeKind::Puzzle
            && config.puzzle.floor != crate::config::PuzzleFloor::Plain)
            || trial.is_some_and(|f| !f.keep_base);
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
            // No art: one plain block per collider box.
            None => {
                base.insert(Visibility::default()).with_children(|parent| {
                    for b in config.base_boxes(game.mode) {
                        let size = Vec2::new(b.x.1 - b.x.0, b.y.1 - b.y.0) * c;
                        let centre = Vec2::new(b.x.0 + b.x.1, b.y.0 + b.y.1) * 0.5 * c;
                        parent.spawn((
                            Sprite::from_color(Color::srgb(0.55, 0.50, 0.45), size),
                            Transform::from_translation((centre + Vec2::Y * c * 0.5).extend(0.0)),
                        ));
                    }
                });
            }
        }
        if let Some(trial) = trial {
            dress_trial_floor(&mut base, &art, trial, c);
        }
        base.with_children(|parent| {
            spawn_lane_layers(parent, &art, game.mode, c);
            let lane = config.cells(config.platform_width + 4.0);
            let line = match game.mode.rules() {
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
                _ => None,
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
                // The finish line's tiles end in a cap at each side, the right one turned
                // 180° (docs/research/trials-obstacles-roofs.md, "Finish line").
                if name == "finish_line" {
                    for side in [-1.0f32, 1.0] {
                        if let Some((mut sprite, anchor, transform)) =
                            art_sprite(&art, "finish_line_end", c)
                        {
                            sprite.flip_x = side > 0.0;
                            sprite.flip_y = side > 0.0;
                            line.with_child((
                                sprite,
                                anchor,
                                transform.with_translation(Vec3::new(
                                    side * (lane * 0.5 - c * 0.7),
                                    0.0,
                                    0.5,
                                )),
                            ));
                        }
                    }
                }
                // A Puzzle trial's laser comes from a glowing ball on a pillar at its
                // right end (the original's PuzzleSinglePlayerLaserLayer: PUZZLE_LASER's
                // Ball and PUZZLE_LASER_PILAR, ?: their size here).
                if name == "laser"
                    && config.trial.is_some()
                    && let Some(ball) = art.image("fx_laserball")
                {
                    let x = lane * 0.5 + c * 1.2;
                    if let Some(tile) = art.image("fx_laser_pillar_tile") {
                        let depth = config.cells(height) + c * 9.0;
                        line.with_child((
                            Sprite {
                                image: tile,
                                custom_size: Some(Vec2::new(c * 1.6, depth)),
                                image_mode: SpriteImageMode::Tiled {
                                    tile_x: false,
                                    tile_y: true,
                                    stretch_value: 1.0,
                                },
                                ..default()
                            },
                            Transform::from_xyz(x, -depth * 0.5, -0.2),
                        ));
                    }
                    if let Some(top) = art.image("fx_laser_pillar_top") {
                        line.with_child((
                            Sprite {
                                image: top,
                                custom_size: Some(Vec2::new(c * 2.4, c * 3.0)),
                                ..default()
                            },
                            Transform::from_xyz(x, -c * 0.6, -0.1),
                        ));
                    }
                    line.with_child((
                        Sprite {
                            image: ball,
                            custom_size: Some(Vec2::splat(c * 2.4)),
                            ..default()
                        },
                        Transform::from_xyz(x, 0.0, 0.6),
                    ));
                }
                // A race trial's finish line carries the time left beside an hourglass
                // (gap-scan.md 4), fading in over the first second.
                let clock = config
                    .trial
                    .and_then(|i| config.trials.get(i))
                    .and_then(|t| t.seconds)
                    .filter(|_| game.mode.rules() == ModeKind::Race);
                if clock.is_some() {
                    // Its place on the line and its size are guesses (?).
                    let x = lane * 0.5 - c * 3.0;
                    let glass = match art.image("hourglass") {
                        Some(image) => Sprite {
                            image,
                            custom_size: Some(Vec2::splat(c * 1.2)),
                            color: Color::WHITE.with_alpha(0.0),
                            ..default()
                        },
                        None => Sprite::from_color(
                            Color::srgba(1.0, 0.85, 0.4, 0.0),
                            Vec2::splat(c * 0.8),
                        ),
                    };
                    line.with_children(|l| {
                        l.spawn((
                            TrialClock,
                            glass,
                            Transform::from_xyz(x - c * 1.3, c * 0.9, 1.0),
                        ));
                        l.spawn((
                            TrialClock,
                            Text2d::new(""),
                            TextFont {
                                font_size: FontSize::Px(c * 1.1),
                                ..default()
                            },
                            // The original's gold.
                            TextColor(Color::srgba(1.0, 0.78, 0.25, 0.0)),
                            Transform::from_xyz(x + c * 1.2, c * 0.9, 1.0),
                        ));
                    });
                }
            }
            if config.magic.enabled {
                // A row of diamonds across the lane, all one colour per line
                // (`update_lines` picks it); no caps at the ends.
                let width = config.cells(config.platform_width + 2.0);
                let look = &config.hud;
                let sprite = art_sprite(&art, "spell_line", c);
                let count = art
                    .entries
                    .get("spell_line")
                    .and_then(|e| e.count)
                    .filter(|_| sprite.is_some())
                    .unwrap_or((width / config.cells(look.diamond_step)) as u32)
                    .max(1);
                let mut line = parent.spawn((
                    SpellLine {
                        tower: platform.tower,
                    },
                    Transform::from_xyz(0.0, 0.0, -1.5),
                    Visibility::Hidden,
                ));
                line.with_children(|line| {
                    for i in 0..count {
                        let x = width * ((i as f32 + 0.5) / count as f32 - 0.5);
                        match &sprite {
                            Some((sprite, anchor, transform)) => {
                                line.spawn((
                                    SpellDiamond,
                                    sprite.clone(),
                                    *anchor,
                                    transform.with_translation(Vec3::new(x, 0.0, 0.0)),
                                ));
                            }
                            None => {
                                let side = config.cells(look.diamond_size)
                                    * std::f32::consts::FRAC_1_SQRT_2;
                                line.spawn((
                                    SpellDiamond,
                                    Sprite::from_color(Color::WHITE, Vec2::splat(side)),
                                    Transform::from_xyz(x, 0.0, 0.0).with_rotation(
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

/// A pale column the width of each falling brick, from the top of the screen down
/// to where the brick will land (light cyan in Race, lilac in Survival).
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
fn update_drop_guide(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    art: Res<Art>,
    view: Res<ViewSize>,
    camera: Query<(&Transform, &Projection), (With<Camera2d>, Without<DropGuide>)>,
    towers: Query<(Entity, &TowerState), With<Tower>>,
    bricks: Query<(&ActiveBrick, &ColliderAabb)>,
    mut guides: Query<(
        Entity,
        &DropGuide,
        &mut Transform,
        &mut Sprite,
        &mut Visibility,
    )>,
) {
    for (entity, guide, ..) in &guides {
        if !towers.contains(guide.tower) {
            commands.entity(entity).despawn();
        }
    }
    let color = match game.mode.look() {
        ModeKind::Survival => config.hud.guide_survival,
        _ => config.hud.guide_race,
    };
    let color = Color::srgba(color[0], color[1], color[2], color[3]);
    for (tower, _) in &towers {
        if !guides.iter().any(|(_, g, ..)| g.tower == tower) {
            let sprite = match art.image("drop_guide") {
                Some(image) => Sprite {
                    image,
                    color,
                    ..default()
                },
                None => Sprite::from_color(color, Vec2::ONE),
            };
            commands.spawn((
                DropGuide { tower },
                sprite,
                Transform::from_xyz(0.0, 0.0, -2.0),
                Visibility::Hidden,
            ));
        }
    }
    let Ok((eye, projection)) = camera.single() else {
        return;
    };
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    let screen_top = eye.translation.y + view.0.y * 0.5 * scale;
    for (_, guide, mut transform, mut sprite, mut visibility) in &mut guides {
        let active = towers
            .get(guide.tower)
            .ok()
            .filter(|_| game.phase == Phase::Playing)
            .and_then(|(_, state)| state.active)
            .and_then(|entity| bricks.get(entity).ok());
        let Some((active, aabb)) = active else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let Some(distance) = active.drop_distance else {
            visibility.set_if_neq(Visibility::Hidden);
            continue;
        };
        let width = aabb.max.x - aabb.min.x;
        let bottom = aabb.min.y - distance;
        let top = screen_top.max(aabb.max.y);
        transform.translation =
            Vec3::new((aabb.min.x + aabb.max.x) * 0.5, (top + bottom) * 0.5, -2.0);
        sprite.custom_size = Some(Vec2::new(width, (top - bottom).max(1.0)));
        visibility.set_if_neq(Visibility::Visible);
    }
}

/// Keeps every tower, from the platforms to the top of the highest one, in view.
fn frame_towers(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    view_size: Res<ViewSize>,
    towers: Query<(&Tower, &TowerState)>,
    rival_lanes: Query<&crate::gameplay::RivalLane>,
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
    // A trial's enemy lane is framed too, and so is the lone player's wizard floating
    // left of its tower (both wizards are on screen in the original's trials).
    for lane in &rival_lanes {
        min_x = min_x.min(lane.origin_x);
        max_x = max_x.max(lane.origin_x);
        if let Some((tower, _)) = towers.iter().next() {
            let wizard = config.cells(config.wizards.solo_offset_x - 3.0);
            min_x = min_x
                .min(tower.origin_x + wizard + config.cells(config.platform_width + 10.0) / 2.0);
        }
    }
    // Follow the tower tops; the race finish only comes into view as towers near it.
    let goal = match game.mode.rules() {
        ModeKind::Puzzle => config.puzzle.line_height + config.puzzle.line_random,
        _ => 0.0,
    };
    let view_top = config.camera_top(top.max(goal));
    // Just past the water line, so the whole base and what hangs under it stay in view.
    let below = config.kill_depth + 1.5;
    let width = max_x - min_x + config.cells(config.platform_width + 10.0);
    // While playing, the camera stays close (about `camera_cells` tall) and follows
    // the tower tops, so a tall tower's base leaves the screen. Once the match is
    // over it pulls back until every tower and its base fit.
    let close = game.phase == Phase::Playing && game.mode.rules() != ModeKind::Puzzle;
    let full = view_top + below;
    let cells_tall = if close {
        config.hud.camera_cells.min(full)
    } else {
        full
    };
    let height = config.cells(cells_tall);
    let scale = (width / window.x).max(height / window.y).max(0.5);
    // Whatever height the width forces is shown from the top down.
    let shown = config.cells(full).min(window.y * scale);
    let bottom = (config.cells(view_top) - shown).max(-config.cells(below));
    let target = Vec2::new((min_x + max_x) * 0.5, bottom + shown * 0.5);

    let t = 1.0 - (-3.0 * time.delta_secs()).exp();
    let current = transform.translation.truncate();
    transform.translation += ((target - current) * t).extend(0.0);
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale += (scale - ortho.scale) * t;
    }
}

/// The debug readout in the top-left corner (height, spells, who won). The
/// original has none, so it stays empty until F3 turns it on.
fn update_hud(
    config: Res<GameConfig>,
    game: Res<Match>,
    log: Res<SpellLog>,
    keys: Option<Res<ButtonInput<KeyCode>>>,
    towers: Query<(&Tower, &TowerState)>,
    mut hud: Query<&mut Text, With<Hud>>,
    mut debug: Local<bool>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
    // F3 shows a text readout (height, spells, result) for development.
    if keys.is_some_and(|k| k.just_pressed(KeyCode::F3)) {
        *debug = !*debug;
    }
    if !*debug {
        if !text.0.is_empty() {
            text.0.clear();
        }
        return;
    }
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
        ModeKind::EndlessSurvival => format!(
            "ENDLESS SURVIVAL  score {}  bricks {}",
            you.endless_score(),
            you.bricks_placed,
        ),
        ModeKind::EndlessPuzzle => format!(
            "ENDLESS PUZZLE  score {}  stay under the laser ({:.1})",
            you.endless_score(),
            you.laser(&config)
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
            } else if i == crate::spells::ENEMY_WIZARD {
                "The enemy wizard".to_string()
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
                ModeKind::EndlessSurvival | ModeKind::EndlessPuzzle => format!(
                    "RUN OVER! Score {}. Press R to try again",
                    you.endless_score()
                ),
                _ => "OUT OF LIVES. Press R to try again".into(),
            },
        });
    }
    text.0 = lines.join("\n");
}

/// Moves each tower's spell line and the race finish line to where they are now.
/// Counts the race trial's seconds down on the finish line, fading it in.
fn show_trial_clock(
    config: Res<GameConfig>,
    game: Res<Match>,
    mut sprites: Query<&mut Sprite, With<TrialClock>>,
    mut texts: Query<(&mut Text2d, &mut TextColor), With<TrialClock>>,
) {
    let Some(seconds) = config
        .trial
        .and_then(|i| config.trials.get(i))
        .and_then(|t| t.seconds)
    else {
        return;
    };
    let alpha = game.elapsed.clamp(0.0, 1.0);
    // Minutes, seconds and hundredths, as the original's clock reads (0:58:44).
    let hundredths = ((seconds - game.elapsed).max(0.0) * 100.0) as u32;
    let words = format!(
        "{}:{:02}:{:02}",
        hundredths / 6000,
        hundredths / 100 % 60,
        hundredths % 100
    );
    for mut sprite in &mut sprites {
        sprite.color.set_alpha(alpha);
    }
    for (mut text, mut color) in &mut texts {
        if text.0 != words {
            text.0 = words.clone();
        }
        color.0.set_alpha(alpha);
    }
}

fn update_lines(
    config: Res<GameConfig>,
    game: Res<Match>,
    towers: Query<&TowerState>,
    art: Res<Art>,
    mut lines: Query<(&SpellLine, &Children, &mut Transform, &mut Visibility), Without<FinishLine>>,
    mut finish: Query<(&FinishLine, &mut Transform, &mut Visibility), Without<SpellLine>>,
    mut diamonds: Query<&mut Sprite, With<SpellDiamond>>,
) {
    let c = config.cell_size;
    // Once a match is decided, the finish line and spell lines go at once.
    let over = game.phase == Phase::Over && game.mode.rules() != ModeKind::Puzzle;
    let colors: Vec<Color> = match art.entries.get("spell_line").map(|e| e.colors()) {
        Some(colors) if !colors.is_empty() => colors,
        _ => config
            .hud
            .line_colors
            .iter()
            .map(|&[r, g, b]| Color::srgb_u8(r, g, b))
            .collect(),
    };
    for (line, mut transform, mut visibility) in &mut finish {
        visibility.set_if_neq(if over {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        });
        let height = match game.mode.rules() {
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
    for (line, children, mut transform, mut visibility) in &mut lines {
        let state = towers.get(line.tower).ok();
        let height = state.and_then(|s| s.spell_line).filter(|_| !over);
        if let (Some(state), false) = (state, colors.is_empty()) {
            let color = colors[state.lines_reached as usize % colors.len()];
            for child in children {
                if let Ok(mut sprite) = diamonds.get_mut(*child)
                    && sprite.color != color
                {
                    sprite.color = color;
                }
            }
        }
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
