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
use bevy::camera::visibility::{RenderLayers, VisibilitySystems};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::sprite_render::{AlphaMode2d, Material2d, Material2dPlugin};

const DECORATION_SHADER: Handle<Shader> = uuid_handle!("68bb8140-7021-40bf-b58e-2fb6e07cedf9");

const DECORATION_WGSL: &str = r#"
#import bevy_sprite::mesh2d_vertex_output::VertexOutput

struct Decoration {
    clip: vec4<f32>,
    tint: vec4<f32>,
    uv: vec4<f32>,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: Decoration;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var picture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var picture_sampler: sampler;

@fragment
fn fragment(mesh: VertexOutput) -> @location(0) vec4<f32> {
    let uv = material.uv.xy + mesh.uv * material.uv.zw;
    let colour = textureSample(picture, picture_sampler, uv) * material.tint;
    if mesh.world_position.x < material.clip.x || mesh.world_position.x > material.clip.y {
        discard;
    }
    return colour;
}
"#;

#[derive(Asset, TypePath, AsBindGroup, Clone)]
struct ClippedDecoration {
    #[uniform(0)]
    look: DecorationLook,
    #[texture(1)]
    #[sampler(2)]
    picture: Handle<Image>,
}

#[derive(ShaderType, Clone, Copy, PartialEq)]
struct DecorationLook {
    clip: Vec4,
    tint: Vec4,
    uv: Vec4,
}

impl Material2d for ClippedDecoration {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(DECORATION_SHADER)
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

/// A decoration that belongs to a tower's world and is cut by its camera.
#[derive(Component)]
pub(crate) struct LaneDecoration(pub Entity);

#[derive(Component)]
struct DecorationQuad(Entity);

fn fixed_lane_half(config: &crate::config::GameConfig, puzzle: bool, lanes: usize) -> Option<f32> {
    (!puzzle && lanes > 1).then(|| config.cells(config.lanes.width(lanes)) * 0.5)
}

fn decoration_look(
    sprite: &Sprite,
    anchor: Vec2,
    image_size: Vec2,
    clip: Vec4,
) -> Option<(Transform, DecorationLook)> {
    if sprite.texture_atlas.is_some() || !matches!(sprite.image_mode, SpriteImageMode::Auto) {
        return None;
    }
    let rect = sprite.rect.unwrap_or(Rect::from_corners(Vec2::ZERO, image_size));
    let size = sprite.custom_size.unwrap_or(rect.size());
    if size.min_element() <= 0.0 || rect.size().min_element() <= 0.0 {
        return None;
    }
    let lo = rect.min / image_size;
    let hi = rect.max / image_size;
    let uv_min = Vec2::new(if sprite.flip_x { hi.x } else { lo.x }, if sprite.flip_y { hi.y } else { lo.y });
    let uv_max = Vec2::new(if sprite.flip_x { lo.x } else { hi.x }, if sprite.flip_y { lo.y } else { hi.y });
    let look = DecorationLook {
        clip,
        tint: Vec4::from_array(sprite.color.to_linear().to_f32_array()),
        uv: Vec4::new(uv_min.x, uv_min.y, uv_max.x - uv_min.x, uv_max.y - uv_min.y),
    };
    let transform = Transform::from_translation((-anchor * size).extend(0.0)).with_scale(size.extend(1.0));
    Some((transform, look))
}

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn clip_decorations(
    mut commands: Commands,
    config: Res<crate::config::GameConfig>,
    game: Res<crate::gameplay::Match>,
    towers: Query<&crate::gameplay::Tower>,
    sprites: Query<(Entity, &Sprite, &bevy::sprite::Anchor, &LaneDecoration, Option<&DecorationQuad>, Option<&RenderLayers>)>,
    mut quads: Query<(&mut Transform, &MeshMaterial2d<ClippedDecoration>), Without<LaneDecoration>>,
    images: Res<Assets<Image>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ClippedDecoration>>,
    mut unit: Local<Option<Handle<Mesh>>>,
) {
    let puzzle = game.mode.rules() == crate::config::ModeKind::Puzzle;
    let half = fixed_lane_half(&config, puzzle, towers.iter().count());
    for (entity, sprite, anchor, owner, quad, layers) in &sprites {
        let current = half.zip(towers.get(owner.0).ok()).and_then(|(half, tower)| {
            let origin = tower.origin_x;
            let clip = Vec4::new(origin - half, origin + half, 0.0, 0.0);
            decoration_look(sprite, anchor.0, images.get(&sprite.image)?.size_f32(), clip)
        });
        let Some((transform, look)) = current else {
            if let Some(quad) = quad {
                commands.entity(quad.0).despawn();
                commands.entity(entity).remove::<(DecorationQuad, RenderLayers)>();
            }
            continue;
        };
        if let Some(quad) = quad {
            if let Ok((mut place, material)) = quads.get_mut(quad.0) {
                place.set_if_neq(transform);
                if materials.get(&material.0).is_some_and(|m| m.look != look || m.picture != sprite.image)
                    && let Some(mut m) = materials.get_mut(&material.0)
                {
                    m.look = look;
                    m.picture = sprite.image.clone();
                }
            }
        } else if layers.is_none() {
            // Keep the source sprite for animation and tint. The quad inherits its transform and visibility.
            let mesh = unit.get_or_insert_with(|| meshes.add(Rectangle::new(1.0, 1.0))).clone();
            let material = materials.add(ClippedDecoration { look, picture: sprite.image.clone() });
            let quad = commands.spawn((Mesh2d(mesh), MeshMaterial2d(material), transform, ChildOf(entity))).id();
            commands.entity(entity).insert((DecorationQuad(quad), RenderLayers::none()));
        }
    }
}

const GALAXY_SHADER: Handle<Shader> = uuid_handle!("5b0c8f8e-2a4e-4c1e-9a61-3f1b7a2d9c04");

const GALAXY_WGSL: &str = r#"// A Galaxy brick, drawn the way the original's brick shader does it (docs/research/galaxy.md),
// written from those notes. The maths runs on stored (sRGB) values like the original, which
// blends in gamma space, and the result goes back to linear light for Bevy.
#import bevy_sprite::{
    mesh2d_vertex_output::VertexOutput,
    mesh2d_view_bindings::{globals, view},
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
    // Its tower's lane on the screen: x left, y right (world pixels), z 1 when cut there.
    clip: vec4<f32>,
    // x: 1 when iced (ICE_ON).
    ice: vec4<f32>,

};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: GalaxyBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var sky: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var sky_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(5) var star: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(6) var star_sampler: sampler;

// ICE_ON's shine: two sets of diagonal bands fixed to the camera, a cell apart in units.
fn ice_shine(world: vec2<f32>, cell: f32) -> f32 {
    let at = (world - view.world_position.xy) / cell;
    let band = fract(vec2(at.x + at.y) * vec2(0.5, 0.6)) - vec2(0.8, 0.9);
    return clamp(ceil(max(band.x, 0.0) + max(band.y, 0.0)), 0.0, 1.0);
}

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
// Each tower has its own camera over its share of the screen in the original, so a brick
// is cut off where it leaves its lane.
fn in_lane(world_x: f32) -> f32 {
    if (material.clip.z < 0.5) {
        return 1.0;
    }
    return step(material.clip.x, world_x) * step(world_x, material.clip.y);
}

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
    var map_alpha = map.a;
    var border = material.border.rgb;
    if (material.ice.x > 0.5) {
        // ICE_ON, as the lit bricks': the sky half way to (0.6, 0.8, 1) with the shine, the
        // highlight half way to icy white, a lilac shadow and a blue outline.
        colour = mix(colour, vec3(0.6, 0.8, 1.0), 0.5) + ice_shine(mesh.world_position.xy, material.params.z) * 0.1;
        map_alpha = map.a * 1.1;
        let light = mix(material.highlight, vec4(0.945098, 0.984314, 1.0, 1.0), 0.5);
        colour = mix(colour, light.rgb, clamp(d, 0.0, 1.0) * light.a);
        colour = mix(colour, vec3(0.470588, 0.431373, 0.745098), clamp(-d, 0.0, 1.0));
        border = vec3(0.329412, 0.337255, 0.752941);
    } else {
        colour = mix(colour, material.highlight.rgb, clamp(d, 0.0, 1.0) * material.highlight.a);
        colour = mix(colour, material.shadow.rgb, clamp(-d, 0.0, 1.0) * material.shadow.a);
    }

