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
pub mod casts;
pub mod collapse;
pub mod config;
pub mod cups;
pub mod cups_menu;
pub mod debug;
#[cfg(feature = "debug-panel")]
pub mod debug_panel;
pub mod effects;
pub mod game_info;
pub mod gameplay;
pub mod hud;
pub mod input;
pub mod keybinding;
pub mod letters;
pub mod menu;
pub mod mods;
pub mod navbar;
pub mod offscreen;
pub mod packs;
pub mod particles;
pub mod popup;
pub mod results;
pub mod rival;
pub mod select;
pub mod settings;
pub mod single;
pub mod spells;
pub mod start;
pub mod stats;
pub mod trial_list;
pub mod trials;
pub mod trials_menu;
pub mod visuals;
pub mod weather;
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
            collapse::CollapsePlugin,
            trials::TrialsPlugin,
            // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
            debug::DebugPlugin,
            // DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
            cups::CupsPlugin,
            ai::AiPlugin,
        ));
    }
}
