//! Full-screen effects drawn over the world before the menus: the grey the original's cup
//! standings popup puts behind itself (its background image uses a Black & White material on
//! the screen, under the dark modal layer), and Endless Race's pacer, which turns the screen
//! under its line grey and pink (Custom/EndlessRacePacer on the EndlessRacePaceLayer sprite,
//! 75 units of screen hanging from the line). And the drop guide (Custom/BrickGuide), which
//! grabs the screen after the background is drawn and lays its tint over it in an overlay
//! blend: here a second camera draws the background layers into a picture the guide reads.
//! Puzzle's heat haze (Custom/DesertDistortion, a post effect on the camera after the desert
//! sun and background) reads the same picture through a full-screen quad behind the towers.
//! The WGSL is ours, kept here as strings.

use bevy::core_pipeline::Core2dSystems;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::core_pipeline::schedule::Core2d;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

const GREY_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("b19f4d27-63ac-4e58-9d10-7a2c5e8f3b61");

const GREY_WGSL: &str = r#"// The screen turned grey by `amount` (the original's Black & White shader: 0.21 R +
// 0.72 G + 0.07 B, on stored sRGB values).
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct GreyOut {
    params: vec4<f32>,
    pacer: vec4<f32>,
    lanes: vec4<f32>,
    shake_x: vec4<f32>,
    shake_y: vec4<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: GreyOut;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    // A shaking player's lane (its own camera in the original) shows the screen moved by its
    // shake, kept inside the lane.
    var left = 0.0;
    var right = 1.0;
    var shift = vec2(0.0);
    for (var i = 0; i < 4; i++) {
        if in.uv.x < settings.lanes[i] {
            right = settings.lanes[i];
            shift = vec2(settings.shake_x[i], settings.shake_y[i]);
            break;
        }
        left = settings.lanes[i];
    }
    let uv = clamp(in.uv + shift, vec2(left, 0.0), vec2(right, 1.0));
    let c = textureSample(screen_texture, texture_sampler, uv);
    let s = linear_to_srgb(c.rgb);
    let g = dot(s, vec3(0.21, 0.72, 0.07));
    var o = mix(s, vec3(g), settings.params.x);
    // Under the pacer's line (params.y, from the top): grey times the tint, by params.z.
    if in.uv.y > settings.params.y {
        o = mix(o, dot(o, vec3(0.21, 0.72, 0.07)) * settings.pacer.rgb, settings.params.z);
    }
    return vec4(srgb_to_linear(o), c.a);
}
"#;

const SPOT_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("8c3f1d27-5a94-4e60-b1d8-27e9c4f5a613");

const SPOT_WGSL: &str = r#"// SpriteBlendMode/SpriteBlendModeOverlay (the winner's spotlight in the Race and Puzzle
// worlds): the picture times its colour laid over the grabbed background in an overlay
// blend, at the picture's alpha times the colour's. On stored (sRGB) values.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct SpotOverlay {
    colour: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: SpotOverlay;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var picture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var picture_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var grab: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var grab_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let p = textureSample(picture, picture_sampler, mesh.uv);
    let a = linear_to_srgb(p.rgb) * material.colour.rgb;
    let uv = (mesh.position.xy - view.viewport.xy) / view.viewport.zw;
    let b = linear_to_srgb(textureSample(grab, grab_sampler, uv).rgb);
    let low = 2.0 * a * b;
    let high = vec3(1.0) - 2.0 * (vec3(1.0) - a) * (vec3(1.0) - b);
    let o = select(high, low, b < vec3(0.5));
    return vec4(srgb_to_linear(o), p.a * material.colour.a);
}
"#;

/// The winner's spotlight drawn as SpriteBlendModeOverlay: `colour` is the sprite colour
/// (its alpha the fade).
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct SpotOverlay {
    #[uniform(0)]
    pub colour: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub picture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub grab: Handle<Image>,
}

impl Material2d for SpotOverlay {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SPOT_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const GUIDE_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("5d0b8a93-2e71-4c6f-b8d4-91f3a6c27e05");

const GUIDE_WGSL: &str = r#"// The drop guide: its tint laid over the background behind it in an overlay blend, then
// mixed in by the tint's alpha (Blend SrcAlpha OneMinusSrcAlpha). On stored (sRGB) values.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct GuideOverlay {
    // The tint as the original's 0-1 values, with its alpha.
    tint: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: GuideOverlay;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var grab: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var grab_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (mesh.position.xy - view.viewport.xy) / view.viewport.zw;
    let b = linear_to_srgb(textureSample(grab, grab_sampler, uv).rgb);
    let a = material.tint.rgb;
    let low = 2.0 * a * b;
    let high = vec3(1.0) - 2.0 * (vec3(1.0) - a) * (vec3(1.0) - b);
    let o = select(high, low, b < vec3(0.5));
    return vec4(srgb_to_linear(o), material.tint.a);
}
"#;