    // The outline (the map's blue, and its outside) in the border colour.
    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, border, outline);
    colour = colour + material.params.w * (1.0 - colour);
    let wet = under_water(mesh.world_position.y, &colour);

    let alpha = max(map_alpha, map.b) * wet;
    return vec4(srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0))) * material.tint.rgb, alpha * material.tint.a * in_lane(mesh.world_position.x));
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
    /// The tower's lane: x left, y right (world pixels), z 1 when the brick is cut there.
    pub clip: Vec4,
    /// x: 1 when iced (the shader's ICE_ON).
    pub ice: Vec4,
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
    mesh2d_view_bindings::{globals, view},
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
    // Its tower's lane on the screen: x left, y right (world pixels), z 1 when cut there.
    clip: vec4<f32>,
    // x: 1 when iced (ICE_ON).
    ice: vec4<f32>,

};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: DiamondBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pattern: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var pattern_sampler: sampler;

// ICE_ON's shine: two sets of diagonal bands fixed to the camera, a cell apart in units.
fn ice_shine(world: vec2<f32>, cell: f32) -> f32 {
    let at = (world - view.world_position.xy) / cell;
    let band = fract(vec2(at.x + at.y) * vec2(0.5, 0.6)) - vec2(0.8, 0.9);
    return clamp(ceil(max(band.x, 0.0) + max(band.y, 0.0)), 0.0, 1.0);
}

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
// Each tower has its own camera over its share of the screen in the original, so a brick
// is cut off where it leaves its lane.
fn in_lane(world_x: f32) -> f32 {
    if (material.clip.z < 0.5) {
        return 1.0;
    }
    return step(material.clip.x, world_x) * step(world_x, material.clip.y);
}

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
    var pattern_alpha = pat_linear.a;
    var border = material.border.rgb;
    if (material.ice.x > 0.5) {
        // ICE_ON: half way to (0.6, 0.8, 1) with the shine, then lit like the ice bricks; the
        // pack has no highlight, so its light is icy white at half strength, then a lilac
        // shadow and a blue outline.
        colour = mix(colour, vec3(0.6, 0.8, 1.0), 0.5) + ice_shine(mesh.world_position.xy, material.water.z) * 0.1;
        pattern_alpha = pattern_alpha * 1.1;
        let d = dot(world_n, vec2(-0.44721359, 0.89442718));
        colour = mix(colour, vec3(0.472549, 0.492157, 0.5), clamp(d, 0.0, 1.0) * 0.5);
        colour = mix(colour, vec3(0.470588, 0.431373, 0.745098), clamp(-d, 0.0, 1.0));
        border = vec3(0.329412, 0.337255, 0.752941);
    }

    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, border, outline);
    colour = colour + material.params.w * (1.0 - colour);
    let wet = under_water(mesh.world_position.y, &colour);
    let alpha = max(pattern_alpha, map.b) * wet;
    return vec4(srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0))) * material.tint.rgb, alpha * material.tint.a * in_lane(mesh.world_position.x));
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
    /// The tower's lane: x left, y right (world pixels), z 1 when the brick is cut there.
    pub clip: Vec4,
    /// x: 1 when iced (the shader's ICE_ON).
    pub ice: Vec4,
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
#import bevy_sprite::mesh2d_view_bindings::view

