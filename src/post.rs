//! Full-screen effects drawn over the world before the menus: the grey the original's cup
//! standings popup puts behind itself (its background image uses a Black & White material on
//! the screen, under the dark modal layer), and Endless Race's pacer, which turns the screen
//! under its line grey and pink (Custom/EndlessRacePacer on the EndlessRacePaceLayer sprite,
//! 75 units of screen hanging from the line). The WGSL is ours, kept here as a string.

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
    pacer: vec4<f32>,
};

@group(0) @binding(0) var screen_texture: texture_2d<f32>;
@group(0) @binding(1) var texture_sampler: sampler;
@group(0) @binding(2) var<uniform> settings: GreyOut;

@fragment
fn fragment(in: FullscreenVertexOutput) -> @location(0) vec4<f32> {
    let c = textureSample(screen_texture, texture_sampler, in.uv);
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

/// On the camera: how grey the world is drawn (x: 0 none, 1 fully), and the pacer's grey:
/// y its line's place on the screen (0 top, 1 bottom), z its strength, `pacer` its tint.
#[derive(Component, ExtractComponent, Clone, Copy, ShaderType)]
pub struct GreyOut {
    pub params: Vec4,
    pub pacer: Vec4,
}

impl Default for GreyOut {
    fn default() -> Self {
        Self {
            params: Vec4::new(0.0, 2.0, 0.0, 0.0),
            pacer: Vec4::ONE,
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
    lines: Query<(&crate::visuals::FinishLine, &GlobalTransform, &InheritedVisibility)>,
    mut cameras: Query<(&Camera, &GlobalTransform, &mut GreyOut)>,
    // Stopped or not, the grey when that last changed, and the time since.
    mut tween: Local<(bool, f32, f32)>,
) {
    let line = game
        .filter(|g| g.mode == crate::config::ModeKind::Endless && !menu.is_some_and(|m| m.0))
        .and_then(|_| lines.iter().find(|(_, _, shown)| shown.get()));
    let paused = line
        .and_then(|(l, _, _)| towers.get(l.tower).ok())
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
            .and_then(|(_, line, _)| {
                let size = camera.logical_viewport_size()?;
                let y = camera.world_to_viewport(at, line.translation()).ok()?.y;
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
            .add_systems(Update, (follow_grey, follow_pacer));
        if let Some(mut shaders) = app.world_mut().get_resource_mut::<Assets<Shader>>() {
            let _ = shaders.insert(
                GREY_SHADER.id(),
                Shader::from_wgsl(GREY_WGSL, "grey_out.wgsl"),
            );
            app.add_plugins(FullscreenMaterialPlugin::<GreyOut>::default());
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