const HAZE_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("e2a47c18-6b39-4d05-9f8e-3c1d7b2a5e94");

const HAZE_WGSL: &str = r#"// The desert heat haze: the background read from a place moved by gradient noise over
// the screen (50 x 30 cells, drifting by the time and twice it), more towards the top and
// bottom edges and nothing across the middle.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::{view, globals},
}

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var grab: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var grab_sampler: sampler;

fn hash(p: vec2<f32>) -> vec2<f32> {
    let q = vec2(dot(p, vec2(127.1, 311.7)), dot(p, vec2(269.5, 183.3)));
    return fract(sin(q) * 43758.5469) * 2.0 - 1.0;
}

fn noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = dot(hash(i), f);
    let b = dot(hash(i + vec2(1.0, 0.0)), f - vec2(1.0, 0.0));
    let c = dot(hash(i + vec2(0.0, 1.0)), f - vec2(0.0, 1.0));
    let d = dot(hash(i + vec2(1.0, 1.0)), f - vec2(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (mesh.position.xy - view.viewport.xy) / view.viewport.zw;
    // The original's screen coordinates run up from the bottom.
    let up = vec2(uv.x, 1.0 - uv.y);
    let t = globals.time;
    let n = noise(up * vec2(50.0, 30.0) + vec2(t, 2.0 * t));
    let shift = n * abs(up.y - 0.5) * 0.00859375;
    return textureSample(grab, grab_sampler, uv + vec2(shift, -shift));
}
"#;

const LANE_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("c83f1a26-7d49-4e5b-a0c2-5b9e14d7f638");

const LANE_WGSL: &str = r#"// A dark cast's lane (Custom/BeamLaneShaderMultiply and Additive): the background behind
// the lane, read sideways by a wobble of three sines up the screen, is multiplied by (or has
// added) the lane picture times its tint, and mixed in by the picture's alpha times the
// tint's. On stored (sRGB) values.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::{view, globals},
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct BeamLane {
    // The tint as the original's 0-1 values, with its alpha.
    tint: vec4<f32>,
    // x: 1 to add the light, 0 to multiply.
    mode: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: BeamLane;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var picture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var picture_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var grab: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var grab_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let screen = (mesh.position.xy - view.viewport.xy) / view.viewport.zw;
    // Unity's grab coordinates run up from the bottom; its _Time.z is twice the time.
    let up = 1.0 - screen.y;
    let p = up * vec3(6.238, 12.476, 18.714) + vec3(4.0 * globals.time);
    let wobble = sin(p.x) * sin(p.y) * sin(p.z) * 0.03;
    let back = linear_to_srgb(textureSample(grab, grab_sampler, vec2(screen.x + wobble, screen.y)).rgb);
    let tex = textureSample(picture, picture_sampler, vec2(mesh.uv.x + wobble, mesh.uv.y));
    let lane = linear_to_srgb(tex.rgb) * material.tint.rgb;
    let o = select(lane * back, lane + back, material.mode.x > 0.5);
    return vec4(srgb_to_linear(clamp(o, vec3(0.0), vec3(1.0))), tex.a * material.tint.a);
}
"#;

/// A dark cast's lane over the grabbed background.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct BeamLane {
    #[uniform(0)]
    pub tint: Vec4,
    #[uniform(0)]
    pub mode: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub picture: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub grab: Handle<Image>,
}

impl Material2d for BeamLane {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(LANE_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// Something drawn from the grabbed background: the grab camera runs while one shows.
#[derive(Component)]
pub struct NeedsGrab;

/// The heat haze's quad: the grabbed background, drawn moved.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HeatHaze {
    #[texture(0)]
    #[sampler(1)]
    pub grab: Handle<Image>,
}

impl Material2d for HeatHaze {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(HAZE_SHADER)
    }
}

/// The heat haze quad, a child of the main camera.
#[derive(Component)]
struct Haze;

/// The depth the grab stops at: the background layers and the far sandstorm lie under it,
/// the near sandstorm (-300), the guides and the towers over it.
const GRAB_Z: f32 = -400.0;

/// The drop guide's look: its tint (stored values, as the original's) over the grabbed
/// background.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GuideOverlay {
    #[uniform(0)]
    pub tint: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub grab: Handle<Image>,
}

