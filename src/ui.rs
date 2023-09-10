use crate::board::PieceModel::*;
use crate::board::*;
use crate::logic::*;
use crate::sets::*;
use crate::turns::*;
use bevy::{prelude::*, transform::TransformSystem, window::WindowResized};
use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    fn greet();
}

const BG_TEX_SIZE: Vec2 = Vec2::new(2560., 1587.);
const PIECE_TEX_SIZE: f32 = 256.;

pub struct UIPlugin;

impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SelectedPiece::default())
            .insert_resource(PromotionSelection::default())
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
                handle_window_resize
            ));
    }
}

#[derive(Component)]
struct Background;

#[derive(Component)]
struct PlayArea;

#[derive(Resource, Deref, DerefMut)]
struct PieceAtlas(Handle<TextureAtlas>);

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

#[derive(Component)]
struct Square;

#[derive(Resource, Default, Deref, DerefMut)]
struct SelectedPiece(Option<Entity>);

#[derive(Component)]
struct Marker;

#[derive(Resource, Deref, DerefMut)]
struct MarkerTexture(Handle<Image>);

#[derive(Resource, Default)]
struct PromotionSelection {
    // There is only Some(Move) while a promotion is being actively chosen
    pub move_: Option<Move>
}

#[derive(Component)]
struct PromotionPopup;

#[derive(Resource, Deref, DerefMut)]
struct PromotionPopupTexture(Handle<Image>);

#[derive(Component, Deref, DerefMut)]
struct PromotionChoice(PieceModel);

fn update_board_display(
    mut commands: Commands,
    mut set: ParamSet<(
        Query<Entity, With<PlayArea>>,
        Query<Entity, Or<(With<Piece>, With<Square>)>>
    )>,
    history: Res<TurnHistory>,
    displayed_turn_idx: Res<DisplayedTurn>,
    piece_atlas: Res<PieceAtlas>
) {
    if !displayed_turn_idx.is_changed() { return; }

    let Some(Turn { board, .. }) = history.get(**displayed_turn_idx)
    else { eprintln!("update_board_display: can't find board to display"); return };

    let Ok(pa_entity) = set.p0().get_single()
    else { eprintln!("update_board_display: no spawned play area"); return };

    for old_entity in set.p1().iter_mut() {
        if let Some(mut ec) = commands.get_entity(old_entity) {
            ec.despawn()
        }
    }

    commands.entity(pa_entity).with_children(|parent| {
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
                        texture_atlas: piece_atlas.clone(),
                        sprite: TextureAtlasSprite {
                            index: piece.texture_index(),
                            custom_size: Some(Vec2::ONE),
                            ..default()
                        },
                        transform: Transform::from_translation(Vec3::new(
                            x as f32, y as f32, 2.,
                        )),
                        ..default()
                    },
                ));
            }
        }
    });
}

fn init_ui(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut atlases: ResMut<Assets<TextureAtlas>>,
) {
    commands.spawn(Camera2dBundle::default());

    commands.insert_resource(PieceAtlas(atlases.add(TextureAtlas::from_grid(
        asset_server.load("pieces.png"),
        Vec2::splat(PIECE_TEX_SIZE),
        6,
        2,
        None,
        None,
    ))));

    commands.insert_resource(MarkerTexture(asset_server.load("marker.png")));

    commands.insert_resource(PromotionPopupTexture(asset_server.load("promotion_popup.png")));

    commands.spawn((
        Background,
        SpriteBundle {
            texture: asset_server.load("bg.png"),
            ..default()
        },
    ));

    commands.spawn((
        PlayArea,
        TransformBundle::default(),
        InverseGTransformCache::default(),
        VisibilityBundle::default(),
    ));
}

