//! The Trials screen: the list of [`GameConfig::trials`] with the ones beaten
//! marked; Up/Down pick, Enter or A plays, Esc or B goes back to Single Player.
//! The layout is a plain list until the original's Trials screen is read (?).

use bevy::prelude::*;

use crate::art::Art;
use crate::config::{Difficulty, GameConfig};
use crate::gameplay::{Match, MatchRequest, Phase};
use crate::menu::MenuScreen;
use crate::select::{label, place, read_taps, shown, spawn_page};
use crate::trials::TrialProgress;

/// Rows shown at once; the list scrolls to keep the picked one in view.
const ROWS: usize = 10;
const ROW: (f32, f32) = (900.0, 64.0);
const ROW_TOP: f32 = 300.0;
const ROW_STEP: f32 = 72.0;

#[derive(Resource, Debug, Default)]
pub struct TrialsMenu {
    pub selected: usize,
}

#[derive(Component)]
pub(crate) struct TrialsRoot;

/// One row of the list: its slot on screen.
#[derive(Component)]
pub(crate) struct TrialRow(usize);

#[derive(Component)]
pub(crate) struct TrialRowText(usize);

pub(crate) fn spawn_trials_screen(mut commands: Commands, art: Option<Res<Art>>) {
    let page = spawn_page(&mut commands, art.as_deref(), "Trials");
    commands
        .entity(page)
        .insert(TrialsRoot)
        .with_children(|page| {
            for slot in 0..ROWS {
                page.spawn((
                    place(ROW, (0.0, ROW_TOP - ROW_STEP * slot as f32)),
                    BackgroundColor(Color::srgba(0.3, 0.24, 0.55, 0.8)),
                    TrialRow(slot),
                ))
                .with_children(|row| {
                    row.spawn((label(3.2, ""), TrialRowText(slot)));
                });
            }
        });
}

/// First trial shown in the list, so the picked one stays in view.
fn first_shown(selected: usize, count: usize) -> usize {
    selected
        .saturating_sub(ROWS / 2)
        .min(count.saturating_sub(ROWS))
}

pub(crate) fn trials_input(
    keys: Option<Res<ButtonInput<KeyCode>>>,
    pads: Query<(Entity, &Gamepad)>,
    game: Res<Match>,
    mut screen: ResMut<MenuScreen>,
    mut menu: ResMut<TrialsMenu>,
    mut config: ResMut<GameConfig>,
    mut request: ResMut<MatchRequest>,
) {
    if game.phase != Phase::Menu || *screen != MenuScreen::Trials {
        return;
    }
    let taps = read_taps(keys.as_deref(), &pads);
    if !taps.pad_back.is_empty() {
        *screen = MenuScreen::Single;
        return;
    }
    let count = config.trials.len();
    if count == 0 {
        return;
    }
    if taps.up {
        menu.selected = (menu.selected + count - 1) % count;
    }
    if taps.down {
        menu.selected = (menu.selected + 1) % count;
    }
    if taps.key_pick || !taps.pad_pick.is_empty() {
        menu.selected = menu.selected.min(count - 1);
        config.trial = Some(menu.selected);
        // A trial's own values go over Normal's (?).
        config.difficulty = Some(Difficulty::Normal);
        request.pending = true;
    }
}

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub(crate) fn show_trials_screen(
    game: Res<Match>,
    screen: Res<MenuScreen>,
    menu: Res<TrialsMenu>,
    config: Res<GameConfig>,
    progress: Res<TrialProgress>,
    mut root: Query<&mut Visibility, With<TrialsRoot>>,
    mut rows: Query<(&TrialRow, &mut BackgroundColor, &mut Visibility), Without<TrialsRoot>>,
    mut texts: Query<(&TrialRowText, &mut Text)>,
) {
    let on = game.phase == Phase::Menu && *screen == MenuScreen::Trials;
    for mut v in &mut root {
        v.set_if_neq(shown(on));
    }
    if !on {
        return;
    }
    let count = config.trials.len();
    let first = first_shown(menu.selected, count);
    for (row, mut color, mut visibility) in &mut rows {
        let index = first + row.0;
        visibility.set_if_neq(shown(index < count));
        let picked = index == menu.selected;
        color.0 = if picked {
            Color::srgb(0.55, 0.42, 0.9)
        } else {
            Color::srgba(0.3, 0.24, 0.55, 0.8)
        };
    }
    for (text, mut words) in &mut texts {
        let index = first + text.0;
        let line = config.trials.get(index).map_or(String::new(), |trial| {
            let done = if progress.is_done(&trial.name) {
                "  - done"
            } else {
                ""
            };
            format!(
                "{}. {}  ({}){done}",
                index + 1,
                trial.name,
                trial.mode.name()
            )
        });
        if words.0 != line {
            words.0 = line;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_scrolls_to_keep_the_pick_in_view() {
        assert_eq!(first_shown(0, 50), 0);
        assert_eq!(first_shown(4, 50), 0);
        assert_eq!(first_shown(20, 50), 15);
        assert_eq!(first_shown(49, 50), 40);
        assert_eq!(first_shown(2, 3), 0);
    }
}
