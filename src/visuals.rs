//! Placeholder visuals: coloured squares, goal lines, a drop guide, a preview
//! of the next brick, a camera that keeps every tower in view, and a text HUD.
//! No original game art is used; this is the layer an asset mod (or a loader
//! for your own installed copy's assets) would replace.

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::config::{GameConfig, ModeKind};
use crate::gameplay::{
    ActiveBrick, Brick, BrickBag, Match, Phase, Platform, Tower, TowerState, TowerStatus,
};
use crate::spells::{SpellLog, Stone, Vined};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (
                    dress_bricks,
                    tint_stone,
                    tint_vines,
                    dress_platforms,
                    update_drop_guide,
                    update_next_preview,
                    frame_towers,
                    update_hud,
                ),
            );
    }
}

#[derive(Component)]
struct Hud;

#[derive(Component)]
struct DropGuide;

#[derive(Component)]
struct NextPreview {
    piece: Option<usize>,
}

/// Child sprite of a brick, one per cell.
#[derive(Component)]
struct CellSprite;

fn setup_scene(mut commands: Commands, config: Res<GameConfig>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, config.cells(6.0), 0.0)));
    commands.spawn((
        DropGuide,
        Sprite::from_color(Color::srgba(0.7, 0.85, 1.0, 0.035), Vec2::ONE),
        Transform::from_xyz(0.0, 0.0, -2.0),
        Visibility::Hidden,
    ));
    commands.spawn((
        NextPreview { piece: None },
        Transform::default(),
        Visibility::Hidden,
    ));
    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(18.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));
}

fn cell_sprites(parent: &mut ChildSpawnerCommands, cells: &[Vec2], cell: f32, color: Color) {
    for &p in cells {
        parent.spawn((
            CellSprite,
            Sprite::from_color(color, Vec2::splat(cell * 0.94)),
            Transform::from_translation((p * cell).extend(0.0)),
        ));
    }
}

/// Gives every new brick one sprite per cell.
fn dress_bricks(
    mut commands: Commands,
    config: Res<GameConfig>,
    bricks: Query<(Entity, &Brick), Added<Brick>>,
) {
    for (entity, brick) in &bricks {
        let cell = config.cell_size * brick.scale;
        commands
            .entity(entity)
            .insert(Visibility::default())
            .with_children(|parent| cell_sprites(parent, &brick.cells, cell, brick.color));
    }
}

fn recolor(
    children: &Children,
    sprites: &mut Query<&mut Sprite, With<CellSprite>>,
    f: impl Fn(Color) -> Color,
) {
    for &child in children {
        if let Ok(mut sprite) = sprites.get_mut(child) {
            sprite.color = f(sprite.color);
        }
    }
}

fn tint_stone(
    bricks: Query<&Children, Added<Stone>>,
    mut sprites: Query<&mut Sprite, With<CellSprite>>,
) {
    for children in &bricks {
        recolor(children, &mut sprites, |c| {
            c.mix(&Color::srgb(0.45, 0.45, 0.48), 0.75)
        });
    }
}

fn tint_vines(
    bricks: Query<&Children, Added<Vined>>,
    mut sprites: Query<&mut Sprite, With<CellSprite>>,
) {
    for children in &bricks {
        recolor(children, &mut sprites, |c| {
            c.mix(&Color::srgb(0.2, 0.7, 0.25), 0.4)
        });
    }
}

/// Draws each new platform and its goal line (race finish or puzzle laser).
fn dress_platforms(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    platforms: Query<Entity, Added<Platform>>,
) {
    let c = config.cell_size;
    for entity in &platforms {
        commands
            .entity(entity)
            .insert(Sprite::from_color(
                Color::srgb(0.55, 0.50, 0.45),
                Vec2::new(config.cells(config.platform_width), c),
            ))
            .with_children(|parent| {
                let line = match game.mode {
                    ModeKind::Race => {
                        Some((config.race.finish_height, Color::srgba(1.0, 1.0, 1.0, 0.6)))
                    }
                    ModeKind::Puzzle => {
                        Some((config.puzzle.line_height, Color::srgba(1.0, 0.2, 0.2, 0.8)))
                    }
                    ModeKind::Survival => None,
                };
                if let Some((height, color)) = line {
                    // The platform's centre is half a cell below its top.
                    parent.spawn((
                        Sprite::from_color(
                            color,
                            Vec2::new(config.cells(config.platform_width + 4.0), 3.0),
                        ),
                        Transform::from_xyz(0.0, config.cells(height) + c * 0.5, -1.0),
                    ));
                }
            });
    }
}