impl Material2d for GuideOverlay {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GUIDE_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// The camera that draws the background layers (render layer 1) into `GrabTarget` for the
/// drop guides; it only runs while a guide shows.
#[derive(Component)]
pub struct GrabCamera;

/// The picture of the background behind the guides.
#[derive(Resource)]
pub struct GrabTarget(pub Handle<Image>);

/// Keeps the grab camera on the main camera's view, at its size, and running only while a
/// drop guide shows.
#[allow(clippy::type_complexity)]
fn follow_grab(
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    target: Res<GrabTarget>,
    game: Option<Res<crate::gameplay::Match>>,
    menu: Option<Res<crate::visuals::MenuScene>>,
    view: Res<crate::visuals::ViewSize>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut hazes: ResMut<Assets<HeatHaze>>,
    mut overlays: ResMut<Assets<GuideOverlay>>,
    mut lanes: ResMut<Assets<BeamLane>>,
    main: Query<
        (Entity, &Camera, &Projection, &Transform),
        (With<crate::visuals::MainCamera>, Without<GrabCamera>, Without<Haze>),
    >,
    mut grab: Query<(&mut Camera, &mut Projection), With<GrabCamera>>,
    mut haze: Query<(&mut Transform, &mut Visibility), With<Haze>>,
    guides: Query<&ViewVisibility, Or<(With<crate::visuals::DropGuide>, With<NeedsGrab>)>>,
) {
    let Ok((eye, camera, projection, at)) = main.single() else {
        return;
    };
    // The desert's haze shows in every Puzzle match (its look), not behind the menus.
    let hazy = game.is_some_and(|g| g.mode.look() == crate::config::ModeKind::Puzzle)
        && !menu.is_some_and(|m| m.0);
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    match haze.single_mut() {
        Ok((mut transform, mut visibility)) => {
            transform.translation.z = GRAB_Z - at.translation.z;
            transform.scale = (view.0 * scale).extend(1.0);
            visibility.set_if_neq(if hazy {
                Visibility::Inherited
            } else {
                Visibility::Hidden
            });
        }
        Err(_) => {
            commands.entity(eye).with_child((
                Haze,
                Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
                MeshMaterial2d(hazes.add(HeatHaze {
                    grab: target.0.clone(),
                })),
                Transform::from_xyz(0.0, 0.0, GRAB_Z),
                Visibility::Hidden,
            ));
        }
    }
    let Ok((mut grab_camera, mut grab_projection)) = grab.single_mut() else {
        commands.entity(eye).with_child((
            Camera2d,
            Camera {
                order: -1,
                is_active: false,
                ..default()
            },
            bevy::camera::RenderTarget::Image(target.0.clone().into()),
            bevy::camera::visibility::RenderLayers::layer(1),
            Msaa::Off,
            GrabCamera,
        ));
        return;
    };
    let on = hazy || guides.iter().any(|v| v.get());
    if grab_camera.is_active != on {
        grab_camera.is_active = on;
    }
    if !on {
        return;
    }
    if let Some(size) = camera.physical_target_size()
        && let Some(image) = images.get(&target.0)
        && image.size() != size
        && let Some(mut image) = images.get_mut(&target.0)
    {
        image.resize(bevy::render::render_resource::Extent3d {
            width: size.x.max(1),
            height: size.y.max(1),
            depth_or_array_layers: 1,
        });
        // Their bind groups still hold the old picture until they change too.
        for _ in hazes.iter_mut() {}
        for _ in overlays.iter_mut() {}
        for _ in lanes.iter_mut() {}
    }
    if let (Projection::Orthographic(from), Projection::Orthographic(to)) =
        (projection, &mut *grab_projection)
    {
        // It draws only what lies under GRAB_Z (the camera looks down -z).
        let near = at.translation.z - GRAB_Z;
        // The grab picture is in screen pixels and an image has no display scaling, so with
        // Windows scaling (125%) the same zoom would see 1.25 times as much: zoom in by it.
        let scaling = camera.target_scaling_factor().unwrap_or(1.0).max(0.01);
        let scale = from.scale / scaling;
        if to.scale != scale || to.near != near {
            *to = from.clone();
            to.scale = scale;
            to.near = near;
        }
    }
}

/// On the camera: how grey the world is drawn (x: 0 none, 1 fully), and the pacer's grey:
/// y its line's place on the screen (0 top, 1 bottom), z its strength, `pacer` its tint.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct GreyOut {
    pub params: Vec4,
    pub pacer: Vec4,
    /// Each lane's right edge on the screen (0 to 1, left to right; past the last, 2), and the
    /// shift of each lane's picture this frame (screen shares; `effects::CameraShake`).
    pub lanes: Vec4,
    pub shake_x: Vec4,
    pub shake_y: Vec4,
}

