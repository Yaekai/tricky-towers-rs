//! The bricks' looks as 2D materials, written from our notes on the original's brick shaders
//! (docs/research/brick-lighting.md, galaxy.md, diamond.md): the shared lit brick (default,
//! Christmas, Halloween, Candy, Spring and Hologram's base), Galaxy and Diamond. The WGSL is
//! ours, kept here as strings so it ships inside the game.
//!
//! Each brick quad hangs under an empty `Sprite` (no size, so it draws nothing; a sprite and a
//! mesh on one entity hide other things): the brick effects tint and move it like any brick
//! sprite (losers turning pale, Undo's zap, Bubble, the Hologram glitch, the Halloween fade,
//! Large) and `follow_bricks` hands that colour to the material along with the brick's turn,
//! which keeps the light fixed in the world.

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
    // The effects' tint, multiplied in like a sprite's colour.
    tint: vec4<f32>,
    // The water: x its height (world pixels), y 1 when on, z pixels per cell; its line's and
    // its own colour.
    water: vec4<f32>,
    water_line: vec4<f32>,
    water_colour: vec4<f32>,

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

// Under the water in Race (the original's WATER_ON): the water line's colour for the first
// 0.12 of a cell, then the water's, fading out over 2 cells (stored sRGB colours).
fn under_water(world_y: f32, colour: ptr<function, vec3<f32>>) -> f32 {
    if (material.water.y < 0.5) {
        return 1.0;
    }
    let depth = (material.water.x - world_y) / material.water.z;
    let w = clamp(1.0 - 0.5 * depth, 0.0, 1.0);
    if (w < 1.0) {
        *colour = material.water_line.rgb;
    }
    if (w < 0.94) {
        *colour = material.water_colour.rgb;
    }
    return w;
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
    let wet = under_water(mesh.world_position.y, &colour);

    let alpha = max(map.a, map.b) * wet;
    return vec4(srgb_to_linear(colour) * material.tint.rgb, alpha * material.tint.a);
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
    pub tint: Vec4,
    pub water: Vec4,
    pub water_line: Vec4,
    pub water_colour: Vec4,
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
    // The effects' tint, multiplied in like a sprite's colour.
    tint: vec4<f32>,
    // The water: x its height (world pixels), y 1 when on, z pixels per cell; its line's and
    // its own colour.
    water: vec4<f32>,
    water_line: vec4<f32>,
    water_colour: vec4<f32>,

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

// Under the water in Race (the original's WATER_ON): the water line's colour for the first
// 0.12 of a cell, then the water's, fading out over 2 cells (stored sRGB colours).
fn under_water(world_y: f32, colour: ptr<function, vec3<f32>>) -> f32 {
    if (material.water.y < 0.5) {
        return 1.0;
    }
    let depth = (material.water.x - world_y) / material.water.z;
    let w = clamp(1.0 - 0.5 * depth, 0.0, 1.0);
    if (w < 1.0) {
        *colour = material.water_line.rgb;
    }
    if (w < 0.94) {
        *colour = material.water_colour.rgb;
    }
    return w;
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
    let wet = under_water(mesh.world_position.y, &colour);
    let alpha = max(pat_linear.a, map.b) * wet;
    return vec4(srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0))) * material.tint.rgb, alpha * material.tint.a);
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
    pub tint: Vec4,
    pub water: Vec4,
    pub water_line: Vec4,
    pub water_colour: Vec4,
}