/// A faint beam under your falling brick showing where it will land.
fn update_drop_guide(
    towers: Query<(&Tower, &TowerState)>,
    bricks: Query<(&ActiveBrick, &ColliderAabb)>,
    mut guide: Query<(&mut Transform, &mut Sprite, &mut Visibility), With<DropGuide>>,
) {
    let Ok((mut transform, mut sprite, mut visibility)) = guide.single_mut() else {
        return;
    };
    let active = towers
        .iter()
        .find(|(tower, _)| tower.human)
        .and_then(|(_, state)| state.active)
        .and_then(|entity| bricks.get(entity).ok());
    let Some((active, aabb)) = active else {
        *visibility = Visibility::Hidden;
        return;
    };
    let Some(distance) = active.drop_distance else {
        *visibility = Visibility::Hidden;
        return;
    };
    let width = aabb.max.x - aabb.min.x;
    let x = (aabb.min.x + aabb.max.x) * 0.5;
    transform.translation = Vec3::new(x, aabb.min.y - distance * 0.5, -2.0);
    sprite.custom_size = Some(Vec2::new(width, distance.max(1.0)));
    *visibility = Visibility::Visible;
}

/// Shows your next brick beside your spawn point.
fn update_next_preview(
    mut commands: Commands,
    config: Res<GameConfig>,
    game: Res<Match>,
    mut towers: Query<(&Tower, &TowerState, &mut BrickBag)>,
    mut preview: Query<(Entity, &mut NextPreview, &mut Transform, &mut Visibility)>,
) {
    let Ok((entity, mut preview, mut transform, mut visibility)) = preview.single_mut() else {
        return;
    };
    let Some((tower, state, mut bag)) = towers.iter_mut().find(|(t, _, _)| t.human) else {
        *visibility = Visibility::Hidden;
        return;
    };
    let finished_puzzle = game.mode == ModeKind::Puzzle
        && state.bricks_placed + state.bricks_dropped + 1 >= config.puzzle.bricks;
    if game.phase != Phase::Playing || finished_puzzle {
        *visibility = Visibility::Hidden;
        return;
    }
    *visibility = Visibility::Visible;
    let floor = if game.mode == ModeKind::Puzzle {
        state.stack_top.max(config.puzzle.line_height)
    } else {
        state.stack_top
    };
    transform.translation = Vec3::new(
        tower.origin_x + config.cells(config.platform_width * 0.5 + 3.0),
        config.cells(floor + config.spawn_gap),
        0.0,
    );
    let next = bag.peek();
    if preview.piece == Some(next) {
        return;
    }
    preview.piece = Some(next);
    let def = &config.pieces[next];
    let color = Color::srgba(def.color[0], def.color[1], def.color[2], 0.6);
    let cells = def.centered_cells();
    commands
        .entity(entity)
        .despawn_children()
        .with_children(|parent| cell_sprites(parent, &cells, config.cell_size * 0.6, color));
}

