//! Placeholder visuals: coloured squares, a camera that follows the tower, and
//! a text HUD. No original game art is used; this is the layer an asset mod (or
//! a loader for your own installed copy's assets) would replace.

use bevy::prelude::*;

use crate::config::{GameConfig, ModeConfig};
use crate::gameplay::{ActiveBrick, BrickBag, Brick, Phase, Platform, Round};

pub struct VisualsPlugin;

impl Plugin for VisualsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ClearColor(Color::srgb(0.12, 0.14, 0.22)))
            .add_systems(Startup, setup_scene)
            .add_systems(
                Update,
                (dress_bricks, dress_platform, follow_tower, update_hud, update_drop_ghost, update_next_preview),
            );
    }
}

#[derive(Component)]
struct Hud;

/// Translucent copy of the active brick drawn where it would land.
#[derive(Component, Default)]
struct DropGhost {
    /// The brick the ghost's cells were built for.
    brick: Option<Entity>,
}

/// UI panel showing the next brick from the bag.
#[derive(Component, Default)]
struct NextPreview {
    piece: Option<usize>,
}

/// Size of one cell in the next-brick panel, in pixels.
const PREVIEW_CELL: f32 = 18.0;
const PREVIEW_BOX: f32 = PREVIEW_CELL * 5.0;

fn setup_scene(mut commands: Commands, config: Res<GameConfig>) {
    commands.spawn((Camera2d, Transform::from_xyz(0.0, config.cells(6.0), 0.0)));

    if let ModeConfig::Race { finish_height, .. } = config.mode {
        commands.spawn((
            Sprite::from_color(
                Color::srgba(1.0, 1.0, 1.0, 0.5),
                Vec2::new(config.cells(config.platform_width + 8.0), 2.0),
            ),
            Transform::from_xyz(0.0, config.cells(finish_height), -1.0),
        ));
    }

    commands.spawn((
        Hud,
        Text::new(""),
        TextFont {
            font_size: FontSize::Px(20.0),
            ..default()
        },
        Node {
            position_type: PositionType::Absolute,
            top: Val::Px(12.0),
            left: Val::Px(12.0),
            ..default()
        },
    ));

    commands.spawn((DropGhost::default(), Transform::default(), Visibility::Hidden));

    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                right: Val::Px(12.0),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.35)),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new("NEXT"),
                TextFont {
                    font_size: FontSize::Px(16.0),
                    ..default()
                },
                Node {
                    margin: UiRect::top(Val::Px(4.0)),
                    ..default()
                },
            ));
            panel.spawn((
                NextPreview::default(),
                Node {
                    width: Val::Px(PREVIEW_BOX),
                    height: Val::Px(PREVIEW_BOX),
                    ..default()
                },
            ));
        });
}

/// Gives every new brick one sprite per cell.
fn dress_bricks(mut commands: Commands, config: Res<GameConfig>, bricks: Query<(Entity, &Brick), Added<Brick>>) {
    let c = config.cell_size;
    for (entity, brick) in &bricks {
        commands.entity(entity).insert(Visibility::default()).with_children(|parent| {
            for &cell in &brick.cells {
                parent.spawn((
                    Sprite::from_color(brick.color, Vec2::splat(c * 0.94)),
                    Transform::from_translation((cell * c).extend(0.0)),
                ));
            }
        });
    }
}

fn dress_platform(mut commands: Commands, config: Res<GameConfig>, platforms: Query<Entity, Added<Platform>>) {
    for entity in &platforms {
        commands.entity(entity).insert(Sprite::from_color(
            Color::srgb(0.55, 0.50, 0.45),
            Vec2::new(config.cells(config.platform_width), config.cell_size),
        ));
    }
}

/// Keeps the top of the tower in view.
fn follow_tower(
    time: Res<Time>,
    config: Res<GameConfig>,
    round: Option<Res<Round>>,
    mut camera: Query<&mut Transform, With<Camera2d>>,
) {
    let (Some(round), Ok(mut transform)) = (round, camera.single_mut()) else {
        return;
    };
    let target = config.cells(round.tower_height.max(6.0) + 2.0);
    let t = 1.0 - (-3.0 * time.delta_secs()).exp();
    transform.translation.y += (target - transform.translation.y) * t;
}