impl Material2d for DiamondBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(DIAMOND_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const LIT_SHADER: Handle<Shader> = uuid_handle!("2e7c9b14-6d3a-4f8e-a5c1-9b0d4e7f2a63");

const LIT_WGSL: &str = r#"// A lit brick, the way the original's World/Bricks shaders light it (docs/research/
// brick-lighting.md), written from those notes, on stored (sRGB) values like the original.
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct LitBrick {
    // rgb: colour, a: strength.
    highlight: vec4<f32>,
    shadow: vec4<f32>,
    // rgb: the outline colour.
    border: vec4<f32>,
    // x: the brick's rotation (radians).
    params: vec4<f32>,
    // The effects' tint, multiplied in like a sprite's colour.
    tint: vec4<f32>,
    // The water: x its height (world pixels), y 1 when on, z pixels per cell; its line's and
    // its own colour.
    water: vec4<f32>,
    water_line: vec4<f32>,
    water_colour: vec4<f32>,

};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: LitBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pattern: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var pattern_sampler: sampler;

// Under the water in Race (the original's WATER_ON): the water line's colour for the first
// 0.12 of a cell, then the water's, fading out over 2 cells (stored sRGB colours).
fn under_water(world_y: f32, colour: ptr<function, vec3<f32>>) -> f32 {
    if (material.water.y < 0.5) {
        return 1.0;
    }
    let depth = (material.water.x - world_y) / material.water.z;
    let w = clamp(1.0 - 0.5 * depth, 0.0, 1.0);
    if (w < 1.0) {
        *colour = material.water_line.rgb;
    }
    if (w < 0.94) {
        *colour = material.water_colour.rgb;
    }
    return w;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let map_linear = textureSample(bevel, bevel_sampler, mesh.uv);
    let map = vec4(linear_to_srgb(map_linear.rgb), map_linear.a);
    let pat_linear = textureSample(pattern, pattern_sampler, mesh.uv);
    var colour = linear_to_srgb(pat_linear.rgb);

    // The bevel's slope, turned with the brick into the world, lit from up and to the left.
    let n = map.rg * 2.0 - 1.0;
    let r = material.params.x;
    let world_n = vec2(cos(r) * n.x - sin(r) * n.y, sin(r) * n.x + cos(r) * n.y);
    let d = dot(world_n, vec2(-0.44721359, 0.89442718));
    colour = mix(colour, material.highlight.rgb, clamp(d, 0.0, 1.0) * material.highlight.a);
    colour = mix(colour, material.shadow.rgb, clamp(-d, 0.0, 1.0) * material.shadow.a);

    // The outline and cracks (the map's blue, and its outside) in the line colour.
    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, material.border.rgb, outline);
    let wet = under_water(mesh.world_position.y, &colour);

    let alpha = max(pat_linear.a, map.b) * wet;
    let out = srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0)));
    return vec4(out * material.tint.rgb, alpha * material.tint.a);
}
"#;

/// A lit brick's look: its bevel map, its unlit pattern, and its light colours.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct LitBrick {
    #[uniform(0)]
    pub look: LitLook,
    #[texture(1)]
    #[sampler(2)]
    pub bevel: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub pattern: Handle<Image>,
}

/// The light colours and outline (stored sRGB, as the original's material has them), x the
/// brick's rotation, and the effects' tint (linear, like a sprite's colour).
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct LitLook {
    pub highlight: Vec4,
    pub shadow: Vec4,
    pub border: Vec4,
    pub params: Vec4,
    pub tint: Vec4,
    pub water: Vec4,
    pub water_line: Vec4,
    pub water_colour: Vec4,
}

impl Material2d for LitBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(LIT_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

const HOLO_SHADER: Handle<Shader> = uuid_handle!("e5b27d90-3a4c-4f1e-9d86-1c7a5f0b3e28");

const HOLO_WGSL: &str = r#"// A Hologram brick, the way the original's World/Bricks/Hologram shader draws it
// (docs/research/hologram.md), written from those notes, on stored (sRGB) values.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::globals,
}
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct HologramBrick {
    highlight: vec4<f32>,
    shadow: vec4<f32>,
    // x: the brick's rotation, y: the glitch's distortion, z: pixels per cell, w: flash.
    params: vec4<f32>,
    tint: vec4<f32>,
    // The water: x its height (world pixels), y 1 when on, z pixels per cell; its line's and
    // its own colour.
    water: vec4<f32>,
    water_line: vec4<f32>,
    water_colour: vec4<f32>,

};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: HologramBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pattern: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var pattern_sampler: sampler;

// The shader's hash.
fn hash(n: f32) -> f32 {
    return fract(sin(n) * 18336.822);
}