struct LitBrick {
    // rgb: colour, a: strength.
    highlight: vec4<f32>,
    shadow: vec4<f32>,
    // rgb: the outline colour.
    border: vec4<f32>,
    // x: the brick's rotation (radians), y: 1 when iced (ICE_ON), w: flash.
    params: vec4<f32>,
    // The effects' tint, multiplied in like a sprite's colour.
    tint: vec4<f32>,
    // The water: x its height (world pixels), y 1 when on, z pixels per cell; its line's and
    // its own colour.
    water: vec4<f32>,
    water_line: vec4<f32>,
    water_colour: vec4<f32>,
    // Its tower's lane on the screen: x left, y right (world pixels), z 1 when cut there.
    clip: vec4<f32>,

};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: LitBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pattern: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var pattern_sampler: sampler;

// Under the water in Race (the original's WATER_ON): the water line's colour for the first
// 0.12 of a cell, then the water's, fading out over 2 cells (stored sRGB colours).
// Each tower has its own camera over its share of the screen in the original, so a brick
// is cut off where it leaves its lane.
fn in_lane(world_x: f32) -> f32 {
    if (material.clip.z < 0.5) {
        return 1.0;
    }
    return step(material.clip.x, world_x) * step(world_x, material.clip.y);
}

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
    var pattern_alpha = pat_linear.a;
    var border = material.border.rgb;
    if (material.params.y > 0.5) {
        // ICE_ON (an iced pack brick): the pattern half way to (0.6, 0.8, 1), two sets of
        // diagonal shine bands fixed to the camera, the highlight half way to icy white, a fixed
        // lilac shadow and a blue outline.
        let at = (mesh.world_position.xy - view.world_position.xy) / material.water.z;
        let band = fract(vec2(at.x + at.y) * vec2(0.5, 0.6)) - vec2(0.8, 0.9);
        let shine = clamp(ceil(max(band.x, 0.0) + max(band.y, 0.0)), 0.0, 1.0);
        colour = mix(colour, vec3(0.6, 0.8, 1.0), 0.5) + shine * 0.1;
        pattern_alpha = pattern_alpha * 1.1;
        let light = mix(material.highlight, vec4(0.945098, 0.984314, 1.0, 1.0), 0.5);
        colour = mix(colour, light.rgb, clamp(d, 0.0, 1.0) * light.a);
        colour = mix(colour, vec3(0.470588, 0.431373, 0.745098), clamp(-d, 0.0, 1.0));
        border = vec3(0.329412, 0.337255, 0.752941);
    } else {
        colour = mix(colour, material.highlight.rgb, clamp(d, 0.0, 1.0) * material.highlight.a);
        colour = mix(colour, material.shadow.rgb, clamp(-d, 0.0, 1.0) * material.shadow.a);
    }

    // The outline and cracks (the map's blue, and its outside) in the line colour.
    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, border, outline);
    colour = colour + material.params.w * (1.0 - colour);
    let wet = under_water(mesh.world_position.y, &colour);

    let alpha = max(pattern_alpha, map.b) * wet;
    let out = srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0)));
    return vec4(out * material.tint.rgb, alpha * material.tint.a * in_lane(mesh.world_position.x));
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
    /// The tower's lane: x left, y right (world pixels), z 1 when the brick is cut there.
    pub clip: Vec4,
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
    mesh2d_view_bindings::{globals, view},
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
    // Its tower's lane on the screen: x left, y right (world pixels), z 1 when cut there.
    clip: vec4<f32>,
    // x: 1 when iced (ICE_ON).
    ice: vec4<f32>,

};

