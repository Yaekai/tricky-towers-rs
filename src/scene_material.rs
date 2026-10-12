//! The Race scene's water materials, written from our notes on the original's shaders
//! (docs/research/race-scene.md): islands cut off at a rippling waterline with foam under it
//! (Custom/WaterIslands), and the foreground water's rippling top (Custom/FlagWave). The WGSL
//! is ours, kept here as strings so it ships inside the game.

use bevy::asset::uuid_handle;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

const ISLAND_SHADER: Handle<Shader> = uuid_handle!("7a3e1c55-9b2d-4f60-8e14-2c6b0d9f4a71");

const ISLAND_WGSL: &str = r#"// An island in the water (docs/research/race-scene.md), on stored (sRGB) values like the
// original. The waterline sits `offset` up the picture plus the bob, rippling by `amplitude`;
// just above it the picture whitens, under it a white foam fades out, and below that nothing.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct IslandWave {
    // x: amplitude, y: waterline height (share of the picture), z: bob (share), w: unused.
    params: vec4<f32>,
    // The layer's tint, linear like a sprite's colour.
    tint: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: IslandWave;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var picture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var picture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let x = mesh.uv.x;
    let y = 1.0 - mesh.uv.y;
    let ripple = sin(5.0 * x - 0.5 * t) * sin(8.0 * x + t) * sin(3.0 * x + t);
    // How far under the waterline this point is (negative above it).
    let d = 0.5 * material.params.x * ripple + material.params.z + material.params.y - y;
    let tex = textureSample(picture, picture_sampler, mesh.uv);
    let colour = linear_to_srgb(tex.rgb) * linear_to_srgb(material.tint.rgb);
    let alpha = tex.a * material.tint.a;
    if (d <= 0.0) {
        var c = colour;
        if (d >= -0.008) {
            let k = 0.8 + 112.5 * d;
            c = c * (1.0 - k) + vec3(k);
        }
        return vec4(srgb_to_linear(clamp(c, vec3(0.0), vec3(1.0))), alpha);
    }
    if (d <= 0.015) {
        return vec4(vec3(1.0), (0.8 - 46.666668 * d) * alpha);
    }
    return vec4(0.0);
}
"#;

/// An island's look: its picture and its waterline.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct IslandWave {
    #[uniform(0)]
    pub look: IslandLook,
    #[texture(1)]
    #[sampler(2)]
    pub picture: Handle<Image>,
}

/// x amplitude, y waterline height and z bob (both shares of the picture's height); the tint.
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct IslandLook {
    pub params: Vec4,
    pub tint: Vec4,
}

impl Material2d for IslandWave {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(ISLAND_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const WATER_SHADER: Handle<Shader> = uuid_handle!("c41d8e2a-5f37-4b9c-a0e6-83b5d2f17c49");

const WATER_WGSL: &str = r#"// The foreground water (docs/research/race-scene.md): the picture is read from higher or
// lower by three sine waves along it, so its top edge ripples. On stored (sRGB) values.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct FlagWave {
    // x: amplitude, y: speed, z: offset up, w: how many of the original's 80-unit spans
    // this quad covers.
    params: vec4<f32>,
    tint: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: FlagWave;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var picture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var picture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let w = 2.0 * globals.time * material.params.y;
    let x = 0.5 + (mesh.uv.x - 0.5) * material.params.w;
    let wave = sin(9.0 * x + w) * sin(30.0 * x + 0.1 * w) * sin(5.0 * x + 0.5 * w);
    let y = clamp(wave * material.params.x + material.params.z + 1.0 - mesh.uv.y, 0.0, 1.0);
    let tex = textureSample(picture, picture_sampler, vec2(mesh.uv.x, 1.0 - y));
    let colour = linear_to_srgb(tex.rgb) * linear_to_srgb(material.tint.rgb);
    return vec4(srgb_to_linear(colour), tex.a * material.tint.a);
}
"#;

/// The foreground water's look: its picture and its ripple.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct FlagWave {
    #[uniform(0)]
    pub look: WaterLook,
    #[texture(1)]
    #[sampler(2)]
    pub picture: Handle<Image>,
}

/// x amplitude, y speed, z offset up, w spans of 80 units covered; the tint.
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct WaterLook {
    pub params: Vec4,
    pub tint: Vec4,
}

