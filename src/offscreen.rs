//! Offscreen runs: no window and no sound. The game draws into a picture instead
//! of a window and saves it at set times, then quits. Handy for scripted checks
//! on a computer someone is using: nothing shows up and nothing plays.
//!
//! `--offscreen <folder>` turns it on; `--shots 5,20` picks the seconds to save at.

use std::path::PathBuf;

use bevy::app::AppExit;
use bevy::camera::RenderTarget;
use bevy::prelude::*;
use bevy::render::render_resource::TextureFormat;
use bevy::render::view::screenshot::{Screenshot, save_to_disk};

use crate::gameplay::Match;
use crate::visuals::ViewSize;

pub struct OffscreenPlugin {
    /// Folder the pictures go in.
    pub dir: PathBuf,
    /// Seconds after start to save a picture at.
    pub shots: Vec<f32>,
    /// Picture size in pixels.
    pub size: UVec2,
}

#[derive(Resource)]
struct Shots {
    dir: PathBuf,
    times: Vec<f32>,
    taken: usize,
    target: Handle<Image>,
    elapsed: f32,
}

impl Plugin for OffscreenPlugin {
    fn build(&self, app: &mut App) {
        let mut times = self.shots.clone();
        times.sort_by(f32::total_cmp);
        let image = Image::new_target_texture(
            self.size.x,
            self.size.y,
            TextureFormat::Rgba8UnormSrgb,
            None,
        );
        let target = app.world_mut().resource_mut::<Assets<Image>>().add(image);
        app.insert_resource(ViewSize(self.size.as_vec2()))
            .insert_resource(Shots {
                dir: self.dir.clone(),
                times,
                taken: 0,
                target,
                elapsed: 0.0,
            })
            .add_systems(Update, (aim_cameras, take_shots));
    }
}

/// Points every camera at the picture instead of a window.
fn aim_cameras(
    mut commands: Commands,
    shots: Res<Shots>,
    cameras: Query<(Entity, Option<&RenderTarget>), With<Camera>>,
) {
    for (entity, target) in &cameras {
        if !matches!(target, Some(RenderTarget::Image(_))) {
            // With no window, the UI needs telling which camera to draw on.
            commands.entity(entity).insert((
                RenderTarget::Image(shots.target.clone().into()),
                IsDefaultUiCamera,
            ));
        }
    }
}

fn take_shots(
    mut commands: Commands,
    time: Res<Time>,
    game: Res<Match>,
    mut shots: ResMut<Shots>,
    mut exit: MessageWriter<AppExit>,
) {
    shots.elapsed += time.delta_secs();
    let Some(&due) = shots.times.get(shots.taken) else {
        // Give the last picture a moment to be written, then quit.
        let last = shots.times.last().copied().unwrap_or(0.0);
        if shots.elapsed > last + 2.0 {
            exit.write(AppExit::Success);
        }
        return;
    };
    if shots.elapsed < due {
        return;
    }
    shots.taken += 1;
    let mode = format!("{:?}", game.mode).to_lowercase();
    let path = shots.dir.join(format!("{mode}_{due:.0}s.png"));
    info!("saving {}", path.display());
    commands
        .spawn(Screenshot::image(shots.target.clone()))
        .observe(save_to_disk(path));
}
