use std::f32::consts::PI;

use crate::board::PieceModel::*;
use crate::board::*;
use crate::logic::*;
use crate::sets::*;
use crate::turns::*;
use bevy::{prelude::*, transform::TransformSystem, window::WindowResized};

const BG_TEX_SIZE: Vec2 = Vec2::new(2560., 1587.);
const PIECE_TEX_SIZE: f32 = 256.;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BoardDisplayState::default())
            .insert_resource(Selections::default())
            .add_systems(Startup, init_ui.in_set(GameSet::UISetup))
            .add_systems(PostUpdate,
                update_transform_cache
                    .after(TransformSystem::TransformPropagate),
            )
            .add_systems(Update, (
                (
                    move_piece,
                    update_board_display
                ).chain(),
                update_playground_transform
            ));
    }
}

#[derive(Component)]
struct Background;

#[derive(Component)]
struct Playground;

#[derive(Component)]
struct Square;

#[derive(Component)]
struct Marker;

#[derive(Component)]
struct PromotionPopup;

#[derive(Component, Deref, DerefMut)]
struct PromotionChoice(PieceModel);

// Cache of a Global Transform's inverse matrix
// Updated automatically through change detection
// Used to compute where the cursor is relative to a transformed entity
#[derive(Component)]
struct InverseGTransformCache {
    matrix: Mat4,
}

impl Default for InverseGTransformCache {
    fn default() -> Self {
        Self {
            matrix: Mat4::IDENTITY,
        }
    }
}

#[derive(Resource, Default)]
pub struct BoardDisplayState {
    pub displayed_turn: usize,
    pub bottom_side: Side,
}

#[derive(Resource, Default)]
struct Selections {
    pub piece: Option<Entity>,
    pub promotion: Option<Move>
}

#[derive(Resource)]
struct Textures {
    pub pieces: Handle<Image>,
    pub pieces_tal: Handle<TextureAtlasLayout>,
    pub marker: Handle<Image>,
    pub promotion_popup: Handle<Image>
}

fn init_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlasLayout>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(Textures {
        pieces: asset_server.load("pieces.png"),
        pieces_tal: atlases.add(TextureAtlasLayout::from_grid(
            Vec2::splat(PIECE_TEX_SIZE),
            6,
            2,
            None,
            None
        )),
        marker: asset_server.load("marker.png"),
        promotion_popup: asset_server.load("promotion_popup.png")
    });

    commands.spawn((
        Background,
        SpriteBundle {
            texture: asset_server.load("bg.webp"),
            ..default()
        },
    ));

    commands.spawn((
        Playground,
        TransformBundle::default(),
        InverseGTransformCache::default(),
        VisibilityBundle::default(),
    ));
}

fn update_board_display(
    mut commands: Commands,
    mut set: ParamSet<(
        Query<Entity, With<Playground>>,
        Query<Entity, Or<(With<Piece>, With<Square>)>>
    )>,
    turns: Res<Turns>,
    display_state: Res<BoardDisplayState>,
    textures: Res<Textures>
) {
    if !display_state.is_changed() { return; }

    let Some(Turn { board, .. }) = turns.history.get(display_state.displayed_turn)
    else { eprintln!("update_board_display: can't find board to display"); return };

    let Ok(pg_entity) = set.p0().get_single()
    else { eprintln!("update_board_display: no spawned playground"); return };

    for old_entity in set.p1().iter_mut() {
        if let Some(mut ec) = commands.get_entity(old_entity) {
            ec.despawn()
        }
    }

    commands.entity(pg_entity).with_children(|parent| {
        for ((x, y), space) in board.spaces.indexed_iter() {
            if *space != Space::Hole {
                parent.spawn((
                    Square,
                    SpriteBundle {
                        sprite: Sprite {
                            color: if (x + y) % 2 == 0 {
                                Color::rgb(0.2, 0.3, 0.4)
                            } else {
                                Color::rgb(0.8, 0.8, 0.8)
                            },
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(x as f32, y as f32, 1.)),
                        ..default()
                    }
                ));
            }
    
            if let Space::Square { slot: Some(piece), .. } = space
            {
                parent.spawn((
                    *piece,
                    Coords {
                        x: x as isize,
                        y: y as isize,
                    },
                    SpriteSheetBundle {
                        texture: textures.pieces.clone(),
                        atlas: TextureAtlas { layout: textures.pieces_tal.clone(), index: piece.texture_index() },
                        sprite: Sprite {
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            x as f32, y as f32, 2.,
                        )).with_rotation(Quat::from_rotation_z(
                            match display_state.bottom_side {
                                Side::White => 0.,
                                Side::Black => PI
                            }
                        )),
                        ..default()
                    },
                ));
            }
        }
    });
}