impl Material2d for FlagWave {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(WATER_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const FLAG_SHADER: Handle<Shader> = uuid_handle!("2b8f61d4-7e09-4c3a-95b2-d04e6a1c8f37");

const FLAG_WGSL: &str = r#"// An Endless Race checkpoint flag (docs/research/race-scene.md): read from higher or lower
// by a sine along it, more towards its free end (the original's Custom/CheckpointFlag).
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}

struct CheckpointFlag {
    // x: amplitude, y: frequency, z: speed.
    params: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: CheckpointFlag;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var picture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var picture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let x = mesh.uv.x;
    let wave = sin((2.0 * globals.time * material.params.z + x) * material.params.y);
    let y = (1.0 - mesh.uv.y) + wave * material.params.x * x;
    return textureSample(picture, picture_sampler, vec2(x, 1.0 - y));
}
"#;

/// A checkpoint flag's look: its picture and its wave.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct CheckpointFlag {
    #[uniform(0)]
    pub params: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub picture: Handle<Image>,
}

impl Material2d for CheckpointFlag {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(FLAG_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const CLOUDS_SHADER: Handle<Shader> = uuid_handle!("9b5e2d71-4c83-4a06-b1f9-7e2c58d03a16");

const CLOUDS_WGSL: &str = r#"// Endless Race's perspective clouds (PerspectiveCloudsBackground, Custom/EndlessRaceClouds):
// two tilted cloud planes seen through a 60 degree camera 10 units back, rising from the
// horizon to over the view. Each screen point's ray is met with each plane; the cloud picture
// there is cut by two scrolling masks (blue of the first, red of the second) and fades out
// over the far half.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::view,
}

struct PerspectiveClouds {
    // x: the picture's scroll, y: the first mask's, z: the second's, w: alpha.
    scroll: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: PerspectiveClouds;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var near_cloud: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var near_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var far_cloud: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var far_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var mask_1: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var mask_1_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(7) var mask_2: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(8) var mask_2_sampler: sampler;

// A plane's cloud at this ray: its (u, v) on the plane, or a negative u when the ray
// misses it. The planes are Unity's 10 x 10 plane scaled 2 x 2.35, turned -158.9 degrees
// about x, 11.52 ahead and 0.04 up; u runs to -x and v away from the camera.
fn on_plane(dir: vec3<f32>, x: f32) -> vec2<f32> {
    let centre = vec3(x, 0.04, 11.52);
    let along = vec3(0.0, 0.360, -0.933);
    let normal = vec3(0.0, -0.933, -0.360);
    let t = dot(centre, normal) / dot(dir, normal);
    if t <= 0.0 {
        return vec2(-1.0);
    }
    let d = dir * t - centre;
    let a = d.x / 10.0;
    let b = dot(d, along) / 11.75;
    if abs(a) > 1.0 || abs(b) > 1.0 {
        return vec2(-1.0);
    }
    return vec2((1.0 - a) * 0.5, (1.0 - b) * 0.5);
}

fn cloud(main: vec4<f32>, uv: vec2<f32>) -> vec4<f32> {
    let m1 = textureSample(mask_1, mask_1_sampler, fract(uv + vec2(0.0, material.scroll.y))).b;
    let m2 = textureSample(mask_2, mask_2_sampler, fract(uv + vec2(0.0, material.scroll.z))).r;
    var a = (m1 + m2) * main.a;
    if uv.y > 0.5 {
        a = a * 2.0 * (1.0 - uv.y);
    }
    return vec4(main.rgb, clamp(a, 0.0, 1.0) * material.scroll.w);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = (mesh.position.xy - view.viewport.xy) / view.viewport.zw;
    let k = tan(radians(30.0));
    let aspect = view.viewport.z / view.viewport.w;
    let dir = vec3((uv.x * 2.0 - 1.0) * k * aspect, (1.0 - uv.y * 2.0) * k, 1.0);
    // The farther plane (CloudPlane02, x 4.4) first, the nearer one over it.
    var out = vec4(0.0);
    let far = on_plane(dir, 4.4);
    if far.x >= 0.0 {
        let main = textureSample(far_cloud, far_sampler, fract(far + vec2(0.0, material.scroll.x)));
        out = cloud(main, far);
    }
    let near = on_plane(dir, -3.3);
    if near.x >= 0.0 {
        let main = textureSample(near_cloud, near_sampler, fract(near + vec2(0.0, material.scroll.x)));
        let c = cloud(main, near);
        let a = c.a + out.a * (1.0 - c.a);
        out = vec4((c.rgb * c.a + out.rgb * out.a * (1.0 - c.a)) / max(a, 0.0001), a);
    }
    return out;
}
"#;

/// Endless Race's perspective clouds: CloudPlane01 draws gph_cloudLargeRaceEndless_02,
/// CloudPlane02 _01, both cut by the same two masks.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct PerspectiveClouds {
    #[uniform(0)]
    pub scroll: Vec4,
    #[texture(1)]
    #[sampler(2)]
    pub near_cloud: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub far_cloud: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub mask_1: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    pub mask_2: Handle<Image>,
}

impl Material2d for PerspectiveClouds {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(CLOUDS_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// The perspective clouds' screen quad, a child of the main camera.
#[derive(Component)]
struct CloudQuad;

/// Their depth: over the Endless sky gradient, under the rest of its scene
/// (cameras 1, 2, 3: gradient, these clouds, the islands and water).
const CLOUDS_Z: f32 = -910.0;

/// PerspectiveCloudsLayer: each frame the picture scrolls by 0.06 to 0.12, the masks by
/// 0.003 to 0.06 and 0.005 to 0.072 of their height a second (random each frame; their
/// middles here), times a speed that eases (by the frame time) towards the pacer's
/// speed / 0.5.
const CLOUD_SCROLL: Vec3 = Vec3::new(0.09, 0.0315, 0.0385);

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn run_perspective_clouds(
    mut commands: Commands,
    time: Res<Time>,
    art: Option<Res<crate::art::Art>>,
    game: Option<Res<crate::gameplay::Match>>,
    menu: Option<Res<crate::visuals::MenuScene>>,
    look: Option<Res<crate::visuals::MenuLook>>,
    towers: Query<&crate::gameplay::TowerState>,
    view: Res<crate::visuals::ViewSize>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<PerspectiveClouds>>,
    camera: Query<(Entity, &Projection, &Transform), (With<crate::visuals::MainCamera>, Without<CloudQuad>)>,
    mut quad: Query<(&MeshMaterial2d<PerspectiveClouds>, &mut Transform, &mut Visibility), With<CloudQuad>>,
    mut speed: Local<Option<f32>>,
) {
    use crate::config::ModeKind;
    let Ok((eye, projection, at)) = camera.single() else {
        return;
    };
    let in_menus = menu.is_some_and(|m| m.0);
    let shown = if in_menus {
        look.is_some_and(|l| matches!(l.0, Some((ModeKind::Endless, _))))
    } else {
        game.as_ref().is_some_and(|g| g.mode == ModeKind::Endless)
    };
    let Ok((material, mut transform, mut visibility)) = quad.single_mut() else {
        let Some(art) = art else { return };
        let (Some(near_cloud), Some(far_cloud), Some(mask_1), Some(mask_2)) = (
            art.image("endless_cloud_2"),
            art.image("endless_cloud_1"),
            art.image("endless_cloud_mask_1"),
            art.image("endless_cloud_mask_2"),
        ) else {
            return;
        };
        commands.entity(eye).with_child((
            CloudQuad,
            Mesh2d(meshes.add(Rectangle::new(1.0, 1.0))),
            MeshMaterial2d(materials.add(PerspectiveClouds {
                scroll: Vec4::new(0.0, 0.0, 0.0, 1.0),
                near_cloud,
                far_cloud,
                mask_1,
                mask_2,
            })),
            Transform::from_xyz(0.0, 0.0, CLOUDS_Z),
            Visibility::Hidden,
            bevy::camera::visibility::RenderLayers::from_layers(&[0, 1]),
        ));
        return;
    };
    visibility.set_if_neq(if shown { Visibility::Inherited } else { Visibility::Hidden });
    if !shown {
        *speed = None;
        return;
    }
    let scale = match projection {
        Projection::Orthographic(ortho) => ortho.scale,
        _ => 1.0,
    };
    transform.translation.z = CLOUDS_Z - at.translation.z;
    transform.scale = (view.0 * scale).extend(1.0);
    let dt = time.delta_secs();
    let s = speed.get_or_insert(1.0);
    let pacer = (!in_menus).then(|| towers.iter().next().map(|t| t.pacer_speed)).flatten();
    if let Some(pacer) = pacer {
        *s += (pacer / 0.5 - *s) * dt.min(1.0);
    }
    if let Some(mut m) = materials.get_mut(&material.0) {
        let step = CLOUD_SCROLL * dt * *s;
        m.scroll.x = (m.scroll.x + step.x).fract();
        m.scroll.y = (m.scroll.y + step.y).fract();
        m.scroll.z = (m.scroll.z + step.z).fract();
    }
}

const SUN_SHADER: Handle<Shader> = uuid_handle!("3c7d9e15-82a4-4b6f-9d03-5e1f7a2c8b64");

const SUN_WGSL: &str = r#"// Endless Puzzle's sun (DesertEndlessSun, World/Background/Puzzle/Eclipse): a disc of
// radius 0.3 fading out by 0.57 of the quad, with a moon of radius 0.2 (fading out by 0.38)
// sliding in from the left to x = 0.67 x amount - 0.4. How much of the sun still shows picks
// a point on the eclipse gradient: its blue mixes the gradient colour towards the sun colour,
// its alpha is the alpha.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct EndlessSun {
    // x: the eclipse's amount, 0 to 1.
    params: vec4<f32>,
    // The colours (sRGB): the gradient's, which turns orange, and the sun's.
    gradient: vec4<f32>,
    sun: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: EndlessSun;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var ramp: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var ramp_sampler: sampler;

fn inside(d: f32, start: f32, k: f32) -> f32 {
    let s = clamp((d - start) * k, 0.0, 1.0);
    return 1.0 - s * s * (3.0 - 2.0 * s);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = mesh.uv;
    let a = material.params.x;
    let moon = inside(length(vec2(uv.x - (0.67 * a - 0.4), uv.y - 0.5)), 0.2, 5.5555558);
    let sun = inside(length(uv - vec2(0.5)), 0.3, 3.7037032);
    let shows = clamp(sun - moon, 0.0, 1.0);
    let t = textureSample(ramp, ramp_sampler, vec2(1.0 - shows, 0.5));
    let b = linear_to_srgb(t.rgb).b;
    let colour = material.gradient.rgb + b * (material.sun.rgb - material.gradient.rgb);
    return vec4(srgb_to_linear(colour), t.a);
}
"#;

/// Endless Puzzle's sun: the eclipse's amount and colours, and the eclipse gradient.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct EndlessSun {
    #[uniform(0)]
    pub look: EndlessSunLook,
    #[texture(1)]
    #[sampler(2)]
    pub ramp: Handle<Image>,
}

#[derive(bevy::render::render_resource::ShaderType, Clone, Copy, Debug)]
pub struct EndlessSunLook {
    pub params: Vec4,
    pub gradient: Vec4,
    pub sun: Vec4,
}

impl Material2d for EndlessSun {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(SUN_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// The Eclipse material's colours: the gradient starts pale gold and the eclipse turns it
/// to #FF641F (DesertBackgroundEndlessSun.AnimateIn); the sun stays white.
pub const SUN_GRADIENT_FROM: Vec4 = Vec4::new(1.0, 0.8833671, 0.63235295, 1.0);
pub const SUN_GRADIENT_TO: Vec4 = Vec4::new(1.0, 100.0 / 255.0, 31.0 / 255.0, 1.0);

impl EndlessSunLook {
    /// The look `amount` (0 to 1, already eased) into the eclipse.
    pub fn at(amount: f32) -> Self {
        Self {
            params: Vec4::new(amount, 0.0, 0.0, 0.0),
            gradient: SUN_GRADIENT_FROM.lerp(SUN_GRADIENT_TO, amount),
            sun: Vec4::ONE,
        }
    }
}

pub struct SceneMaterialPlugin;

impl Plugin for SceneMaterialPlugin {
    fn build(&self, app: &mut App) {
        let _ = app.world_mut().resource_mut::<Assets<Shader>>().insert(
            ISLAND_SHADER.id(),
            Shader::from_wgsl(ISLAND_WGSL, "island_wave.wgsl"),
        );
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(WATER_SHADER.id(), Shader::from_wgsl(WATER_WGSL, "flag_wave.wgsl"));
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(FLAG_SHADER.id(), Shader::from_wgsl(FLAG_WGSL, "checkpoint_flag.wgsl"));
        app.add_plugins((
            Material2dPlugin::<IslandWave>::default(),
            Material2dPlugin::<FlagWave>::default(),
            Material2dPlugin::<CheckpointFlag>::default(),
            Material2dPlugin::<PerspectiveClouds>::default(),
            Material2dPlugin::<EndlessSun>::default(),
        ));
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(SUN_SHADER.id(), Shader::from_wgsl(SUN_WGSL, "endless_sun.wgsl"));
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(CLOUDS_SHADER.id(), Shader::from_wgsl(CLOUDS_WGSL, "perspective_clouds.wgsl"));
        app.add_systems(Update, run_perspective_clouds);
    }
}
