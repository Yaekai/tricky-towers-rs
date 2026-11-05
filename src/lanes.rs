//! Puzzle battles' split screen. The original gives each tower its own camera
//! (PuzzleGameMode, WidthTowerCamera) centred in its own share of the screen, all zoomed
//! out together to fit the widest tower; the background has its own camera behind them
//! and DesertForeground's sand its own in front. Ours does the same while a Puzzle battle
//! is on: a back camera draws the scene full screen, one camera per tower draws the game
//! world in that tower's viewport (so a wide tower is cut at its own lane's edge), and the
//! main camera, now last, draws the foreground and the UI over them. Every other mode keeps
//! the one main camera.
//!
//! Screen positions worked out from the main camera (HUD, scores, banners) add the
//! tower's [`LaneViews::shift`] to a point first, so they land where its own camera
//! shows it.

use bevy::camera::visibility::{RenderLayers, VisibilitySystems};
use bevy::camera::{ClearColorConfig, RenderTarget, Viewport};
use bevy::prelude::*;

use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{Match, Phase, Tower};
use crate::visuals::{FramedCamera, MainCamera, MenuScene, SCENE_LAYER, ViewSize};

/// Scene pieces that lay on the game world's layer, moved off it while the screen is split
/// so only the back camera draws them.
pub(crate) const BACK_LAYER: usize = 2;
/// Scene pieces in front of the towers (DesertForeground), drawn by the main camera last.
pub(crate) const FRONT_LAYER: usize = 4;

/// A scene piece drawn over the towers (the desert's front sand and sandstorm).
#[derive(Component)]
pub struct FrontScene;

/// One tower's camera while the screen is split: its tower's index.
#[derive(Component)]
pub struct LaneCamera(pub usize);

/// The camera behind the lane cameras that draws the scene full screen.
#[derive(Component)]
pub struct BackCamera;

/// A scene piece's own render layers from before the split, put back after it.
#[derive(Component)]
struct Relayered(Option<RenderLayers>);

/// Each tower's view while the screen is split (empty otherwise).
#[derive(Resource, Default, Clone, Debug)]
pub struct LaneViews {
    pub lanes: Vec<LaneView>,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LaneView {
    /// The tower's index.
    pub tower: usize,
    /// Its camera's centre, in world units.
    pub centre_x: f32,
    /// Half the width its camera shows, in world units.
    pub half_width: f32,
    /// Added to a point of this tower's to find where the main camera's full-screen view
    /// would show it on screen.
    pub shift: f32,
    /// Its part of the screen, as fractions of the width and height (y down from the top).
    pub rect: Rect,
}

impl LaneViews {
    pub fn split(&self) -> bool {
        !self.lanes.is_empty()
    }

    pub fn view(&self, tower: usize) -> Option<&LaneView> {
        self.lanes.iter().find(|l| l.tower == tower)
    }

    /// See [`LaneView::shift`]; 0 unless the screen is split.
    pub fn shift(&self, tower: usize) -> f32 {
        self.view(tower).map_or(0.0, |l| l.shift)
    }

    /// The world point of `tower`'s, as the main camera's math should see it.
    pub fn to_main(&self, tower: usize, p: Vec2) -> Vec2 {
        p + Vec2::X * self.shift(tower)
    }