fn move_piece(
    mut commands: Commands,
    mut turns: ResMut<Turns>,
    mut display_state: ResMut<BoardDisplayState>,
    mut selections: ResMut<Selections>,
    mut displayed_pieces: Query<(Entity, &Piece, &mut Transform, &Coords), Without<PromotionChoice>>,
    windows: Query<&Window>,
    buttons: Res<ButtonInput<MouseButton>>,
    playground: Query<(Entity, &InverseGTransformCache), With<Playground>>,
    markers: Query<Entity, With<Marker>>,
    textures: Res<Textures>,
    promotion_choices: Query<(&PromotionChoice, &Transform)>,
    promotion_graphics: Query<Entity, Or<(With<PromotionPopup>, With<PromotionChoice>)>>
) {
    let Some(displayed_turn @ Turn { board: displayed_board, .. }) = turns.history.get(display_state.displayed_turn)
    else { eprintln!("select_piece: could not find current turn"); return };

    let Ok((pg_entity, InverseGTransformCache { matrix: pg_inv_matrix })) = playground.get_single() else { return };
    
    let mouse_pos = {
        let Ok(window) = windows.get_single() else { eprintln!("select_piece: Could not fetch window"); return };
        let Some(mut pos) = window.cursor_position() else { return };
        pos.x -= window.width() / 2.;
        pos.y = window.height() / 2. - pos.y;

        pg_inv_matrix.transform_point3(pos.extend(0.))
    };

    let mouse_coords = Coords {
        x: mouse_pos.x.round() as isize,
        y: mouse_pos.y.round() as isize,
    };


    if let Some(ref mut prom_move) = selections.promotion {
        if buttons.just_released(MouseButton::Left) {
            for (PromotionChoice(model), choice_transform) in promotion_choices.iter() {
                if Vec2::distance(choice_transform.translation.truncate(),  mouse_pos.truncate()) < 0.5 {
                    prom_move.promotion = Some(*model);

                    let new_board = get_next_board(displayed_board, prom_move);

                    let new_turn = Turn {
                        previous_move: *prom_move,
                        possible_moves: compute_possible_moves(&new_board, true),
                        board: new_board
                    };
                    
                    turns.history.truncate(display_state.displayed_turn + 1);

                    turns.history.push_back(new_turn);

                    display_state.displayed_turn += 1;
                    
                    for entity in promotion_graphics.iter() {
                        commands.entity(entity).despawn();
                    }

                    selections.promotion = None;
                    
                    break;
                }
            }
        }
    }
    else if buttons.just_pressed(MouseButton::Left) {
        // Get the movable piece at mouse position if it exists
        if let Some((piece_entity, piece, mut piece_transform, piece_coords)) = displayed_pieces
            .iter_mut()
            .find(|(_, piece, _, &piece_coords)|
                mouse_coords == piece_coords && piece.side == displayed_board.side
            )
        {
            // Make it the currently selected piece
            selections.piece = Some(piece_entity);

            // Snap the piece to mouse position
            piece_transform.translation = mouse_pos.truncate().extend(3.);
            piece_transform.scale = Vec2::splat(1.2).extend(1.);

            // Display possible move markers
            if let Some(moves) = displayed_turn.possible_moves.get(piece_coords) {
                commands.entity(pg_entity).with_children(|parent| {
                    for move_ in moves {
                        parent.spawn((
                            Marker,
                            SpriteBundle {
                                sprite: Sprite {
                                    color: Color::rgb(0.2, 0.6, 0.3),
                                    custom_size: Some(Vec2::ONE),
                                    ..default()
                                },
                                transform: Transform::from_translation(Vec3::new(
                                    move_.target.x as f32,
                                    move_.target.y as f32,
                                    1.,
                                )),
                                texture: textures.marker.clone(),
                                ..default()
                            },
                        ));
                    }
                });
            }

            // Display promotion squares markers
            if let Piece { model: Pawn { .. }, side } = piece {
                commands.entity(pg_entity).with_children(|parent| {
                    for ((x, y), space) in displayed_turn.board.spaces.indexed_iter() {
                        if let Space::Square { promotes, .. } = space {
                            if promotes[*side as usize] {
                                parent.spawn((
                                    Marker,
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::rgb(0.8, 0.6, 0.),
                                            custom_size: Some(Vec2::ONE),
                                            ..default()
                                        },
                                        transform: Transform::from_translation(Vec3::new(
                                            x as f32, y as f32, 1.,
                                        )),
                                        texture: textures.marker.clone(),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    }
                });
            }
        }
    } else if let Some(piece_entity) = selections.piece {
        // A piece is currently grabbed
        if buttons.pressed(MouseButton::Left) {
            if let Ok((_, _, mut piece_transform, _)) = displayed_pieces.get_mut(piece_entity) {
                // Update its position to the mouse's
                piece_transform.translation = mouse_pos.truncate().extend(3.);
            }
        }
        // A piece is being released
        else if buttons.just_released(MouseButton::Left) {
            // Reset piece position
            if let Ok((_, _, mut piece_transform, piece_coords)) = displayed_pieces.get_mut(piece_entity) {
                piece_transform.translation = Vec3::new(piece_coords.x as f32, piece_coords.y as f32, 2.);
                piece_transform.scale = Vec3::ONE;
            }

            // Reset selection
            selections.piece = None;

            // Stop displaying move markers
            markers.iter().for_each(|marker_entity| {
                if let Some(mut ec) = commands.get_entity(marker_entity) {
                    ec.despawn()
                }
            });

            if let Ok((_, piece, _, piece_coords)) = displayed_pieces.get(piece_entity) {
                if let Some(piece_moves) = displayed_turn.possible_moves.get(piece_coords) {
                    // In the case of a promotion, there are multiple selected moves
                    let selected_moves: Vec<_> = piece_moves
                        .iter()
                        .filter(|move_| {
                            move_.target == mouse_coords
                        })
                        .collect();
                    
                    if !selected_moves.is_empty() {
                        // Put up a popup for promotions
                        if selected_moves.iter().all(|move_| move_.promotion.is_some()) {
                            selections.promotion = Some(Move {
                                promotion: None,
                                ..**selected_moves.first().unwrap()
                            });

                            commands.entity(piece_entity).despawn();
                            
                            commands.entity(pg_entity).with_children(|parent| {
                                let target = selected_moves.first().unwrap().target;

                                parent.spawn((
                                    PromotionPopup,
                                    SpriteBundle {
                                        sprite: Sprite {
                                            color: Color::rgb(0.82, 0.63, 0.51),
                                            custom_size: Some(Vec2 { x: 4.078, y: 1.078 }),
                                            ..default()
                                        },
                                        transform: Transform::from_translation(Vec3::new(
                                            target.x as f32, target.y as f32, 3.,
                                        )),
                                        texture: textures.promotion_popup.clone(),
                                        ..default()
                                    },
                                ));
                                
                                let mut choice_x = target.x as f32 - (selected_moves.len() - 1) as f32 / 2.;

                                for move_ in selected_moves {
                                    let model = move_.promotion.unwrap();

                                    parent.spawn((
                                        PromotionChoice(model),
                                        SpriteSheetBundle {
                                            texture: textures.pieces.clone(),
                                            atlas: TextureAtlas {
                                                layout: textures.pieces_tal.clone(),
                                                index: Piece {
                                                    model,
                                                    side: piece.side
                                                }.texture_index()
                                            },
                                            sprite: Sprite {
                                                custom_size: Some(Vec2::ONE),
                                                ..default()
                                            },
                                            transform: Transform::from_translation(Vec3::new(
                                                choice_x, target.y as f32, 4.,
                                            )).with_rotation(Quat::from_rotation_z(
                                                match display_state.bottom_side {
                                                    Side::White => 0.,
                                                    Side::Black => PI
                                                }
                                            )),
                                            ..default()
                                        },
                                    ));

                                    choice_x += 1.0;
                                }

                            });
                        }
                        // Add a turn to the turn history if a valid move has been played
                        // All graphical updates will be handled later by update_board_display
                        else if selected_moves.len() == 1 {
                            let selected_move = selected_moves.first().unwrap();

                            let new_board = get_next_board(displayed_board, selected_move);

                            let new_turn = Turn {
                                previous_move: **selected_move,
                                possible_moves: compute_possible_moves(&new_board, true),
                                board: new_board
                            };
                            
                            turns.history.truncate(display_state.displayed_turn + 1);

                            turns.history.push_back(new_turn);

                            display_state.displayed_turn += 1;
                        }
                        else {
                            eprintln!("move_piece: mix of promotion and non-promotion moves");
                        }
                    }
                }
            }
        }
    }
}

// This system should be run in PostUpdate after transform propagation
fn update_transform_cache(
    mut query: Query<(&mut InverseGTransformCache, &GlobalTransform), Changed<GlobalTransform>>,
) {
    let Ok((mut cache, transform)) = query.get_single_mut() else { return };
    cache.matrix = transform.compute_matrix().inverse();
}

// Updates background / playground transforms when the window is resized or the board changes
// The playground transform is set up so that:
// - piece positions reflect their coordinates on the board
//   (with the origin set to the bottom-left square seen from the white side)
// - pieces have a sprite size of (1.,1.)
fn update_playground_transform(
    mut resize_events: EventReader<WindowResized>,
    mut set: ParamSet<(
        Query<&mut Transform, With<Background>>,
        Query<&mut Transform, With<Playground>>,
    )>,
    turns: Res<Turns>,
    display_state: Res<BoardDisplayState>,
    windows: Query<&Window>
) {
    let Some(Turn { board, .. }) = turns.history.get(display_state.displayed_turn)
    else { eprintln!("update_playground_transform: no board in history"); return };

    let (bw, bh) = board.spaces.dim();
    let (bw, bh) = (bw as f32, bh as f32);

    let mut update_transforms = |ww: f32, wh: f32| {
        // Resize the background so that it always fully covers the window
        if let Ok(mut transform) = set.p0().get_single_mut() {
            transform.scale =
                Vec2::splat(f32::max(ww / BG_TEX_SIZE.x, wh / BG_TEX_SIZE.y)).extend(1.);
        }

        // Resize the playground so that it is always fully visible
        if let Ok(mut transform) = set.p1().get_single_mut() {
            let pg_scale = f32::min(ww / (bw + 2.), wh / (bh + 2.));
            transform.scale = Vec3 {
                x: pg_scale,
                y: pg_scale,
                z: 1.,
            };

            if display_state.bottom_side == Side::White {
                transform.rotation = Quat::IDENTITY;
                transform.translation = Vec3 {
                    x: -(bw - 1.) / 2. * pg_scale,
                    y: -(bh - 1.) / 2. * pg_scale,
                    z: 0.,
                };
            }
            else {
                transform.rotation = Quat::from_rotation_z(PI);
                transform.translation = Vec3 {
                    x: (bw - 1.) / 2. * pg_scale,
                    y: (bh - 1.) / 2. * pg_scale,
                    z: 0.,
                };
            }
            
        }
    };

    for event in resize_events.read() {
        update_transforms(event.width as f32, event.height as f32);
    }

    if display_state.is_changed() {
        let Ok(window) = windows.get_single()
        else { eprintln!("update_playground_transform: could not fetch window"); return };

        update_transforms(window.width(), window.height());
    }
}