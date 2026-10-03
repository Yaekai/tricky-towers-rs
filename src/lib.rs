//! A clean-room, moddable reimplementation of a physics brick-stacking game.
//!
//! The game is split into plugins so each layer can be swapped by a mod:
//! - [`TowerCorePlugin`]: config, physics, the gameplay loop, spells and AI
//!   opponents (headless-safe).
//! - [`input::KeyboardInputPlugin`]: keyboard to your tower's [`gameplay::PlayerInput`].
//! - [`menu::MenuPlugin`]: mode and opponent selection.
//! - [`visuals::VisualsPlugin`]: placeholder sprites, camera and HUD.

pub mod ai;
pub mod art;
pub mod config;
pub mod gameplay;
pub mod input;
pub mod letters;
pub mod menu;
pub mod spells;
pub mod visuals;
pub mod wizards;

use avian2d::prelude::*;
use bevy::prelude::*;

use config::GameConfig;

/// Physics plus gameplay. Needs a [`GameConfig`] resource; inserts the default
/// one if none was added before this plugin.
pub struct TowerCorePlugin;

impl Plugin for TowerCorePlugin {
    fn build(&self, app: &mut App) {
        if !app.world().contains_resource::<GameConfig>() {
            app.insert_resource(GameConfig::default());
        }
        let cell = app.world().resource::<GameConfig>().cell_size;
        app.add_plugins((
            PhysicsPlugins::default().with_length_unit(cell),
            gameplay::GameplayPlugin,
            ai::AiPlugin,
        ));
    }
}