    /// Back from the main camera's world point to `tower`'s own.
    pub fn from_main(&self, tower: usize, p: Vec2) -> Vec2 {
        p - Vec2::X * self.shift(tower)
    }
}

/// Lane `i` of `n` across the screen: its left edge and width as fractions of the width.
/// The original's Puzzle towers get the same fixed screen rectangles as every battle
/// (GamePositionHelper's margins and gaps, [`crate::config::Lanes`]).
pub fn lane_rect(lanes: &crate::config::Lanes, i: usize, n: usize) -> (f32, f32) {
    let screen = lanes.screen_width.max(0.01);
    let width = lanes.width(n);
    (
        (lanes.margin + i as f32 * lanes.spacing(n)) / screen,
        width / screen,
    )
}

/// GamePositionHelper's 16:9 rectangle on a screen of `size` (any units): the original
/// fits every tower's viewport into it. It tests `Screen.width / Screen.height` in whole
/// numbers, so 2 or more (21:9) is pillarboxed to 16:9 in the middle, and anything else is
/// letterboxed to the full width (16:9 itself fills the screen).
pub fn screen_box(size: Vec2) -> Rect {
    const RATIO: f32 = 16.0 / 9.0;
    let (w, h) = (size.x.max(1.0), size.y.max(1.0));
    if (w as u32) / (h as u32).max(1) >= 2 {
        let bw = (h * RATIO).min(w);
        Rect::new((w - bw) * 0.5, 0.0, (w + bw) * 0.5, h)
    } else {
        let bh = (w / RATIO).min(h);
        Rect::new(0.0, (h - bh) * 0.5, w, (h + bh) * 0.5)
    }
}

/// Lane `i` of `n` on a screen of `size`, as fractions of the screen (y down from the top):
/// [`lane_rect`] inside the [`screen_box`].
pub fn screen_rect(lanes: &crate::config::Lanes, i: usize, n: usize, size: Vec2) -> Rect {
    let size = size.max(Vec2::ONE);
    let b = screen_box(size);
    let (left, width) = lane_rect(lanes, i, n);
    let x0 = b.min.x + left * b.width();
    Rect::new(x0 / size.x, b.min.y / size.y, (x0 + width * b.width()) / size.x, b.max.y / size.y)
}

/// Whether the match on now splits the screen into one camera per tower.
pub fn splits(config: &GameConfig, game: &Match, towers: usize, menu: bool) -> bool {
    game.mode.rules() == ModeKind::Puzzle
        && config.trial.is_none()
        && towers > 1
        && !menu
        && game.phase != Phase::Menu
}

pub struct LanesPlugin;

impl Plugin for LanesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LaneViews>()
            .add_systems(Update, find_lane_views.after(FramedCamera))
            .add_systems(
                PostUpdate,
                (follow_lanes, relayer_scene)
                    .chain()
                    .before(bevy::transform::TransformSystems::Propagate)
                    .before(VisibilitySystems::CheckVisibility)
                    .before(bevy::camera::CameraUpdateSystems),
            );
    }
}

/// Works out each tower's view from the main camera's framing this frame.
#[allow(clippy::too_many_arguments)]
fn find_lane_views(
    config: Res<GameConfig>,
    game: Res<Match>,
    menu: Res<MenuScene>,
    view: Res<ViewSize>,
    towers: Query<&Tower>,
    camera: Query<(&Transform, &Projection), With<MainCamera>>,
    mut lanes: ResMut<LaneViews>,
) {
    let count = towers.iter().count();
    let Ok((eye, Projection::Orthographic(ortho))) = camera.single() else {
        return;
    };
    if !splits(&config, &game, count, menu.0) {
        if lanes.split() {
            lanes.lanes.clear();
        }
        return;
    }
    let (min_x, max_x) = towers.iter().fold((f32::MAX, f32::MIN), |(lo, hi), t| {
        (lo.min(t.origin_x), hi.max(t.origin_x))
    });
    // The towers' middle, where the main camera aims (frame_towers).
    let centre = (min_x + max_x) * 0.5;
    let width = view.0.x * ortho.scale;
    let mut sorted: Vec<&Tower> = towers.iter().collect();
    sorted.sort_by_key(|t| t.index);
    let found: Vec<LaneView> = sorted
        .iter()
        .enumerate()
        .map(|(i, t)| {
            let rect = screen_rect(&config.lanes, i, count, view.0);
            // The camera eases and shakes as one: each tower's sits as far from its tower
            // as the main camera does from the towers' middle.
            let centre_x = t.origin_x + (eye.translation.x - centre);
            let middle = eye.translation.x + (rect.center().x - 0.5) * width;
            LaneView {
                tower: t.index,
                centre_x,
                half_width: width * rect.width() * 0.5,
                shift: middle - centre_x,
                rect,
            }
        })
        .collect();
    if lanes.lanes != found {
        lanes.lanes = found;
    }
}