@group(#{MATERIAL_BIND_GROUP}) @binding(0) var<uniform> material: HologramBrick;
@group(#{MATERIAL_BIND_GROUP}) @binding(1) var bevel: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2) var bevel_sampler: sampler;
@group(#{MATERIAL_BIND_GROUP}) @binding(3) var pattern: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(4) var pattern_sampler: sampler;

// ICE_ON's shine: two sets of diagonal bands fixed to the camera, a cell apart in units.
fn ice_shine(world: vec2<f32>, cell: f32) -> f32 {
    let at = (world - view.world_position.xy) / cell;
    let band = fract(vec2(at.x + at.y) * vec2(0.5, 0.6)) - vec2(0.8, 0.9);
    return clamp(ceil(max(band.x, 0.0) + max(band.y, 0.0)), 0.0, 1.0);
}

// The shader's hash.
fn hash(n: f32) -> f32 {
    return fract(sin(n) * 18336.822);
}

// Under the water in Race (the original's WATER_ON): the water line's colour for the first
// 0.12 of a cell, then the water's, fading out over 2 cells (stored sRGB colours).
// Each tower has its own camera over its share of the screen in the original, so a brick
// is cut off where it leaves its lane.
fn in_lane(world_x: f32) -> f32 {
    if (material.clip.z < 0.5) {
        return 1.0;
    }
    return step(material.clip.x, world_x) * step(world_x, material.clip.y);
}

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
    // The outline, black at rest, (0.8, 1, 1) through a glitch.
    var border = d * vec3(0.8, 1.0, 1.0);
    var map_alpha = flicker * map.a;
    if (material.ice.x > 0.5) {
        // ICE_ON: half way to (0.6, 0.8, 1) with the shine, the highlight half way to icy
        // white, a lilac shadow and a blue outline.
        colour = mix(colour, vec3(0.6, 0.8, 1.0), 0.5) + ice_shine(mesh.world_position.xy, material.params.z) * 0.1;
        map_alpha = map_alpha * 1.1;
        let light = mix(material.highlight, vec4(0.945098, 0.984314, 1.0, 1.0), 0.5);
        colour = mix(colour, light.rgb, clamp(l, 0.0, 1.0) * light.a);
        colour = mix(colour, vec3(0.470588, 0.431373, 0.745098), clamp(-l, 0.0, 1.0));
        border = vec3(0.329412, 0.337255, 0.752941);
    } else {
        colour = mix(colour, material.highlight.rgb, clamp(l, 0.0, 1.0) * material.highlight.a);
        colour = mix(colour, material.shadow.rgb, clamp(-l, 0.0, 1.0) * material.shadow.a);
    }

    let outline = 1.0 - (1.0 - map.b) * map.a;
    colour = mix(colour, border, outline);
    colour = colour + material.params.w * (1.0 - colour);
    let wet = under_water(mesh.world_position.y, &colour);
    let alpha = max(map_alpha, map.b) * wet;
    let out = srgb_to_linear(clamp(colour, vec3(0.0), vec3(1.0)));
    return vec4(out * material.tint.rgb, alpha * material.tint.a * in_lane(mesh.world_position.x));
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
    /// The tower's lane: x left, y right (world pixels), z 1 when the brick is cut there.
    pub clip: Vec4,
    /// x: 1 when iced (the shader's ICE_ON).
    pub ice: Vec4,
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
    ice: bool,
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
            clip: Vec4::ZERO,
            ice: Vec4::X * ice as u8 as f32,
        },
        bevel,
        pattern,
    });
    Some((meshes.add(Rectangle::new(size.x, size.y)), material))
}