fn update_hud(
    config: Res<GameConfig>,
    round: Option<Res<Round>>,
    mut hud: Query<&mut Text, With<Hud>>,
) {
    let (Some(round), Ok(mut text)) = (round, hud.single_mut()) else {
        return;
    };
    let goal = match config.mode {
        ModeConfig::Race {
            finish_height,
            hold_seconds,
        } => {
            let hold = if round.hold_timer > 0.0 {
                format!("  hold {:.1}/{hold_seconds:.0}s", round.hold_timer)
            } else {
                String::new()
            };
            format!("RACE  height {:.1} / {finish_height:.0}{hold}", round.tower_height)
        }
        ModeConfig::Survival { bricks_to_place, .. } => format!(
            "SURVIVAL  bricks {} / {bricks_to_place}  lives {}",
            round.bricks_placed,
            round.lives.unwrap_or(0)
        ),
    };
    let status = match round.phase {
        Phase::Playing => String::new(),
        Phase::Won => "\nYOU WIN! Press R to play again".into(),
        Phase::Lost => "\nTOWER FELL. Press R to try again".into(),
    };
    text.0 = format!(
        "{goal}\nplaced {}  dropped {}\nLeft/Right move  Up/Space rotate  Down drop faster  R restart{status}",
        round.bricks_placed, round.bricks_dropped
    );
}

/// Moves the ghost to the active brick's landing spot, rebuilding its cells
/// whenever a new brick takes control.
fn update_drop_ghost(
    mut commands: Commands,
    config: Res<GameConfig>,
    active: Query<(Entity, &Brick, &ActiveBrick, &Transform), Without<DropGhost>>,
    mut ghost: Query<(Entity, &mut DropGhost, &mut Transform, &mut Visibility)>,
) {
    let Ok((ghost_entity, mut ghost, mut ghost_transform, mut visibility)) = ghost.single_mut() else {
        return;
    };
    let Ok((entity, brick, control, transform)) = active.single() else {
        *visibility = Visibility::Hidden;
        return;
    };
    if ghost.brick != Some(entity) {
        ghost.brick = Some(entity);
        let c = config.cell_size;
        let color = brick.color.with_alpha(0.25);
        commands
            .entity(ghost_entity)
            .despawn_related::<Children>()
            .with_children(|parent| {
                for &cell in &brick.cells {
                    parent.spawn((
                        Sprite::from_color(color, Vec2::splat(c * 0.94)),
                        Transform::from_translation((cell * c).extend(0.0)),
                    ));
                }
            });
    }
    match control.landing {
        // Hide the ghost once it would overlap the brick itself.
        Some(landing) if transform.translation.y - landing.y > config.cell_size * 0.5 => {
            ghost_transform.translation = landing.extend(-0.5);
            ghost_transform.rotation = transform.rotation;
            *visibility = Visibility::Inherited;
        }
        _ => *visibility = Visibility::Hidden,
    }
}

/// Redraws the next-brick panel when the upcoming piece changes.
fn update_next_preview(
    mut commands: Commands,
    config: Res<GameConfig>,
    bag: Option<ResMut<BrickBag>>,
    mut preview: Query<(Entity, &mut NextPreview)>,
) {
    let (Some(mut bag), Ok((entity, mut preview))) = (bag, preview.single_mut()) else {
        return;
    };
    let next = bag.peek();
    if preview.piece == Some(next) {
        return;
    }
    preview.piece = Some(next);
    let def = &config.pieces[next];
    let cells = def.centered_cells();
    // Shrink big custom pieces so they still fit in the box.
    let extent = cells.iter().fold(0.0_f32, |m, p| m.max(p.x.abs()).max(p.y.abs())) + 0.5;
    let size = PREVIEW_CELL.min(PREVIEW_BOX * 0.45 / extent);
    let color = Color::srgb(def.color[0], def.color[1], def.color[2]);
    commands
        .entity(entity)
        .despawn_related::<Children>()
        .with_children(|parent| {
            for cell in cells {
                // UI y grows downwards.
                let left = PREVIEW_BOX * 0.5 + (cell.x - 0.5) * size;
                let top = PREVIEW_BOX * 0.5 - (cell.y + 0.5) * size;
                parent.spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        left: Val::Px(left + 1.0),
                        top: Val::Px(top + 1.0),
                        width: Val::Px(size - 2.0),
                        height: Val::Px(size - 2.0),
                        ..default()
                    },
                    BackgroundColor(color),
                ));
            }
        });
}