/// Spawns, places and removes the back and lane cameras, and puts the main camera last
/// while the screen is split.
#[allow(clippy::type_complexity)]
fn follow_lanes(
    mut commands: Commands,
    lanes: Res<LaneViews>,
    menu: Res<MenuScene>,
    mut main: Query<
        (
            Entity,
            &mut Camera,
            &Transform,
            &Projection,
            Option<&RenderTarget>,
            Option<&Children>,
        ),
        (With<MainCamera>, Without<LaneCamera>, Without<BackCamera>),
    >,
    mut lane_cameras: Query<
        (
            Entity,
            &LaneCamera,
            &mut Camera,
            &mut Transform,
            &mut Projection,
        ),
        (Without<MainCamera>, Without<BackCamera>),
    >,
    mut back: Query<
        (Entity, &mut Projection),
        (With<BackCamera>, Without<MainCamera>, Without<LaneCamera>),
    >,
    shake: Option<Res<crate::effects::CameraShake>>,
) {
    // Each tower's camera shakes alone (the original's shakes move that tower's camera
    // container): its lane's offset from `CameraShake`.
    let offsets = shake.map_or([Vec2::ZERO; 4], |s| s.lane_offsets);
    let off = |tower: usize| offsets.get(tower).copied().unwrap_or_default();
    let Ok((eye, mut camera, at, projection, target, _)) = main.single_mut() else {
        return;
    };
    if !lanes.split() {
        if lane_cameras.is_empty() && back.is_empty() {
            return;
        }
        for (entity, ..) in &lane_cameras {
            commands.entity(entity).despawn();
        }
        for (entity, _) in &back {
            commands.entity(entity).despawn();
        }
        camera.order = 0;
        camera.clear_color = ClearColorConfig::Default;
        let layer = if menu.0 { SCENE_LAYER } else { 0 };
        commands.entity(eye).insert(RenderLayers::layer(layer));
        return;
    }
    let count = lanes.lanes.len();
    if camera.order != count as isize + 1 {
        camera.order = count as isize + 1;
        camera.clear_color = ClearColorConfig::None;
        commands
            .entity(eye)
            .insert(RenderLayers::layer(FRONT_LAYER));
    }
    let size = camera.physical_target_size();
    match back.single_mut() {
        Ok((_, mut p)) => *p = projection.clone(),
        Err(_) => {
            let mut back = commands.spawn((
                Camera2d,
                Camera {
                    order: 0,
                    ..default()
                },
                projection.clone(),
                Transform::IDENTITY,
                RenderLayers::from_layers(&[SCENE_LAYER, BACK_LAYER]),
                BackCamera,
                ChildOf(eye),
            ));
            if let Some(target) = target {
                back.insert(target.clone());
            }
        }
    }
    let mut seen = vec![false; count];
    for (entity, lane, mut lane_camera, mut transform, mut p) in &mut lane_cameras {
        let Some((i, view)) = lanes
            .lanes
            .iter()
            .enumerate()
            .find(|(_, l)| l.tower == lane.0)
        else {
            commands.entity(entity).despawn();
            continue;
        };
        seen[i] = true;
        let o = off(view.tower);
        transform.translation = Vec3::new(view.centre_x + o.x, at.translation.y + o.y, at.translation.z);
        transform.rotation = at.rotation;
        *p = projection.clone();
        lane_camera.order = 1 + i as isize;
        lane_camera.viewport = size.map(|s| viewport(s, view.rect));
    }
    for (i, view) in lanes.lanes.iter().enumerate() {
        if seen[i] {
            continue;
        }
        let mut lane = commands.spawn((
            Camera2d,
            Camera {
                order: 1 + i as isize,
                clear_color: ClearColorConfig::None,
                viewport: size.map(|s| viewport(s, view.rect)),
                ..default()
            },
            projection.clone(),
            Transform::from_xyz(
                view.centre_x + off(view.tower).x,
                at.translation.y + off(view.tower).y,
                at.translation.z,
            ),
            RenderLayers::layer(0),
            LaneCamera(view.tower),
        ));
        if let Some(target) = target {
            lane.insert(target.clone());
        }
    }
}

