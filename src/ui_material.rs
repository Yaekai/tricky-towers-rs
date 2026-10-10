//! Menu pictures drawn with a UI material: a picture turning inside its node and cut to it,
//! as the original's masked menu panels show their sunrays (UI_ModeMenuMask). Bevy cuts a
//! node's children to it, but not children that are turned, so the turn is done in the
//! material instead. The WGSL is ours, kept here as a string so it ships inside the game.

use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::shader::ShaderRef;
use bevy::ui_render::prelude::{UiMaterial, UiMaterialPlugin};

const TURN_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("6c2a9e51-0f84-4d37-b8a6-e93d1c57f402");

const TURN_WGSL: &str = r#"// A picture centred on its node, turned and cut to the node's edges.
#import bevy_ui::ui_vertex_output::UiVertexOutput

struct TurningPicture {
    color: vec4<f32>,
    // x: the turn (radians), yz: the picture's size as shares of the node's.
    params: vec4<f32>,
};

@group(1) @binding(0) var<uniform> material: TurningPicture;
@group(1) @binding(1) var picture: texture_2d<f32>;
@group(1) @binding(2) var picture_sampler: sampler;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {
    let p = (in.uv - 0.5) * in.size;
    let a = -material.params.x;
    let r = vec2(cos(a) * p.x - sin(a) * p.y, sin(a) * p.x + cos(a) * p.y);
    let uv = r / (material.params.yz * in.size) + 0.5;
    if (any(uv < vec2(0.0)) || any(uv > vec2(1.0))) {
        return vec4(0.0);
    }
    return textureSample(picture, picture_sampler, uv) * material.color;
}
"#;

/// A turning picture: its colour (linear), turn and size against its node's.
#[derive(Asset, TypePath, AsBindGroup, Clone)]
pub struct TurningPicture {
    #[uniform(0)]
    pub look: TurnLook,
    #[texture(1)]
    #[sampler(2)]
    pub picture: Handle<Image>,
}

#[derive(ShaderType, Clone, Copy, Debug)]
pub struct TurnLook {
    pub color: Vec4,
    pub params: Vec4,
}

impl TurningPicture {
    /// `picture` at `share` of its node's size (width, height), not turned.
    pub fn new(picture: Handle<Image>, share: Vec2) -> Self {
        Self {
            look: TurnLook {
                color: Vec4::ONE,
                params: Vec4::new(0.0, share.x, share.y, 0.0),
            },
            picture,
        }
    }
}

impl UiMaterial for TurningPicture {
    fn fragment_shader() -> ShaderRef {
        ShaderRef::Handle(TURN_SHADER)
    }
}

/// Sets a turning picture's turn (radians), when it changed.
pub fn set_turn(
    materials: &mut Assets<TurningPicture>,
    handle: &Handle<TurningPicture>,
    turn: f32,
) {
    if materials
        .get(handle)
        .is_some_and(|m| (m.look.params.x - turn).abs() > 1e-4)
        && let Some(mut m) = materials.get_mut(handle)
    {
        m.look.params.x = turn;
    }
}

pub struct MenuMaterialPlugin;

impl Plugin for MenuMaterialPlugin {
    fn build(&self, app: &mut App) {
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let _ = shaders.insert(
                TURN_SHADER.id(),
                Shader::from_wgsl(TURN_WGSL, "turning_picture.wgsl"),
            );
            app.add_plugins(UiMaterialPlugin::<TurningPicture>::default());
        }
    }
}
