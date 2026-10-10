//! The brick packs' own looks as a 2D material, written from our notes on the original's
//! brick shaders (docs/research/galaxy.md, diamond.md). Galaxy and Diamond so far; the WGSL is ours, kept here as a
//! string so it ships inside the game.

use bevy::asset::uuid_handle;
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

const GALAXY_SHADER: Handle<Shader> = uuid_handle!("5b0c8f8e-2a4e-4c1e-9a61-3f1b7a2d9c04");

const GALAXY_WGSL: &str = r#"// A Galaxy brick, drawn the way the original's brick shader does it (docs/research/galaxy.md),
// written from those notes. The maths runs on stored (sRGB) values like the original, which
// blends in gamma space, and the result goes back to linear light for Bevy.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct GalaxyBrick {
    // rgb: colour, a: strength.
    highlight: vec4<f32>,
    shadow: vec4<f32>,
    // rgb: the outline colour.
    border: vec4<f32>,
    // x: hue turn (radians), y: the brick's rotation (radians), z: pixels per cell, w: flash.
    params: vec4<f32>,
    // The 49 stars of the original's Galaxy scene: x, y in scene units from its centre,
    // z scale, w turn (radians).
    stars: array<vec4<f32>, 49>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: GalaxyBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sky: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sky_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var star: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var star_sampler: sampler;

// The original's hue turn, a YIQ-style rotation.
fn hue(c: vec3<f32>, angle: f32) -> vec3<f32> {
    let s = sin(angle);
    let k = cos(angle);
    return vec3(
        (0.299 + 0.701 * k + 0.168 * s) * c.r + (0.587 - 0.587 * k + 0.330 * s) * c.g + (0.114 - 0.114 * k - 0.497 * s) * c.b,
        (0.299 - 0.299 * k - 0.328 * s) * c.r + (0.587 + 0.413 * k + 0.035 * s) * c.g + (0.114 - 0.114 * k + 0.292 * s) * c.b,
        (0.299 - 0.300 * k + 1.25 * s) * c.r + (0.587 - 0.588 * k - 1.05 * s) * c.g + (0.114 + 0.886 * k - 0.203 * s) * c.b,
    );
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let map_linear = textureSample(bevel, bevel_sampler, mesh.uv);
    let map = vec4(linear_to_srgb(map_linear.rgb), map_linear.a);

    // The sky: world position at a tenth (one repeat every 10 cells), plus the scene's scroll
    // and the shader's sway, in Unity's up-is-positive UVs; our picture's rows run down.
    let world = mesh.world_position.xy / material.params.z;
    let u = world.x * 0.1 + 0.05 * t + 0.1 * cos(0.3 * t);
    let v = world.y * 0.1 - 0.07 * t + 0.2 * sin(0.1 * t);
    let sky_linear = textureSample(sky, sky_sampler, vec2(u, -v)).rgb;
    var sky_colour = linear_to_srgb(sky_linear);

    // The stars are in the sky picture the original renders (one 10-unit scene), so they go in
    // before the hue turn and the light. Each star picture is 18 px at 26 to the unit.
    let field = fract(vec2(u, v)) * 10.0 - 5.0;
    for (var i = 0u; i < 49u; i = i + 1u) {
        let s = material.stars[i];
        var q = field - s.xy;
        q = q - 10.0 * round(q / 10.0);
        let k = cos(-s.w);
        let sn = sin(-s.w);
        let r = vec2(k * q.x - sn * q.y, sn * q.x + k * q.y);
        let half = 0.5 * (18.0 / 26.0) * s.z;
        if (abs(r.x) < half && abs(r.y) < half) {
            let st = r / (2.0 * half) + 0.5;
            let tex = textureSampleLevel(star, star_sampler, vec2(st.x, 1.0 - st.y), 0.0);
            sky_colour = mix(sky_colour, linear_to_srgb(tex.rgb), tex.a);
        }
    }
    var colour = clamp(hue(sky_colour, material.params.x), vec3(0.0), vec3(1.0));

    // The bevel's slope, turned with the brick into the world, lit from up and to the left.
    let n = map.rg * 2.0 - 1.0;
    let r = material.params.y;
    let world_n = vec2(cos(r) * n.x - sin(r) * n.y, sin(r) * n.x + cos(r) * n.y);
    let d = dot(world_n, vec2(-0.44721359, 0.89442718));
    colour = mix(colour, material.highlight.rgb, clamp(d, 0.0, 1.0) * material.highlight.a);
    colour = mix(colour, material.shadow.rgb, clamp(-d, 0.0, 1.0) * material.shadow.a);

    // The outline (the map's blue, and its outside) in the border colour.
    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, material.border.rgb, outline);
    colour = colour + material.params.w * (1.0 - colour);

    let alpha = max(map.a, map.b);
    return vec4(srgb_to_linear(colour), alpha);
}
"#;

/// A Galaxy brick's look: its bevel map, the shared sky, and the shape's colours.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct GalaxyBrick {
    #[uniform(0)]
    pub look: GalaxyLook,
    #[texture(1)]
    #[sampler(2)]
    pub bevel: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub sky: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    pub star: Handle<Image>,
}

