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
        ));
    }
}
