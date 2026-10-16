//! DEBUG-TOOL: not in the original; remove before the final build (docs/release-checklist.md).
//!
//! Debug commands: a small set of actions (start a mode, press a button for a tower, give
//! or cast a spell, set lives, win or lose, take a picture, quit) that tests, script
//! files and the debug panel all go through. Push a [`DebugCommand`] into
//! [`DebugQueue`]; it runs at the start of the next fixed step, before gameplay.
//!
//! Script files (`--script file.ron`) are timed lists of commands, so a rare moment
//! can be set up exactly and pictured offscreen:
//! ```ron
//! [
//!     (at: 0.5, run: Start(mode: Race, opponents: Some(1))),
//!     (at: 2.0, run: GiveDark(tower: 0, spells: [Large, Ice])),
//!     (at: 2.1, run: Press(tower: 0, key: CastDark)),
//!     (at: 3.0, run: Picture(name: "large_ice")),
//!     (at: 4.0, run: Quit),
//! ]
//! ```
//!
//! Headless-safe: no rendering or keyboard here. Pictures are taken by the offscreen
//! layer (src/offscreen.rs) when it runs; otherwise `Picture` is ignored.

use bevy::app::AppExit;
use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::config::{Curse, Difficulty, GameConfig, ModeKind, SpellKind};
use crate::gameplay::{
    Brick, GameplaySystems, Match, MatchRequest, PlayerInput, Placed, Tower, TowerSkin,
    TowerState, TowerStatus, brick_body, piece_def,
};
use crate::spells::EnemyCasts;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<DebugQueue>()
            .add_message::<DebugPicture>()
            .add_systems(
                FixedUpdate,
                (run_script, apply_debug_commands)
                    .chain()
                    .before(GameplaySystems),
            );
    }
}

/// A button on a tower's controls, for [`DebugCommand::Press`] and `Hold`.
#[derive(Deserialize, Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    NudgeLeft,
    NudgeRight,
    RotateRight,
    RotateLeft,
    Drop,
    CastLight,
    CastDark,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub enum DebugCommand {
    /// Start (or restart) a match in `mode`; unset fields keep the current config.
    Start {
        mode: ModeKind,
        #[serde(default)]
        difficulty: Option<Difficulty>,
        #[serde(default)]
        players: Option<u32>,
        #[serde(default)]
        opponents: Option<u32>,
        /// Play this trial (an index into the trials) instead.
        #[serde(default)]
        trial: Option<usize>,
        /// Start this cup (0 Tiny, 1 Big, 2 Super) with the match's mode and difficulty.
        #[serde(default)]
        cup: Option<usize>,
        /// DEBUG-TOOL: every tower uses this brick pack (as if picked on Select Players).
        #[serde(default)]
        pack: Option<crate::config::BrickPack>,
    },
    /// DEBUG-TOOL: one static brick of each shape (up to 7), in a row, in `pack`, for side
    /// by side comparison with a reference picture. No physics; spawns in place.
    PackLine {
        pack: crate::config::BrickPack,
    },
    /// DEBUG-TOOL: switch the Galaxy starfield's scroll on or off.
    GalaxyFrame {
        scroll: bool,
    },
    /// Press a button once for tower `tower` (0 is player 1).
    Press {
        tower: usize,
        key: Key,
    },
    /// Hold or let go of Left, Right or Drop.
    Hold {
        tower: usize,
        key: Key,
        on: bool,
    },
    /// Put a light spell in the tower's hand.
    GiveLight {
        tower: usize,
        spell: SpellKind,
    },
    /// Put a dark spell (one or more curses cast together) in the tower's hand.
    GiveDark {
        tower: usize,
        spells: Vec<SpellKind>,
    },
    /// Curse every tower at once, like a trial's enemy wizard.
    Curse {
        spells: Vec<SpellKind>,
    },
    SetLives {
        tower: usize,
        lives: u32,
    },
    /// End the tower's match: it wins, or it is out.
    Win {
        tower: usize,
    },
    Lose {
        tower: usize,
    },
    /// Tap a keyboard key for the menus and popups: `Enter`, `Escape`,
    /// `Tab`, `Space`, `Up`, `Down`, `Left` or `Right`.
    Tap {
        key: String,
    },
    /// DEBUG-TOOL: pick the menu language by its Settings name (`German`, `French`...), for
    /// pictures of the translated menus.
    Language {
        name: String,
    },
    /// Save a picture named `name` (offscreen runs only).
    Picture {
        name: String,
    },
    Quit,
}