impl Default for GreyOut {
    fn default() -> Self {
        Self {
            params: Vec4::new(0.0, 2.0, 0.0, 0.0),
            pacer: Vec4::ONE,
            lanes: Vec4::splat(2.0),
            shake_x: Vec4::ZERO,
            shake_y: Vec4::ZERO,
        }
    }
}

/// Lays each shaking lane's shift on the camera: lanes split halfway between towers.
fn follow_lane_shake(
    shake: Option<Res<crate::effects::CameraShake>>,
    towers: Query<&crate::gameplay::Tower>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut GreyOut)>,
) {
    let offsets = shake.map_or([Vec2::ZERO; 4], |s| s.lane_offsets);
    let mut order: Vec<(f32, usize)> = towers.iter().map(|t| (t.origin_x, t.index)).collect();
    order.sort_by(|a, b| a.0.total_cmp(&b.0));
    for (camera, at, mut grey) in &mut cameras {
        let Some(size) = camera.logical_viewport_size() else {
            continue;
        };
        let screen = |p: Vec2| camera.world_to_viewport(at, p.extend(0.0)).ok().map(|v| v / size);
        let mut lanes = Vec4::splat(2.0);
        let (mut sx, mut sy) = (Vec4::ZERO, Vec4::ZERO);
        if order.len() > 1 {
            for (slot, &(x, index)) in order.iter().take(4).enumerate() {
                let edge = order.get(slot + 1).and_then(|&(next, _)| screen(Vec2::new((x + next) * 0.5, 0.0)));
                lanes[slot] = edge.map_or(2.0, |e| e.x);
                let off = offsets.get(index).copied().unwrap_or_default();
                // The camera moves by the shake, so the picture moves the other way.
                if let (Some(a), Some(b)) = (screen(Vec2::ZERO), screen(off)) {
                    let d = b - a;
                    sx[slot] = d.x;
                    sy[slot] = d.y;
                }
            }
        }
        if grey.lanes != lanes || grey.shake_x != sx || grey.shake_y != sy {
            grey.lanes = lanes;
            grey.shake_x = sx;
            grey.shake_y = sy;
        }
    }
}

impl FullscreenMaterial for GreyOut {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GREY_SHADER)
    }

    fn schedule() -> impl bevy::ecs::schedule::ScheduleLabel + Clone {
        Core2d
    }

    fn schedule_configs(system: ScheduleConfigs<BoxedSystem>) -> ScheduleConfigs<BoxedSystem> {
        system.in_set(Core2dSystems::PostProcess)
    }
}

/// How grey the world should be; the camera's grey follows it over 0.2 s (the popup's fade).
#[derive(Resource, Default)]
pub struct GreyTarget(pub f32);

const FADE_SECONDS: f32 = 0.2;
/// The pacer material's _TintColor.
const PACER_TINT: Vec4 = Vec4::new(1.0, 0.8455882, 1.0, 1.0);
/// The pacer's grey eases to 0.2 over 1 s (out-sine) when it stops at a checkpoint and back
/// to 1 over 3 s (in-sine) when it goes on (LeaderboardPaceLineLayer).
const PACER_PAUSED: f32 = 0.2;
const PACER_STOP_SECONDS: f32 = 1.0;
const PACER_GO_SECONDS: f32 = 3.0;

/// The pacer's grey for how long it has been stopped or moving, starting from `from`.
fn pacer_effect(paused: bool, from: f32, since: f32) -> f32 {
    use std::f32::consts::FRAC_PI_2;
    if paused {
        let x = (since / PACER_STOP_SECONDS).min(1.0);
        from + (PACER_PAUSED - from) * (x * FRAC_PI_2).sin()
    } else {
        let x = (since / PACER_GO_SECONDS).min(1.0);
        from + (1.0 - from) * (1.0 - (x * FRAC_PI_2).cos())
    }
}