fn viewport(size: UVec2, rect: Rect) -> Viewport {
    let (w, h) = (size.x as f32, size.y as f32);
    let x0 = ((w * rect.min.x).round() as u32).min(size.x.saturating_sub(1));
    let x1 = ((w * rect.max.x).round() as u32).min(size.x);
    let y0 = ((h * rect.min.y).round() as u32).min(size.y.saturating_sub(1));
    let y1 = ((h * rect.max.y).round() as u32).min(size.y);
    Viewport {
        physical_position: UVec2::new(x0, y0),
        physical_size: UVec2::new(x1.saturating_sub(x0).max(1), y1.saturating_sub(y0).max(1)),
        ..default()
    }
}

/// While the screen is split, moves the scene's pieces (the main camera's children) off the
/// game world's layer so the lane cameras don't draw them again, and the front ones onto the
/// front layer; afterwards puts their own layers back.
#[allow(clippy::type_complexity)]
fn relayer_scene(
    mut commands: Commands,
    lanes: Res<LaneViews>,
    main: Query<Entity, With<MainCamera>>,
    children: Query<&Children>,
    pieces: Query<(
        Option<&RenderLayers>,
        Has<FrontScene>,
        Has<Camera>,
        Has<Relayered>,
    )>,
    moved: Query<(Entity, &Relayered)>,
) {
    if !lanes.split() {
        for (entity, original) in &moved {
            let mut e = commands.entity(entity);
            e.remove::<Relayered>();
            match &original.0 {
                Some(layers) => e.insert(layers.clone()),
                None => e.remove::<RenderLayers>(),
            };
        }
        return;
    }
    let Ok(eye) = main.single() else {
        return;
    };
    for entity in children.iter_descendants(eye) {
        let Ok((layers, front, camera, done)) = pieces.get(entity) else {
            continue;
        };
        if camera || done {
            continue;
        }
        let own = layers.cloned();
        let now = layers.cloned().unwrap_or_default();
        let new = if front {
            RenderLayers::layer(FRONT_LAYER)
        } else if now.intersects(&RenderLayers::layer(0)) {
            now.without(0).with(BACK_LAYER)
        } else {
            continue;
        };
        commands.entity(entity).insert((new, Relayered(own)));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lanes_keep_the_original_margins_and_gaps() {
        let lanes = crate::config::Lanes::default();
        let (l0, w0) = lane_rect(&lanes, 0, 2);
        let (l1, w1) = lane_rect(&lanes, 1, 2);
        let screen = lanes.screen_width;
        assert!((l0 * screen - lanes.margin).abs() < 1e-4);
        assert!(((l1 - l0 - w0) * screen - lanes.gap).abs() < 1e-4);
        assert!(((1.0 - l1 - w1) * screen - lanes.margin).abs() < 1e-4);
        assert!((w0 - w1).abs() < 1e-6);
    }

    #[test]
    fn viewports_stay_inside_the_target() {
        let lanes = crate::config::Lanes::default();
        let size = UVec2::new(1281, 720);
        let mut right = 0;
        for i in 0..4 {
            let v = viewport(size, screen_rect(&lanes, i, 4, size.as_vec2()));
            assert!(v.physical_position.x >= right);
            right = v.physical_position.x + v.physical_size.x;
            assert!(right <= size.x);
        }
    }

    #[test]
    fn other_screens_fit_the_lanes_in_a_16_9_box() {
        // 16:9 fills the screen; 21:9 is pillarboxed; 5:4 and 16:10 are letterboxed.
        let full = screen_box(Vec2::new(1920.0, 1080.0));
        assert_eq!(full, Rect::new(0.0, 0.0, 1920.0, 1080.0));
        let wide = screen_box(Vec2::new(3440.0, 1440.0));
        assert!((wide.width() - 2560.0).abs() < 0.5 && (wide.min.x - 440.0).abs() < 0.5);
        let square = screen_box(Vec2::new(1280.0, 1024.0));
        assert!((square.height() - 720.0).abs() < 0.5 && (square.min.y - 152.0).abs() < 0.5);
        let tall = screen_box(Vec2::new(1920.0, 1200.0));
        assert!((tall.height() - 1080.0).abs() < 0.5);
        // The lanes keep their 16:9 shapes inside it.
        let lanes = crate::config::Lanes::default();
        let a = screen_rect(&lanes, 0, 2, Vec2::new(1920.0, 1080.0));
        let b = screen_rect(&lanes, 0, 2, Vec2::new(3440.0, 1440.0));
        assert!((a.width() * 1920.0 / 1080.0 - b.width() * 3440.0 / 1440.0).abs() < 1e-3);
    }

    #[test]
    fn a_shifted_point_comes_back() {
        let views = LaneViews {
            lanes: vec![LaneView {
                tower: 1,
                centre_x: 10.0,
                half_width: 5.0,
                shift: 3.5,
                rect: Rect::new(0.5, 0.0, 1.0, 1.0),
            }],
        };
        let p = Vec2::new(2.0, 7.0);
        assert_eq!(views.from_main(1, views.to_main(1, p)), p);
        assert_eq!(views.shift(0), 0.0);
    }

    /// The first acceptance check: as the shared zoom widens (a wide tower), each tower's
    /// camera still puts its tower in the middle of its own fixed screen rectangle.
    #[test]
    fn each_tower_stays_centred_in_its_rectangle_as_the_view_widens() {
        let mut app = App::new();
        let config = GameConfig {
            mode: ModeKind::Puzzle,
            ..default()
        };
        let c = config.cell_size;
        let lanes = config.lanes.clone();
        app.insert_resource(config)
            .insert_resource(Match {
                phase: Phase::Playing,
                mode: ModeKind::Puzzle,
                ..default()
            })
            .init_resource::<MenuScene>()
            .insert_resource(ViewSize(Vec2::new(1280.0, 720.0)))
            .init_resource::<LaneViews>()
            .add_systems(Update, find_lane_views);
        for (index, x) in [(0, -17.0 * c), (1, 17.0 * c)] {
            app.world_mut().spawn(Tower {
                index,
                origin_x: x,
                human: true,
                player: Some(index),
            });
        }
        let eye = app
            .world_mut()
            .spawn((
                MainCamera,
                Transform::default(),
                Projection::Orthographic(OrthographicProjection::default_2d()),
            ))
            .id();
        let mut last = None;
        for scale in [1.0, 1.6, 2.5] {
            if let Some(mut p) = app.world_mut().get_mut::<Projection>(eye)
                && let Projection::Orthographic(o) = &mut *p
            {
                o.scale = scale;
            }
            app.update();
            let views = app.world().resource::<LaneViews>().clone();
            assert_eq!(views.lanes.len(), 2);
            let width = 1280.0 * scale;
            for (i, view) in views.lanes.iter().enumerate() {
                let rect = screen_rect(&lanes, i, 2, Vec2::new(1280.0, 720.0));
                // The tower's origin, through its own camera, at its rectangle's middle.
                let origin = if i == 0 { -17.0 * c } else { 17.0 * c };
                assert!((view.centre_x - origin).abs() < 1e-3);
                let on_screen = (views.to_main(i, Vec2::new(origin, 0.0)).x / width + 0.5) * 1280.0;
                let middle = rect.center().x * 1280.0;
                assert!(
                    (on_screen - middle).abs() < 0.01,
                    "scale {scale}: {on_screen} vs {middle}"
                );
                assert!((view.half_width - width * rect.width() * 0.5).abs() < 1e-3);
            }
            last = Some(views);
        }
        // Back to the menu: no split.
        app.world_mut().resource_mut::<Match>().phase = Phase::Menu;
        app.update();
        assert!(!app.world().resource::<LaneViews>().split());
        assert!(last.is_some());
    }
}
