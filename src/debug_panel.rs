//! DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
//!
//! The F1 debug panel for people playing the game: buttons for the debug commands
//! (src/debug.rs) and a live readout of every tower. Built only with the default
//! `debug-panel` feature and only in a window (not offscreen).

use bevy::prelude::*;
use bevy_egui::{EguiContexts, EguiPlugin, EguiPrimaryContextPass, egui};

use crate::config::{Difficulty, GameConfig, ModeKind, SpellKind};
use crate::debug::{DebugCommand, DebugQueue, Key};
use crate::gameplay::{Match, Tower, TowerState};

pub struct DebugPanelPlugin;

impl Plugin for DebugPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .init_resource::<PanelState>()
            .add_systems(Update, toggle_panel)
            .add_systems(EguiPrimaryContextPass, draw_panel);
    }
}

#[derive(Resource)]
struct PanelState {
    open: bool,
    tower: usize,
    spell: SpellKind,
    lives: u32,
    difficulty: Difficulty,
    opponents: u32,
}

impl Default for PanelState {
    fn default() -> Self {
        Self {
            open: false,
            tower: 0,
            spell: SpellKind::Large,
            lives: 3,
            difficulty: Difficulty::Normal,
            opponents: 1,
        }
    }
}

const SPELLS: [SpellKind; 17] = [
    SpellKind::Ivy,
    SpellKind::Petrify,
    SpellKind::Undo,
    SpellKind::ExtraLife,
    SpellKind::Minibase,
    SpellKind::AutoBuild,
    SpellKind::Large,
    SpellKind::Fast,
    SpellKind::NoRotate,
    SpellKind::AutoRotate,
    SpellKind::Ice,
    SpellKind::Mist,
    SpellKind::Slow,
    SpellKind::Mystery,
    SpellKind::Bubble,
    SpellKind::Grassy,
    SpellKind::Ghost,
];

fn toggle_panel(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    mut state: ResMut<PanelState>,
    mut cursors: Query<&mut bevy::window::CursorOptions>,
) {
    if keys.is_some_and(|k| k.just_pressed(KeyCode::F1)) {
        state.open = !state.open;
        // The pointer hides in play; the panel needs it.
        for mut cursor in &mut cursors {
            cursor.visible |= state.open;
        }
    }
}

fn draw_panel(
    mut contexts: EguiContexts,
    mut state: ResMut<PanelState>,
    mut queue: ResMut<DebugQueue>,
    game: Res<Match>,
    config: Res<GameConfig>,
    towers: Query<(&Tower, &TowerState)>,
) -> Result {
    if !state.open {
        return Ok(());
    }
    let ctx = contexts.ctx_mut()?;
    let mut send = |command: DebugCommand| queue.0.push(command);
    egui::Window::new("Debug (F1)").show(ctx, |ui| {
        ui.label(format!(
            "{:?}  {}  {:.1} s  round {}",
            game.phase,
            game.mode.name(),
            game.elapsed,
            game.round
        ));

        ui.separator();
        ui.horizontal(|ui| {
            ui.label("Difficulty");
            for d in Difficulty::ALL {
                ui.selectable_value(&mut state.difficulty, d, d.name());
            }
        });
        ui.add(egui::Slider::new(&mut state.opponents, 0..=3).text("AI opponents"));
        ui.horizontal_wrapped(|ui| {
            for mode in ModeKind::ALL {
                if ui.button(mode.name()).clicked() {
                    send(DebugCommand::Start {
                        mode,
                        difficulty: Some(state.difficulty),
                        players: Some(1),
                        opponents: Some(state.opponents),
                        trial: None,
                        cup: None,
                        pack: None,
                    });
                }
            }
        });

        ui.separator();
        ui.add(egui::Slider::new(&mut state.tower, 0..=3).text("Tower"));
        egui::ComboBox::from_label("Spell")
            .selected_text(state.spell.name())
            .show_ui(ui, |ui| {
                for spell in SPELLS {
                    ui.selectable_value(&mut state.spell, spell, spell.name());
                }
            });
        let (tower, spell) = (state.tower, state.spell);
        ui.horizontal_wrapped(|ui| {
            if ui.button("Give light").clicked() {
                send(DebugCommand::GiveLight { tower, spell });
            }
            if ui.button("Give dark").clicked() {
                send(DebugCommand::GiveDark {
                    tower,
                    spells: vec![spell],
                });
            }
            if ui.button("Cast light").clicked() {
                send(DebugCommand::Press {
                    tower,
                    key: Key::CastLight,
                });
            }
            if ui.button("Cast dark").clicked() {
                send(DebugCommand::Press {
                    tower,
                    key: Key::CastDark,
                });
            }
            if ui.button("Curse everyone").clicked() {
                send(DebugCommand::Curse {
                    spells: vec![spell],
                });
            }
        });
        ui.horizontal(|ui| {
            ui.add(egui::Slider::new(&mut state.lives, 0..=9).text("Lives"));
            if ui.button("Set").clicked() {
                send(DebugCommand::SetLives {
                    tower,
                    lives: state.lives,
                });
            }
        });
        ui.horizontal(|ui| {
            if ui.button("Win").clicked() {
                send(DebugCommand::Win { tower });
            }
            if ui.button("Lose").clicked() {
                send(DebugCommand::Lose { tower });
            }
        });

        ui.separator();
        let mut sorted: Vec<_> = towers.iter().collect();
        sorted.sort_by_key(|(t, _)| t.index);
        for (tower, s) in sorted {
            ui.label(format!(
                "#{} {}  {:?}  height {:.1}  placed {}  dropped {}  lives {:?}  light {:?}  dark {:?}",
                tower.index,
                if tower.human { "human" } else { "AI" },
                s.status,
                s.height,
                s.bricks_placed,
                s.bricks_dropped,
                s.lives,
                s.light_spell,
                s.dark_spell.as_ref().map(|c| c.name()),
            ));
        }
        if let Some(i) = config.trial {
            ui.label(format!("trial {}", i + 1));
        }
    });
    Ok(())
}