/// Commands waiting for the next fixed step.
#[derive(Resource, Debug, Default)]
pub struct DebugQueue(pub Vec<DebugCommand>);

/// One line of a script file: seconds after the game starts, and the command.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq)]
pub struct ScriptStep {
    pub at: f32,
    pub run: DebugCommand,
}

/// A loaded script, run in time order.
#[derive(Resource, Debug, Default)]
pub struct DebugScript {
    pub steps: Vec<ScriptStep>,
    next: usize,
    clock: f32,
}

impl DebugScript {
    pub fn new(mut steps: Vec<ScriptStep>) -> Self {
        steps.sort_by(|a, b| a.at.total_cmp(&b.at));
        Self {
            steps,
            next: 0,
            clock: 0.0,
        }
    }

    /// Reads a script file (a RON list of `(at: seconds, run: Command)`).
    pub fn load(path: &std::path::Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| e.to_string())?;
        let steps: Vec<ScriptStep> = ron::from_str(&text).map_err(|e| e.to_string())?;
        Ok(Self::new(steps))
    }

    pub fn finished(&self) -> bool {
        self.next >= self.steps.len()
    }
}

/// Ask the offscreen layer for a picture.
#[derive(Message, Clone, Debug)]
pub struct DebugPicture(pub String);

fn run_script(time: Res<Time>, script: Option<ResMut<DebugScript>>, mut queue: ResMut<DebugQueue>) {
    let Some(mut script) = script else {
        return;
    };
    script.clock += time.delta_secs();
    while let Some(step) = script.steps.get(script.next) {
        if step.at > script.clock {
            break;
        }
        queue.0.push(step.run.clone());
        script.next += 1;
    }
}

