//! The brick packs' own looks as a 2D material, written from our notes on the original's
//! brick shaders (docs/research/galaxy.md). Galaxy first; the WGSL is ours, kept here as a
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
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: GalaxyBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sky: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sky_sampler: sampler;

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
    var colour = clamp(hue(linear_to_srgb(sky_linear), material.params.x), vec3(0.0), vec3(1.0));

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
}

/// The shape's colours (stored sRGB values, as the original's material has them) and the
/// changing values: x hue turn (radians), y the brick's rotation, z pixels per cell, w flash.
#[derive(ShaderType, Clone, Copy, Debug)]
pub struct GalaxyLook {
    pub highlight: Vec4,
    pub shadow: Vec4,
    pub border: Vec4,
    pub params: Vec4,
}

impl Material2d for GalaxyBrick {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(GALAXY_SHADER)
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
        app.add_plugins(Material2dPlugin::<GalaxyBrick>::default());
    }
}