/// A quad drawn by one of the brick materials; its parent's `Sprite` carries the effects' tint.
#[derive(Component)]
pub struct BrickQuad;

/// On a brick: how white its material draws, 1 at a flash (FlashBrickModifier's `_Flash`).
#[derive(Component)]
pub struct BrickFlash(pub f32);

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
    ice: bool,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<LitBrick>,
) -> Option<(Handle<Mesh>, Handle<LitBrick>)> {
    let entry = art.entries.get(key)?;
    let ((hr, hg, hb, mut ha), (sr, sg, sb, mut sa)) = entry.light.as_ref()?.get()?;
    if crate::art::unlit_brick(key) {
        (ha, sa) = (0.0, 0.0);
    }
    let (lr, lg, lb) = entry.line.unwrap_or(crate::art::BRICK_LINE);
    let bevel = art.images.get(entry.bevel.as_ref()?)?.clone();
    let pattern = art.images.get(&format!("{key}_raw"))?.clone();
    let material = materials.add(LitBrick {
        look: LitLook {
            highlight: Vec4::new(hr, hg, hb, ha),
            shadow: Vec4::new(sr, sg, sb, sa),
            border: Vec4::new(lr, lg, lb, 1.0),
            params: Vec4::new(0.0, if ice { 1.0 } else { 0.0 }, 0.0, 0.0),
            tint: Vec4::from_array(color.to_linear().to_f32_array()),
            water: Vec4::ZERO,
            water_line: Vec4::ZERO,
            water_colour: Vec4::ZERO,
            clip: Vec4::ZERO,
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
    parents: Query<&ChildOf>,
    flashes: Query<&BrickFlash>,
    bricks: Query<&crate::gameplay::Brick>,
    towers: Query<&crate::gameplay::Tower>,
) {
    // The original turns the bricks' water on in Race (and the single-player race modes).
    let look = &config.water;
    let on = game.as_ref().is_some_and(|g| g.mode.rules() == crate::config::ModeKind::Race);
    let rgb = |[r, g, b]: [u8; 3]| Vec4::new(r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0, 1.0);
    let water = (
        Vec4::new(-config.cells(config.kill_depth), if on { 1.0 } else { 0.0 }, config.cell_size, 0.0),
        rgb(look.brick_line),
        rgb(look.brick_colour),
    );
    // The quad's parent is the tint sprite, whose parent is the brick.
    let flash = |of: &ChildOf| {
        parents
            .get(of.parent())
            .and_then(|brick| flashes.get(brick.parent()))
            .map_or(0.0, |f| f.0)
    };
    // The lane each tower's camera shows: its share of the screen around its centre
    // (`Lanes`). A lone tower's camera is the whole screen, so nothing is cut.
    // Puzzle battles give each tower its own camera and viewport (crate::lanes), which cut
    // it at its lane's edge, so the materials don't cut Puzzle.
    let puzzle = game.as_ref().is_some_and(|g| g.mode.rules() == crate::config::ModeKind::Puzzle);
    let half = fixed_lane_half(&config, puzzle, towers.iter().count());
    let clip = |of: &ChildOf| {
        parents
            .get(of.parent())
            .ok()
            .and_then(|brick| bricks.get(brick.parent()).ok())
            .and_then(|brick| towers.get(brick.tower).ok())
            .zip(half)
            .map_or(Vec4::ZERO, |(t, half)| Vec4::new(t.origin_x - half, t.origin_x + half, 1.0, 0.0))
    };
    let tint = |of: &ChildOf| {
        sprites
            .get(of.parent())
            .map_or(Vec4::ONE, |s| Vec4::from_array(s.color.to_linear().to_f32_array()))
    };
    if let Some(materials) = lit_materials.as_mut() {
        for (material, at, of) in &lit {
            let angle = at.rotation().to_euler(EulerRot::XYZ).2;
            let tint = tint(of);
            let flash = flash(of);
            let lane = clip(of);
            let same = materials
                .get(&material.0)
                .is_none_or(|m| {
                    (m.look.params.x - angle).abs() < 1e-3 && m.look.tint == tint && m.look.water == water.0
                        && m.look.params.w == flash && m.look.clip == lane
                });
            if !same && let Some(mut brick) = materials.get_mut(&material.0) {
                brick.look.clip = lane;
                brick.look.params.x = angle;
                brick.look.params.w = flash;
                brick.look.tint = tint;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
    if let Some(materials) = holo_materials.as_mut() {
        for (material, at, of) in &holo {
            let angle = at.rotation().to_euler(EulerRot::XYZ).2;
            let tint = tint(of);
            let flash = flash(of);
            let lane = clip(of);
            let same = materials
                .get(&material.0)
                .is_none_or(|m| {
                    (m.look.params.x - angle).abs() < 1e-3 && m.look.tint == tint && m.look.water == water.0
                        && m.look.params.w == flash && m.look.clip == lane
                });
            if !same && let Some(mut brick) = materials.get_mut(&material.0) {
                brick.look.clip = lane;
                brick.look.params.x = angle;
                brick.look.params.w = flash;
                brick.look.tint = tint;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
    if let Some(materials) = galaxy_materials.as_mut() {
        for (material, of) in &galaxy {
            let tint = tint(of);
            let flash = flash(of);
            let lane = clip(of);
            if materials.get(&material.0).is_some_and(|m| m.look.tint != tint || m.look.params.w != flash || m.look.water != water.0 || m.look.clip != lane)
                && let Some(mut brick) = materials.get_mut(&material.0)
            {
                brick.look.clip = lane;
                brick.look.tint = tint;
                brick.look.params.w = flash;
                (brick.look.water, brick.look.water_line, brick.look.water_colour) = water;
            }
        }
    }
    if let Some(materials) = diamond_materials.as_mut() {
        for (material, of) in &diamond {
            let tint = tint(of);
            let flash = flash(of);
            let lane = clip(of);
            if materials.get(&material.0).is_some_and(|m| m.look.tint != tint || m.look.params.w != flash || m.look.water != water.0 || m.look.clip != lane)
                && let Some(mut brick) = materials.get_mut(&material.0)
            {
                brick.look.clip = lane;
                brick.look.tint = tint;
                brick.look.params.w = flash;
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
        let _ = app.world_mut().resource_mut::<Assets<Shader>>().insert(
            DECORATION_SHADER.id(), Shader::from_wgsl(DECORATION_WGSL, "clipped_decoration.wgsl"),
        );
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
            Material2dPlugin::<ClippedDecoration>::default(),
        ))
        .add_systems(Update, follow_bricks)
        .add_systems(PostUpdate, clip_decorations
            .before(bevy::transform::TransformSystems::Propagate)
            .before(bevy::asset::AssetEventSystems)
            .before(VisibilitySystems::VisibilityPropagate)
            .before(VisibilitySystems::CalculateBounds));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{GameConfig, ModeKind};
    use crate::gameplay::{Match, Tower};

    #[test]
    fn clipped_decoration_preserves_the_sprite_frame_flip_and_anchor() {
        let sprite = Sprite {
            rect: Some(Rect::from_corners(Vec2::new(8.0, 16.0), Vec2::new(24.0, 32.0))),
            custom_size: Some(Vec2::new(40.0, 20.0)),
            flip_x: true,
            ..default()
        };
        let clip = Vec4::new(-90.0, -20.0, 0.0, 0.0);
        let (quad, look) = decoration_look(&sprite, Vec2::new(0.25, -0.5), Vec2::splat(64.0), clip).unwrap();
        assert_eq!(quad.scale, Vec3::new(40.0, 20.0, 1.0));
        assert_eq!(quad.translation, Vec3::new(-10.0, 10.0, 0.0));
        assert_eq!(look.uv, Vec4::new(0.375, 0.25, -0.25, 0.25));
        assert_eq!(look.clip, clip);
    }

    #[test]
    fn a_decoration_uses_its_owners_lane_and_returns_to_a_sprite_in_puzzle() {
        let mut app = App::new();
        app.init_resource::<GameConfig>()
            .init_resource::<Assets<Image>>()
            .init_resource::<Assets<Mesh>>()
            .init_resource::<Assets<ClippedDecoration>>()
            .insert_resource(Match { mode: ModeKind::Race, ..default() })
            .add_systems(PostUpdate, clip_decorations);
        let tower = |index, origin_x| Tower { index, origin_x, human: true, player: Some(index) };
        let world = app.world_mut();
        world.spawn(tower(0, -100.0));
        let owner = world.spawn(tower(1, 100.0)).id();
        let picture = world.resource_mut::<Assets<Image>>().add(Image::default());
        let sprite = world.spawn((Sprite::from_image(picture), LaneDecoration(owner))).id();
        app.update();
        let world = app.world();
        let config = world.resource::<GameConfig>();
        let half = config.cells(config.lanes.width(2)) * 0.5;
        let quad = world.get::<DecorationQuad>(sprite).unwrap().0;
        let material = world.get::<MeshMaterial2d<ClippedDecoration>>(quad).unwrap();
        assert_eq!(world.get::<RenderLayers>(sprite), Some(&RenderLayers::none()));
        assert_eq!(world.get::<ChildOf>(quad).unwrap().parent(), sprite);
        assert_eq!(world.resource::<Assets<ClippedDecoration>>().get(&material.0).unwrap().look.clip,
            Vec4::new(100.0 - half, 100.0 + half, 0.0, 0.0));

        app.world_mut().resource_mut::<Match>().mode = ModeKind::Puzzle;
        app.update();
        assert!(app.world().get_entity(quad).is_err());
        assert!(app.world().get::<Sprite>(sprite).is_some());
        assert!(app.world().get::<RenderLayers>(sprite).is_none());
    }
}