fn move_piece(
    mut commands: Commands,
    windows: Query<&Window>,
    buttons: Res<Input<MouseButton>>,
    play_area: Query<(Entity, &InverseGTransformCache), With<PlayArea>>,
    mut displayed_pieces: Query<(Entity, &Piece, &mut Transform, &Coords), Without<PromotionChoice>>,
    mut selected: ResMut<SelectedPiece>,
    markers: Query<Entity, With<Marker>>,
    marker_texture: Res<MarkerTexture>,
    mut history: ResMut<TurnHistory>,
    mut displayed_turn_idx: ResMut<DisplayedTurn>,
    mut promotion_selection: ResMut<PromotionSelection>,
    promotion_popup_texture: Res<PromotionPopupTexture>,
    piece_atlas: Res<PieceAtlas>,
    promotion_choices: Query<(&PromotionChoice, &Transform)>,
    promotion_graphics: Query<Entity, Or<(With<PromotionPopup>, With<PromotionChoice>)>>
) {
    let Some(displayed_turn @ Turn { board: displayed_board, .. }) = history.get(**displayed_turn_idx)
    else { eprintln!("select_piece: could not find current turn"); return };

    let Ok((pa_entity, InverseGTransformCache { matrix: pa_inv_matrix })) = play_area.get_single() else { return };
    
    let mouse_pos = {
        let Ok(window) = windows.get_single() else { eprintln!("select_piece: Could not fetch window"); return };
        let Some(mut pos) = window.cursor_position() else { return };
        pos.x -= window.width() / 2.;
        pos.y = window.height() / 2. - pos.y;

        pa_inv_matrix.transform_point3(pos.extend(0.))
    };

    let mouse_coords = Coords {
        x: mouse_pos.x.round() as isize,
        y: mouse_pos.y.round() as isize,
    };


    if let Some(ref mut prom_move) = promotion_selection.move_ {
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
                    
                    history.truncate(**displayed_turn_idx + 1);

                    history.push_back(new_turn);

                    **displayed_turn_idx += 1;
                    
                    for entity in promotion_graphics.iter() {
                        commands.entity(entity).despawn();
                    }

                    promotion_selection.move_ = None;
                    
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
            **selected = Some(piece_entity);

            // Snap the piece to mouse position
            piece_transform.translation = mouse_pos.truncate().extend(3.);
            piece_transform.scale = Vec2::splat(1.2).extend(1.);

            // Display possible move markers
            if let Some(moves) = displayed_turn.possible_moves.get(piece_coords) {
                commands.entity(pa_entity).with_children(|parent| {
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
                                texture: marker_texture.clone(),
                                ..default()
                            },
                        ));
                    }
                });
            }

            // Display promotion squares markers
            if let Piece { model: Pawn { .. }, side } = piece {
                commands.entity(pa_entity).with_children(|parent| {
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
                                        texture: marker_texture.clone(),
                                        ..default()
                                    },
                                ));
                            }
                        }
                    }
                });
            }
        }
    } else if let Some(piece_entity) = **selected {
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
            **selected = None;

            // Stop displaying move markers
            markers.for_each(|marker_entity| {
                if let Some(mut ec) = commands.get_entity(marker_entity) {
                    ec.despawn()
                }
            });

            if let Ok((_, piece, _, piece_coords)) = displayed_pieces.get(piece_entity) {
                if let Some(piece_moves) = displayed_turn.possible_moves.get(piece_coords) {
                    let selected_moves: Vec<_> = piece_moves
                        .iter()
                        .filter(|move_| {
                            move_.target == mouse_coords
                        })
                        .collect();
                    
                    if !selected_moves.is_empty() {
                        // Put up a popup for promotions
                        if selected_moves.iter().all(|move_| move_.promotion.is_some()) {
                            promotion_selection.move_ = Some(Move {
                                promotion: None,
                                ..**selected_moves.first().unwrap()
                            });

                            commands.entity(piece_entity).despawn();
                            
                            commands.entity(pa_entity).with_children(|parent| {
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
                                        texture: promotion_popup_texture.clone(),
                                        ..default()
                                    },
                                ));
                                
                                let mut choice_x = target.x as f32 - (selected_moves.len() - 1) as f32 / 2.;

                                for move_ in selected_moves {
                                    let model = move_.promotion.unwrap();

                                    parent.spawn((
                                        PromotionChoice(model),
                                        SpriteSheetBundle {
                                            texture_atlas: piece_atlas.clone(),
                                            sprite: TextureAtlasSprite {
                                                index: Piece {
                                                    model,
                                                    side: piece.side
                                                }.texture_index(),
                                                custom_size: Some(Vec2::ONE),
                                                ..default()
                                            },
                                            transform: Transform::from_translation(Vec3::new(
                                                choice_x, target.y as f32, 4.,
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
                            
                            history.truncate(**displayed_turn_idx + 1);

                            history.push_back(new_turn);

                            **displayed_turn_idx += 1;

                            greet();
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

fn handle_window_resize(
    mut events: EventReader<WindowResized>,
    mut set: ParamSet<(
        Query<&mut Transform, With<Background>>,
        Query<&mut Transform, With<PlayArea>>,
    )>,
    history: Res<TurnHistory>,
) {
    for event in events.iter() {
        let Some(Turn { board, .. }) = history.back() else { eprintln!("handle_window_resize: no board in history"); return };
        let (ww, wh) = (event.width as f32, event.height as f32);
        let (bw, bh) = board.spaces.dim();
        let (bw, bh) = (bw as f32, bh as f32);

        // Resize the background so that it always fully covers the window
        if let Ok(mut transform) = set.p0().get_single_mut() {
            transform.scale =
                Vec2::splat(f32::max(ww / BG_TEX_SIZE.x, wh / BG_TEX_SIZE.y)).extend(1.);
        }

        // Resize the play area so that it is always fully visible
        if let Ok(mut transform) = set.p1().get_single_mut() {
            let pa_scale = f32::min(ww / (bw + 2.), wh / (bh + 2.));
            transform.scale = Vec3 {
                x: pa_scale,
                y: pa_scale,
                z: 1.,
            };
            transform.translation = Vec3 {
                x: -(bw - 1.) / 2. * pa_scale,
                y: -(bh - 1.) / 2. * pa_scale,
                z: 0.,
            };
        }
    }
}