/// Places the pacer's grey under Endless Race's pace line on the screen.
fn follow_pacer(
    time: Res<Time>,
    game: Option<Res<crate::gameplay::Match>>,
    menu: Option<Res<crate::visuals::MenuScene>>,
    towers: Query<&crate::gameplay::TowerState>,
    lines: Query<(Entity, &crate::visuals::FinishLine, &GlobalTransform, &InheritedVisibility)>,
    strips: Query<(&ChildOf, &bevy::camera::primitives::Aabb, &GlobalTransform), With<Sprite>>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut GreyOut)>,
    // Stopped or not, the grey when that last changed, and the time since.
    mut tween: Local<(bool, f32, f32)>,
) {
    let line = game
        .filter(|g| g.mode == crate::config::ModeKind::Endless && !menu.is_some_and(|m| m.0))
        .and_then(|_| lines.iter().find(|(_, _, _, shown)| shown.get()));
    let paused = line
        .and_then(|(_, l, _, _)| towers.get(l.tower).ok())
        .is_some_and(|s| s.pacer_pause > 0.0);
    let (was, from, since) = &mut *tween;
    let now = pacer_effect(*was, *from, *since);
    if paused != *was {
        (*was, *from, *since) = (paused, now, 0.0);
    } else {
        *since += time.delta_secs();
    }
    let effect = pacer_effect(*was, *from, *since);
    for (camera, at, mut grey) in &mut cameras {
        let place = line
            .and_then(|(entity, _, line, _)| {
                let size = camera.logical_viewport_size()?;
                // The original draws the line after its grab, so the grey starts under the
                // line's picture, not at its middle.
                let bottom = strips
                    .iter()
                    .filter(|(of, ..)| of.parent() == entity)
                    .map(|(_, aabb, at)| {
                        at.transform_point(Vec3::from(aabb.center - aabb.half_extents)).y
                    })
                    .fold(line.translation().y, f32::min);
                let y = camera
                    .world_to_viewport(at, line.translation().with_y(bottom))
                    .ok()?
                    .y;
                Some(y / size.y)
            })
            .unwrap_or(2.0);
        let (params, pacer) = (Vec4::new(grey.params.x, place, effect, 0.0), PACER_TINT);
        if grey.params != params || grey.pacer != pacer {
            grey.params = params;
            grey.pacer = pacer;
        }
    }
}

fn follow_grey(
    mut commands: Commands,
    time: Res<Time>,
    target: Res<GreyTarget>,
    mut cameras: Query<(Entity, Option<&mut GreyOut>), With<crate::visuals::MainCamera>>,
) {
    for (camera, grey) in &mut cameras {
        match grey {
            Some(mut grey) => {
                let step = time.delta_secs() / FADE_SECONDS;
                let now = grey.params.x;
                let next = now + (target.0 - now).clamp(-step, step);
                if next != now {
                    grey.params.x = next;
                }
            }
            None => {
                commands.entity(camera).insert(GreyOut::default());
            }
        }
    }
}

pub struct PostPlugin;

impl Plugin for PostPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GreyTarget>()
            .add_systems(Update, (follow_grey, follow_pacer, follow_lane_shake.after(crate::visuals::FramedCamera)));
        let Some(mut images) = app.world_mut().get_resource_mut::<Assets<Image>>() else {
            return;
        };
        let grab = images.add(Image::new_target_texture(
            1,
            1,
            bevy::render::render_resource::TextureFormat::Rgba8UnormSrgb,
            None,
        ));
        app.insert_resource(GrabTarget(grab))
            .add_systems(PostUpdate, follow_grab.before(bevy::camera::CameraUpdateSystems));
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let _ = shaders.insert(
                GREY_SHADER.id(),
                Shader::from_wgsl(GREY_WGSL, "grey_out.wgsl"),
            );
            let _ = shaders.insert(LANE_SHADER.id(), Shader::from_wgsl(LANE_WGSL, "beam_lane.wgsl"));
            let _ = shaders.insert(HAZE_SHADER.id(), Shader::from_wgsl(HAZE_WGSL, "heat_haze.wgsl"));
            let _ = shaders.insert(
                GUIDE_SHADER.id(),
                Shader::from_wgsl(GUIDE_WGSL, "guide_overlay.wgsl"),
            );
            let _ = shaders.insert(SPOT_SHADER.id(), Shader::from_wgsl(SPOT_WGSL, "spot_overlay.wgsl"));
            app.add_plugins((
                FullscreenMaterialPlugin::<GreyOut>::default(),
                Material2dPlugin::<GuideOverlay>::default(),
                Material2dPlugin::<HeatHaze>::default(),
                Material2dPlugin::<BeamLane>::default(),
                Material2dPlugin::<SpotOverlay>::default(),
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacer_grey_eases_down_when_stopped_and_back_when_going() {
        assert_eq!(pacer_effect(false, 1.0, 0.0), 1.0);
        assert!((pacer_effect(true, 1.0, 1.0) - PACER_PAUSED).abs() < 1e-6);
        let mid = pacer_effect(false, PACER_PAUSED, 1.5);
        assert!(mid > PACER_PAUSED && mid < 1.0);
        assert!((pacer_effect(false, PACER_PAUSED, 3.0) - 1.0).abs() < 1e-6);
    }
}