/// The shape's colours (stored sRGB values, as the original's material has them), the
/// changing values (x hue turn in radians, y the brick's rotation, z pixels per cell, w flash),
/// and the shared sky's 49 stars (x, y position, z scale, w turn).
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct GalaxyLook {
    pub highlight: Vec4,
    pub shadow: Vec4,
    pub border: Vec4,
    pub params: Vec4,
    pub stars: [Vec4; 49],
}

impl Material2d for GalaxyBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GALAXY_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const DIAMOND_SHADER: Handle<Shader> = uuid_handle!("8d2f4a61-7c3b-4e5a-b1d2-6e9f0a3c7b15");

const DIAMOND_WGSL: &str = r#"// A Diamond brick, drawn the way the original's brick shader does it (docs/research/diamond.md),
// written from those notes, on stored (sRGB) values like the original.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct DiamondBrick {
    // rgb: colour, a: strength.
    gloss1: vec4<f32>,
    gloss2: vec4<f32>,
    border: vec4<f32>,
    // x: sweep angle (radians), y: sweep strength, z: the brick's rotation, w: flash.
    params: vec4<f32>,
    // x, y: the bevel map's size in units (pixels / 65).
    size: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: DiamondBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pattern: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var pattern_sampler: sampler;

// Photoshop's overlay: darks multiplied by twice the colour, lights screened.
fn overlay(a: vec3<f32>, b: vec3<f32>) -> vec3<f32> {
    let low = 2.0 * a * b;
    let high = 1.0 - 2.0 * (1.0 - a) * (1.0 - b);
    return select(high, low, a <= vec3(0.5));
}

// 1 where the band is lit: frac(x) at or under `lit`.
fn band(x: f32, lit: f32) -> f32 {
    return select(0.0, 1.0, fract(x) <= lit);
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let map_linear = textureSample(bevel, bevel_sampler, mesh.uv);
    let map = vec4(linear_to_srgb(map_linear.rgb), map_linear.a);
    let pat_linear = textureSample(pattern, pattern_sampler, mesh.uv);
    var colour = linear_to_srgb(pat_linear.rgb);

    // Diagonal position across the brick in units, in Unity's up-is-positive UVs.
    let s = mesh.uv.x * material.size.x + (1.0 - mesh.uv.y) * material.size.y;
    // Wide bands of the first gloss, two sets of thin bands of the second, all moving.
    let wide = band((s + 0.2 * t - 0.2) * (2.0 / 3.0), 0.4);
    let thin = band((s + 0.3 * t + 0.2) * (5.0 / 12.0), 1.0 / 12.0) + band((s + 0.4 * t + 1.4) * (5.0 / 12.0), 0.125);
    colour = mix(colour, overlay(colour, material.gloss1.rgb), wide * material.gloss1.a);
    colour = mix(colour, overlay(colour, material.gloss2.rgb), thin * material.gloss2.a);

    // The sweep: white on the bevel faces facing the turning angle, by the sweep's strength.
    let n = map.rg * 2.0 - 1.0;
    let r = material.params.z;
    let world_n = vec2(cos(r) * n.x - sin(r) * n.y, sin(r) * n.x + cos(r) * n.y);
    let a = material.params.x;
    let shine = clamp(dot(vec2(cos(a), sin(a)), world_n), 0.0, 1.0) * material.params.y;
    colour = mix(colour, vec3(1.0), shine);

    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, material.border.rgb, outline);
    colour = colour + material.params.w * (1.0 - colour);
    let alpha = max(pat_linear.a, map.b);
    return vec4(srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0))), alpha);
}
"#;

/// A Diamond brick's look: its bevel map, its own pattern, and the shape's colours.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct DiamondBrick {
    #[uniform(0)]
    pub look: DiamondLook,
    #[texture(1)]
    #[sampler(2)]
    pub bevel: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub pattern: Handle<Image>,
}

/// The shape's gloss and border colours (stored sRGB, as the original's material has them), the
/// changing values (x sweep angle, y sweep strength, z rotation, w flash) and the bevel map's
/// size in units (x, y).
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct DiamondLook {
    pub gloss1: Vec4,
    pub gloss2: Vec4,
    pub border: Vec4,
    pub params: Vec4,
    pub size: Vec4,
}

impl Material2d for DiamondBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(DIAMOND_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

pub struct BrickMaterialPlugin;

impl Plugin for BrickMaterialPlugin {
    fn build(&self, app: &mut App) {
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(GALAXY_SHADER.id(), Shader::from_wgsl(GALAXY_WGSL, "galaxy_brick.wgsl"));
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(DIAMOND_SHADER.id(), Shader::from_wgsl(DIAMOND_WGSL, "diamond_brick.wgsl"));
        app.add_plugins((
            Material2dPlugin::<GalaxyBrick>::default(),
            Material2dPlugin::<DiamondBrick>::default(),
        ));
    }
}