/// Keeps every tower, from the platforms to the top of the highest one, in view.
fn frame_towers(
    time: Res<Time>,
    config: Res<GameConfig>,
    game: Res<Match>,
    windows: Query<&Window>,
    towers: Query<(&Tower, &TowerState)>,
    mut camera: Query<(&mut Transform, &mut Projection), With<Camera2d>>,
) {
    let (Ok(window), Ok((mut transform, mut projection))) = (windows.single(), camera.single_mut())
    else {
        return;
    };
    if towers.is_empty() {
        return;
    }
    let (mut min_x, mut max_x, mut top) = (f32::MAX, f32::MIN, 0.0f32);
    for (tower, state) in &towers {
        min_x = min_x.min(tower.origin_x);
        max_x = max_x.max(tower.origin_x);
        top = top.max(state.stack_top);
    }
    let goal = match game.mode {
        ModeKind::Race => config.race.finish_height,
        ModeKind::Puzzle => config.puzzle.line_height,
        ModeKind::Survival => 0.0,
    };
    let c = config.cell_size;
    let view_top = (top.max(goal) + config.spawn_gap + 3.0).max(16.0);
    let height = config.cells(view_top + 3.0);
    let width = max_x - min_x + config.cells(config.platform_width + 10.0);
    let scale = (width / window.width())
        .max(height / window.height())
        .max(0.5);
    let target = Vec2::new((min_x + max_x) * 0.5, height * 0.5 - c * 3.0);

    let t = 1.0 - (-3.0 * time.delta_secs()).exp();
    let current = transform.translation.truncate();
    transform.translation += ((target - current) * t).extend(0.0);
    if let Projection::Orthographic(ortho) = &mut *projection {
        ortho.scale += (scale - ortho.scale) * t;
    }
}

fn update_hud(
    config: Res<GameConfig>,
    game: Res<Match>,
    log: Res<SpellLog>,
    towers: Query<(&Tower, &TowerState)>,
    mut hud: Query<&mut Text, With<Hud>>,
) {
    let Ok(mut text) = hud.single_mut() else {
        return;
    };
    let mut sorted: Vec<_> = towers.iter().collect();
    sorted.sort_by_key(|(t, _)| t.index);
    let Some((_, you)) = sorted.iter().find(|(t, _)| t.human) else {
        text.0.clear();
        return;
    };

    let goal = match game.mode {
        ModeKind::Race => {
            let hold = if you.hold_timer > 0.0 {
                format!(
                    "  hold {:.1}/{:.0}s",
                    you.hold_timer, config.race.hold_seconds
                )
            } else {
                String::new()
            };
            format!(
                "RACE  height {:.1} / {:.0}{hold}",
                you.height, config.race.finish_height
            )
        }
        ModeKind::Survival => format!(
            "SURVIVAL  bricks {} / {}  lives {}",
            you.bricks_placed,
            config.survival.bricks_to_place,
            you.lives.unwrap_or(0)
        ),
        ModeKind::Puzzle => format!(
            "PUZZLE  bricks {} / {}  stay under the laser ({:.0})",
            you.bricks_placed, config.puzzle.bricks, config.puzzle.line_height
        ),
    };
    let spell = match you.spell {
        Some(s) => format!("spell: {} (E to cast)", s.name()),
        None if config.magic.enabled => format!(
            "spell in {} bricks",
            config
                .magic
                .bricks_per_spell
                .saturating_sub(you.spell_progress)
        ),
        None => String::new(),
    };
    let mut lines = vec![goal, format!("dropped {}   {spell}", you.bricks_dropped)];

    for (tower, state) in sorted.iter().filter(|(t, _)| !t.human) {
        let status = match state.status {
            TowerStatus::Building => String::new(),
            TowerStatus::Won => " (won)".into(),
            TowerStatus::Out => " (out)".into(),
        };
        let lives = state
            .lives
            .map(|l| format!(" lives {l}"))
            .unwrap_or_default();
        lines.push(format!(
            "Wizard {}: height {:.1}{lives}{status}",
            tower.index, state.height
        ));
    }
    if let Some(last) = log.0.last() {
        let who = |i: usize| {
            if i == 0 {
                "You".to_string()
            } else {
                format!("Wizard {i}")
            }
        };
        let on = if last.caster == last.target {
            String::new()
        } else {
            format!(" on {}", who(last.target).to_lowercase())
        };
        lines.push(format!(
            "{} cast {}{on}",
            who(last.caster),
            last.spell.name()
        ));
    }
    lines.push("Left/Right move  Up/Space rotate  Down drop  E spell  R restart  Esc menu".into());
    if game.phase == Phase::Over {
        lines.push(match game.winner {
            Some(0) => "YOU WIN! Press R to play again".into(),
            Some(i) => format!("Wizard {i} wins. Press R to try again"),
            None => match game.mode {
                ModeKind::Puzzle => "PUZZLE FAILED. Press R to try again".into(),
                _ => "OUT OF LIVES. Press R to try again".into(),
            },
        });
    }
    text.0 = lines.join("\n");
}
