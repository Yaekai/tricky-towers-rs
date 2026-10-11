//! Full-screen effects drawn over the world before the menus: the grey the original's cup
//! standings popup puts behind itself (its background image uses a Black & White material on
//! the screen, under the dark modal layer). The WGSL is ours, kept here as a string.

use bevy::core_pipeline::Core2dSystems;
use bevy::core_pipeline::fullscreen_material::{FullscreenMaterial, FullscreenMaterialPlugin};
use bevy::core_pipeline::schedule::Core2d;
use bevy::ecs::schedule::ScheduleConfigs;
use bevy::ecs::system::BoxedSystem;
use bevy::prelude::*;
use bevy::render::extract_component::ExtractComponent;
use bevy::render::render_resource::ShaderType;
use bevy::shader::ShaderRef;

const GREY_SHADER: Handle<Shader> =
    bevy::asset::uuid_handle!("b19f4d27-63ac-4e58-9d10-7a2c5e8f3b61");

const GREY_WGSL: &str = r#"// The screen turned grey by `amount` (the original's Black & White shader: 0.21 R +
// 0.72 G + 0.07 B, on stored sRGB values).
#import bevy_core_pipeline::fullscreen_vertex_shader::FullscreenVertexOutput
#import bevy_render::color_operations::{linear_to_srgb, srgb_to_linear}

struct GreyOut {
    params: vec4<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: GreyOut;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(screen_texture, texture_sampler, in.uv);
    let s = linear_to_srgb(c.rgb);
    let g = dot(s, vec3(0.21, 0.72, 0.07));
    return vec4(srgb_to_linear(mix(s, vec3(g), settings.params.x)), c.a);
}
"#;

/// On the camera: how grey the world is drawn (0 none, 1 fully).
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType, Default)]
pub struct GreyOut {
    pub params: Vec4,
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

fn follow_grey(
    mut commands: Commands,
    time: Res<Time>,
    target: Res<GreyTarget>,
    mut cameras: Query<(Entity, Option<&mut GreyOut>), With<Camera2d>>,
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
            .add_systems(Update, follow_grey);
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let _ = shaders.insert(
                GREY_SHADER.id(),
                Shader::from_wgsl(GREY_WGSL, "grey_out.wgsl"),
            );
            app.add_plugins(FullscreenMaterialPlugin::<GreyOut>::default());
        }
    }
}