#[allow(clippy::too_many_arguments)]
fn apply_debug_commands(
    mut queue: ResMut<DebugQueue>,
    mut config: ResMut<GameConfig>,
    mut request: ResMut<MatchRequest>,
    mut game: ResMut<Match>,
    mut enemy: ResMut<EnemyCasts>,
    mut towers: Query<(&Tower, &mut TowerState, &mut PlayerInput)>,
    mut pictures: MessageWriter<DebugPicture>,
    mut exit: MessageWriter<AppExit>,
    mut keyboard: Option<ResMut<ButtonInput<KeyCode>>>,
    mut cups: ResMut<crate::cups::Cup>,
    mut galaxy: Option<ResMut<crate::packs::GalaxyDebug>>,
    mut settings: Option<ResMut<crate::settings::Settings>>,
    mut commands: Commands,
) {
    // The tower with this index, as (state, input).
    macro_rules! tower {
        ($index:expr) => {
            towers
                .iter_mut()
                .find(|(t, _, _)| t.index == $index)
                .map(|(_, state, input)| (state, input))
        };
    }
    for command in std::mem::take(&mut queue.0) {
        match command {
            DebugCommand::Start {
                mode,
                difficulty,
                players,
                opponents,
                trial,
                cup,
                pack,
            } => {
                config.mode = mode;
                config.trial = trial;
                if let Some(pack) = pack {
                    config.skins = (0..crate::gameplay::MAX_TOWERS).map(|i| (i, pack)).collect();
                }
                if difficulty.is_some() {
                    config.difficulty = difficulty;
                }
                if let Some(players) = players {
                    config.players = players;
                }
                if let Some(opponents) = opponents {
                    config.opponents = opponents;
                }
                if let Some(size) = cup {
                    cups.pick = Some(size);
                    let pair = (mode, config.difficulty.unwrap_or(Difficulty::Normal));
                    cups.current = Some(pair);
                    cups.options = vec![pair];
                    cups.begin();
                }
                // A match started from a menu leaves it.
                if game.phase == crate::gameplay::Phase::Menu {
                    game.phase = crate::gameplay::Phase::Playing;
                }
                request.pending = true;
            }
            DebugCommand::PackLine { pack } => {
                // Show the row in a match, not over the start or menu screens.
                if game.phase == crate::gameplay::Phase::Menu {
                    game.phase = crate::gameplay::Phase::Playing;
                }
                // Bricks need a tower for their skin; a bare entity carries it here.
                let owner = commands.spawn(TowerSkin { wizard: 0, pack }).id();
                let count = config.pieces.len().min(7);
                for piece in 0..count {
                    let def = piece_def(&config, piece);
                    let brick = Brick {
                        tower: owner,
                        piece,
                        cells: def.centered_cells(),
                        color: Color::srgb(def.color[0], def.color[1], def.color[2]),
                        scale: 1.0,
                        mass: def.mass,
                        ice: false,
                    };
                    let (body, density) = brick_body(&config, &brick);
                    let x = (piece as f32 - (count as f32 - 1.0) / 2.0) * 4.0;
                    commands.spawn((
                        brick,
                        Placed(0),
                        avian2d::prelude::RigidBody::Static,
                        body,
                        density,
                        Transform::from_xyz(config.cells(x), 0.0, 0.0),
                        Visibility::Visible,
                    ));
                }
            }
            DebugCommand::GalaxyFrame { scroll } => {
                if let Some(galaxy) = galaxy.as_mut() {
                    galaxy.scroll = scroll;
                }
            }
            DebugCommand::Press { tower: index, key } => {
                if let Some((_, mut input)) = tower!(index) {
                    match key {
                        Key::Left => input.left_pressed = true,
                        Key::Right => input.right_pressed = true,
                        Key::NudgeLeft => input.nudge_left_pressed = true,
                        Key::NudgeRight => input.nudge_right_pressed = true,
                        Key::RotateRight => input.rotate_pressed = true,
                        Key::RotateLeft => input.rotate_ccw_pressed = true,
                        Key::CastLight => input.cast_light_pressed = true,
                        Key::CastDark => input.cast_dark_pressed = true,
                        Key::Drop => input.fast_fall_held = true,
                    }
                }
            }
            DebugCommand::Hold {
                tower: index,
                key,
                on,
            } => {
                if let Some((_, mut input)) = tower!(index) {
                    match key {
                        Key::Left => input.left_held = on,
                        Key::Right => input.right_held = on,
                        Key::Drop => input.fast_fall_held = on,
                        _ => warn!("debug: only Left, Right and Drop can be held"),
                    }
                }
            }
            DebugCommand::GiveLight {
                tower: index,
                spell,
            } => {
                if let Some((mut state, _)) = tower!(index) {
                    state.light_spell = Some(spell);
                }
            }
            DebugCommand::GiveDark {
                tower: index,
                spells,
            } => {
                if let Some((mut state, _)) = tower!(index) {
                    state.dark_spell = Some(Curse::new(&spells, 1, None));
                }
            }
            DebugCommand::Curse { spells } => enemy.0.push(Curse::new(&spells, 1, None)),
            DebugCommand::SetLives {
                tower: index,
                lives,
            } => {
                if let Some((mut state, _)) = tower!(index) {
                    state.lives = Some(lives);
                }
            }
            DebugCommand::Win { tower: index } => {
                if let Some((mut state, _)) = tower!(index) {
                    state.status = TowerStatus::Won;
                }
            }
            DebugCommand::Lose { tower: index } => {
                if let Some((mut state, _)) = tower!(index) {
                    state.status = TowerStatus::Out;
                }
            }
            DebugCommand::Language { name } => {
                if let Some(settings) = settings.as_mut() {
                    settings.language = name;
                }
            }
            DebugCommand::Tap { key } => {
                let code = match key.as_str() {
                    "Enter" => KeyCode::Enter,
                    "Escape" => KeyCode::Escape,
                    "Tab" => KeyCode::Tab,
                    "Space" => KeyCode::Space,
                    "Up" => KeyCode::ArrowUp,
                    "Down" => KeyCode::ArrowDown,
                    "Left" => KeyCode::ArrowLeft,
                    "Right" => KeyCode::ArrowRight,
                    // The default nudge and magic keys.
                    "U" => KeyCode::KeyU,
                    "I" => KeyCode::KeyI,
                    "J" => KeyCode::KeyJ,
                    "K" => KeyCode::KeyK,
                    _ => {
                        warn!("debug: unknown key {key}");
                        continue;
                    }
                };
                // Pressed and let go at once: just_pressed stays set until the
                // next frame's input update, so the menus see one tap.
                if let Some(keyboard) = keyboard.as_mut() {
                    keyboard.press(code);
                    keyboard.release(code);
                }
            }
            DebugCommand::Picture { name } => {
                pictures.write(DebugPicture(name));
            }
            DebugCommand::Quit => {
                exit.write(AppExit::Success);
            }
        }
    }
}