// Under the water in Race (the original's WATER_ON): the water line's colour for the first
// 0.12 of a cell, then the water's, fading out over 2 cells (stored sRGB colours).
fn under_water(world_y: f32, colour: ptr<function, vec3<f32>>) -> f32 {
    if (material.water.y < 0.5) {
        return 1.0;
    }
    let depth = (material.water.x - world_y) / material.water.z;
    let w = clamp(1.0 - 0.5 * depth, 0.0, 1.0);
    if (w < 1.0) {
        *colour = material.water_line.rgb;
    }
    if (w < 0.94) {
        *colour = material.water_colour.rgb;
    }
    return w;
}

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let t = globals.time;
    let d = material.params.y;
    // Unity's up-is-positive UVs.
    let uv = vec2(mesh.uv.x, 1.0 - mesh.uv.y);
    let world_y = mesh.world_position.y / material.params.z;

    // Scan lines: towards (0.8, 1, 1) by a flickering share of frac(3 y + t).
    let k = (sin(50.0 * t) + 1.0) * 0.025 + 0.2;
    let pat = linear_to_srgb(textureSample(pattern, pattern_sampler, mesh.uv).rgb);
    var colour = mix(pat, vec3(0.8, 1.0, 1.0), k * fract(3.0 * world_y + t));

    // The glitch: a flicker ten times a second, rows torn sideways, some bands brighter.
    let step = floor(10.0 * t) * 100.0;
    let flicker = 1.0 - d * hash(step);
    let r10 = floor(10.0 * uv.y) + step;
    let r30 = floor(30.0 * uv.y) + step;
    let n10 = mix(hash(r10), hash(r10 + 1.0), fract(10.0 * uv.y));
    let n30 = mix(hash(r30), hash(r30 + 1.0), fract(30.0 * uv.y));
    let tear = floor((2.0 * n10 - 1.0) * d + 0.5) * 0.1;
    colour = colour + vec3(floor(1.5 * n30 + 0.5) / 6.0 * d);
    let at = vec2(uv.x + tear, uv.y);
    let map_linear = textureSample(bevel, bevel_sampler, vec2(at.x, 1.0 - at.y));
    let map = vec4(linear_to_srgb(map_linear.rgb), map_linear.a);

    // The bevel's slope, turned with the brick into the world, lit from up and to the left.
    let n = map.rg * 2.0 - 1.0;
    let r = material.params.x;
    let world_n = vec2(cos(r) * n.x - sin(r) * n.y, sin(r) * n.x + cos(r) * n.y);
    let l = dot(world_n, vec2(-0.44721359, 0.89442718));
    colour = mix(colour, material.highlight.rgb, clamp(l, 0.0, 1.0) * material.highlight.a);
    colour = mix(colour, material.shadow.rgb, clamp(-l, 0.0, 1.0) * material.shadow.a);

    // The outline, black at rest, (0.8, 1, 1) through a glitch.
    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, d * vec3(0.8, 1.0, 1.0), outline);
    colour = colour + material.params.w * (1.0 - colour);
    let wet = under_water(mesh.world_position.y, &colour);
    let alpha = max(flicker * map.a, map.b) * wet;
    let out = srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0)));
    return vec4(out * material.tint.rgb, alpha * material.tint.a);
}
"#;

/// A Hologram brick's look: its bevel map, the base skin's unlit pattern and its light colours.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct HologramBrick {
    #[uniform(0)]
    pub look: HoloLook,
    #[texture(1)]
    #[sampler(2)]
    pub bevel: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub pattern: Handle<Image>,
}

/// The light colours (stored sRGB), x the brick's rotation, y the glitch's distortion, z pixels
/// per cell, w flash; and the effects' tint.
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct HoloLook {
    pub highlight: Vec4,
    pub shadow: Vec4,
    pub params: Vec4,
    pub tint: Vec4,
    pub water: Vec4,
    pub water_line: Vec4,
    pub water_colour: Vec4,
}

impl Material2d for HologramBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(HOLO_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// The mesh and material of a Hologram brick quad of `size` over manifest entry `key` (the
/// base skin's brick), when it has light colours, a bevel map and its unlit pattern.
pub fn hologram_brick(
    art: &crate::art::Art,
    key: &str,
    size: Vec2,
    cell: f32,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<HologramBrick>,
) -> Option<(Handle<Mesh>, Handle<HologramBrick>)> {
    let entry = art.entries.get(key)?;
    let ((hr, hg, hb, ha), (sr, sg, sb, sa)) = entry.light.as_ref()?.get()?;
    let bevel = art.images.get(entry.bevel.as_ref()?)?.clone();
    let pattern = art.images.get(&format!("{key}_raw"))?.clone();
    let material = materials.add(HologramBrick {
        look: HoloLook {
            highlight: Vec4::new(hr, hg, hb, ha),
            shadow: Vec4::new(sr, sg, sb, sa),
            params: Vec4::new(0.0, 0.0, cell, 0.0),
            tint: Vec4::ONE,
            water: Vec4::ZERO,
            water_line: Vec4::ZERO,
            water_colour: Vec4::ZERO,
        },
        bevel,
        pattern,
    });
    Some((meshes.add(Rectangle::new(size.x, size.y)), material))
}

/// A quad drawn by one of the brick materials; its parent's `Sprite` carries the effects' tint.
#[derive(Component)]
pub struct BrickQuad;

/// Spawns under `brick` an empty sprite at `at` carrying `color` and `on_sprite`, with the quad
/// of `size` drawn by `material` (and carrying `on_quad`) under it.
pub fn spawn_quad<M: Material2d>(
    brick: &mut EntityCommands,
    at: Transform,
    color: Color,
    on_sprite: impl Bundle,
    on_quad: impl Bundle,
    mesh: Handle<Mesh>,
    material: Handle<M>,
) {
    brick.with_children(|parent| {
        parent
            .spawn((
                on_sprite,
                Sprite {
                    color,
                    custom_size: Some(Vec2::ZERO),
                    ..default()
                },
                at,
            ))
            .with_child((on_quad, BrickQuad, Mesh2d(mesh), MeshMaterial2d(material)));
    });
}

/// The mesh and material of a lit brick quad of `size` for manifest entry `key`, when it has
/// light colours, a bevel map and its unlit pattern (`<key>_raw`, kept by `art::light_bricks`).
pub fn lit_brick(
    art: &crate::art::Art,
    key: &str,
    size: Vec2,
    color: Color,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<LitBrick>,
) -> Option<(Handle<Mesh>, Handle<LitBrick>)> {
    let entry = art.entries.get(key)?;
    let ((hr, hg, hb, ha), (sr, sg, sb, sa)) = entry.light.as_ref()?.get()?;
    let (lr, lg, lb) = entry.line.unwrap_or(crate::art::BRICK_LINE);
    let bevel = art.images.get(entry.bevel.as_ref()?)?.clone();
    let pattern = art.images.get(&format!("{key}_raw"))?.clone();
    let material = materials.add(LitBrick {
        look: LitLook {
            highlight: Vec4::new(hr, hg, hb, ha),
            shadow: Vec4::new(sr, sg, sb, sa),
            border: Vec4::new(lr, lg, lb, 1.0),
            params: Vec4::ZERO,
            tint: Vec4::from_array(color.to_linear().to_f32_array()),
            water: Vec4::ZERO,
            water_line: Vec4::ZERO,
            water_colour: Vec4::ZERO,
        },
        bevel,
        pattern,
    });
    Some((meshes.add(Rectangle::new(size.x, size.y)), material))
}

/// Hands each brick quad its turn in the world (so the light stays up and to the left) and the
/// tint the effects gave its sprite.
#[allow(clippy::type_complexity)]
fn follow_bricks(
    config: Res<crate::config::GameConfig>,
    game: Option<Res<crate::gameplay::Match>>,
    sprites: Query<&Sprite>,
    lit: Query<(&MeshMaterial2d<LitBrick>, &GlobalTransform, &ChildOf), With<BrickQuad>>,
    galaxy: Query<(&MeshMaterial2d<GalaxyBrick>, &ChildOf), With<BrickQuad>>,
    diamond: Query<(&MeshMaterial2d<DiamondBrick>, &ChildOf), With<BrickQuad>>,
    holo: Query<(&MeshMaterial2d<HologramBrick>, &GlobalTransform, &ChildOf), With<BrickQuad>>,
    mut holo_materials: Option<ResMut<Assets<HologramBrick>>>,
    mut lit_materials: Option<ResMut<Assets<LitBrick>>>,
    mut galaxy_materials: Option<ResMut<Assets<GalaxyBrick>>>,
    mut diamond_materials: Option<ResMut<Assets<DiamondBrick>>>,
) {
    // The original turns the bricks' water on in Race (and the single-player race modes).
    let look = &config.water;
    let on = game.is_some_and(|g| g.mode.rules() == crate::config::ModeKind::Race);
    let rgb = |[r, g, b]: [u8; 3]| Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
    let water = (
        Vec4::new(-config.cells(config.kill_depth), if on { 1.0 } else { 0.0 }, config.cell_size, 0.0),
        rgb(look.brick_line),
        rgb(look.brick_colour),
    );
    let tint = |of: &ChildOf| {
        sprites
            .get(of.parent())
            .map_or(Vec4::ONE, |s| Vec4::from_array(s.color.to_linear().to_f32_array()))
    };
    if let Some(materials) = lit_materials.as_mut() {
        for (material, at, of) in &lit {
            let angle = at.rotation().to_euler(EulerRot::XYZ).2;
            let tint = tint(of);
            let same = materials
                .get(&material.0)
                .is_none_or(|m| {
                    (m.look.params.x - angle).abs() < 1e-3 && m.look.tint == tint && m.look.water == water.0
                });
            if !same && let Some(mut brick) = materials.get_mut(&material.0) {
                brick.look.params.x = angle;
                brick.look.tint = tint;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
    if let Some(materials) = holo_materials.as_mut() {
        for (material, at, of) in &holo {
            let angle = at.rotation().to_euler(EulerRot::XYZ).2;
            let tint = tint(of);
            let same = materials
                .get(&material.0)
                .is_none_or(|m| {
                    (m.look.params.x - angle).abs() < 1e-3 && m.look.tint == tint && m.look.water == water.0
                });
            if !same && let Some(mut brick) = materials.get_mut(&material.0) {
                brick.look.params.x = angle;
                brick.look.tint = tint;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
    if let Some(materials) = galaxy_materials.as_mut() {
        for (material, of) in &galaxy {
            let tint = tint(of);
            if materials.get(&material.0).is_some_and(|m| m.look.tint != tint || m.look.water != water.0)
                && let Some(mut brick) = materials.get_mut(&material.0)
            {
                brick.look.tint = tint;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
    if let Some(materials) = diamond_materials.as_mut() {
        for (material, of) in &diamond {
            let tint = tint(of);
            if materials.get(&material.0).is_some_and(|m| m.look.tint != tint || m.look.water != water.0)
                && let Some(mut brick) = materials.get_mut(&material.0)
            {
                brick.look.tint = tint;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
}

const ZAP_SHADER: Handle<Shader> = uuid_handle!("9f6c3b12-48e7-4d05-b1a3-5e2d8c7f0a96");

const ZAP_WGSL: &str = r#"// A brick being zapped (docs/research/hologram.md's sibling notes in race-scene.md): white
// in its own outline, eaten away where the noise is under the cutoff, with a coloured edge
// 0.1 of the noise wide (the original's Custom/BrickLaser).
#import bevy_sprite::mesh2d_vertex_output::VertexOutput
#import bevy_render::color_operations::srgb_to_linear

struct ZapBrick {
    // rgb: the edge's colour (stored sRGB).
    edge: vec4<f32>,
    // x: the cutoff, 0 to 1 over the zap.
    params: vec4<f32>,
    tint: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: ZapBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var mask: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var mask_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var noise: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var noise_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let n = textureSample(noise, noise_sampler, mesh.uv).r;
    let a = textureSample(mask, mask_sampler, mesh.uv).a;
    let cutoff = material.params.x;
    if (n <= cutoff) {
        return vec4(0.0);
    }
    var colour = vec3(1.0);
    if (n <= cutoff + 0.1) {
        colour = srgb_to_linear(material.edge.rgb);
    }
    return vec4(colour * material.tint.rgb, a * material.tint.a);
}
"#;

/// A zapped brick's look: its outline (any of its pictures' alpha), the noise and the edge.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct ZapBrick {
    #[uniform(0)]
    pub look: ZapLook,
    #[texture(1)]
    #[sampler(2)]
    pub mask: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    pub noise: Handle<Image>,
}

/// The edge colour (stored sRGB), x the cutoff, and the tint.
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct ZapLook {
    pub edge: Vec4,
    pub params: Vec4,
    pub tint: Vec4,
}

impl Material2d for ZapBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(ZAP_SHADER)
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
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(LIT_SHADER.id(), Shader::from_wgsl(LIT_WGSL, "lit_brick.wgsl"));
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(HOLO_SHADER.id(), Shader::from_wgsl(HOLO_WGSL, "hologram_brick.wgsl"));
        let _ = app
            .world_mut()
            .resource_mut::<Assets<Shader>>()
            .insert(ZAP_SHADER.id(), Shader::from_wgsl(ZAP_WGSL, "zap_brick.wgsl"));
        app.add_plugins((
            Material2dPlugin::<GalaxyBrick>::default(),
            Material2dPlugin::<DiamondBrick>::default(),
            Material2dPlugin::<LitBrick>::default(),
            Material2dPlugin::<HologramBrick>::default(),
            Material2dPlugin::<ZapBrick>::default(),
        ))
        .add_systems(Update, follow_bricks);
    }
}
