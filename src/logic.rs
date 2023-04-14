use bevy::prelude::*;
use std::collections::HashMap;
use std::iter::from_fn;

use crate::board::*;
use crate::board::Side::*;
use crate::board::Space::*;
use crate::board::PieceModel::*;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PossibleMoves{0: default()})
            .add_system(update_possible_moves);
    }
}

#[derive(Resource)]
pub struct PossibleMoves(pub HashMap<Coords, Vec<Move>>);

fn compute_possible_moves(
    board: &Board,
    filter_checks: bool
) -> HashMap<Coords, Vec<Move>> {
    board.spaces
        .indexed_iter()
        // Filter to keep only source pieces of the right color
        .filter_map(|((x, y), space)| 
        {
            if let Square { piece: Some(piece), .. } = space {
                if piece.side == board.turn { 
                    Some((
                        Coords { x: x as isize, y: y as isize },
                        *piece
                    ))
                }
                else { None }
            }
            else { None }
        })
        // Build piece moves
        .map(|(coords, piece)|
        {
            let Coords { x, y } = coords;

            let move_from_target_fn = |target: Coords| {
                match board.spaces.get(target) {
                    Some(Square { piece: None, .. }) => Some(Move {
                        source: coords,
                        target,
                        kind: MoveKind::Standard,
                        promotion: None
                    }),
                    Some(Square { piece: Some(piece), .. }) => {
                        if piece.side != board.turn {
                            Some(Move {
                                source: coords,
                                target,
                                kind: MoveKind::Capture,
                                promotion: None
                            })
                        }
                        else { None }
                    },
                    Some(Hole) => None,
                    _ => None
                }
            };

            let moves_from_direction_fn = |direction: [isize; 2]| {
                let (mut tx, mut ty) = (x, y);
                let mut stop = false;
                from_fn(move || {
                    if stop { return None; }

                    tx += direction[0];
                    ty += direction[1];
                    let tc = Coords { x: tx, y: ty };
                    match board.spaces.get(tc) {
                        Some(Square { piece: None, .. }) => Some(Move {
                            source: coords,
                            target: tc,
                            kind: MoveKind::Standard,
                            promotion: None
                        }),
                        Some(Square { piece: Some(target_piece), .. }) => {
                            if target_piece.side != board.turn {
                                stop = true;
                                Some(Move {
                                    source: coords,
                                    target: tc,
                                    kind: MoveKind::Capture,
                                    promotion: None
                                })
                            }
                            else { None }
                        },
                        Some(Hole) => None,
                        _ => None
                    }
                })
            };

            let filter_checks_fn = |_move: &Move| {
                if !filter_checks { return true; }
                
                let next_board = board.next(_move);
                let king_coords: Vec<_> = next_board.spaces.indexed_iter()
                    .filter_map(|((x, y), space)| {
                        if let Space::Square { piece: Some(Piece { side, model: King { .. } }), .. } = space {
                            if *side == board.turn {
                                Some(Coords { x: x as isize, y: y as isize })
                            }
                            else { None }
                        }
                        else { None }
                    })
                    .collect();

                compute_possible_moves(&next_board, false).iter()
                    .all(|(_, enemy_moves)| {
                        enemy_moves.iter()
                        .all(|enemy_move| {
                            if let Move { kind: MoveKind::Capture | MoveKind::EnPassant, target, .. } = enemy_move {
                                king_coords.iter().all(|coords| *target != *coords )
                            }
                            else { true }
                        })
                    })
            };

            (
                coords,

                // Pattern matching to build a vec of valid destination coords depending on source piece type
                match piece.model {
                    King { can_castle } => {
                        [
                            Coords{x: x-1, y: y+1}, Coords{x, y: y+1}, Coords{x: x+1, y: y+1},
                            Coords{x: x-1, y     },                    Coords{x: x+1, y     },
                            Coords{x: x-1, y: y-1}, Coords{x, y: y-1}, Coords{x: x+1, y: y-1}
                        ].into_iter()
                        // Keep only valid targets
                        .filter_map(move_from_target_fn)
                        // Add castling moves
                        .chain(
                            [-1, 1isize].into_iter()
                            .filter_map(|x_dir| {
                                if !can_castle { return None; }
                                let mut rook_offset = 1;
                                loop {
                                    let rook_coords = Coords{ x: x + x_dir * rook_offset, y };
                                    match board.spaces.get(rook_coords) {
                                        Some(Square { piece: None, .. }) => rook_offset += 1,
                                        Some(Square { piece: Some(Piece { side: rook_side, model: Rook { can_castle: true } }), .. }) => {
                                            if rook_offset > 2 && board.turn == *rook_side {
                                                return Some(Move {
                                                    source: coords,
                                                    target: Coords { x: x + x_dir * 2, y },
                                                    kind: MoveKind::Castle { rook_coords },
                                                    promotion: None
                                                })
                                            }
                                            else { return None }
                                        },
                                        _ => return None
                                    };
                                }
                            })
                        )
                        .filter(filter_checks_fn)
                        .collect()
                    },
                    Queen => {
                        // Movement directions
                        [
                            [-1,  1], [0,  1], [1,  1],
                            [-1,  0],          [1,  0],
                            [-1, -1], [0, -1], [1, -1isize]
                        ].into_iter()
                        // Expand directions until pieces are encountered
                        .map(moves_from_direction_fn)
                        .flatten()
                        .filter(filter_checks_fn)
                        .collect()
                    },
                    Bishop => {
                        [
                            [-1,  1], [1,  1], [-1, -1], [1, -1isize]
                        ].into_iter()
                        .map(moves_from_direction_fn)
                        .flatten()
                        .filter(filter_checks_fn)
                        .collect()
                    }
                    Rook {..} => {
                        [
                            [-1,  0], [0,  1], [0, -1], [1, 0isize]
                        ].into_iter()
                        .map(moves_from_direction_fn)
                        .flatten()
                        .filter(filter_checks_fn)
                        .collect()
                    },
                    Knight => {
                        [
                            Coords{x: x-1, y: y+2}, Coords{x: x+1, y: y+2},
                            Coords{x: x+2, y: y-1}, Coords{x: x+2, y: y+1},
                            Coords{x: x-1, y: y-2}, Coords{x: x+1, y: y-2},
                            Coords{x: x-2, y: y-1}, Coords{x: x-2, y: y+1},
                        ].into_iter()
                        .filter_map(move_from_target_fn)
                        .filter(filter_checks_fn)
                        .collect()
                    },
                    Pawn { can_dash, .. } => {
                        let mut moves = vec!();
                        let y_dir = if board.turn == White { 1isize } else { -1isize };
                        let forward = Coords{ x, y: y + y_dir };

                        let mut push_with_promotions = |base_move: Move| {
                            if let Some(Square { promotes, .. }) = board.spaces.get(base_move.target) {
                                if promotes[piece.side as usize] {
                                    moves.extend(
                                        [Queen, Bishop, Knight, Rook { can_castle: false }].into_iter()
                                        .map(|model| Move {
                                            promotion: Some(model),
                                            ..base_move
                                        })
                                    );
                                }
                                else { moves.push(base_move); }
                            }
                            else { moves.push(base_move); }
                        };

                        if matches!(board.spaces.get(forward), Some(Square { piece: None, .. })) {
                            push_with_promotions(Move {
                                source: coords,
                                target: forward,
                                kind: MoveKind::Standard,
                                promotion: None
                            });
                        
                            let dash = Coords{ x, y: y + 2 * y_dir };
                            if can_dash && matches!(board.spaces.get(dash), Some(Square { piece: None, .. })) {
                                push_with_promotions(Move {
                                    source: coords,
                                    target: dash,
                                    kind: MoveKind::Dash,
                                    promotion: None
                                });
                            }

                            for ep_target in [Coords{ x: x-1, y }, Coords{ x: x+1, y }] {
                                if let Some(Square {
                                    piece: Some(Piece { side: target_side, model: Pawn { just_dashed: true, .. } }),
                                    .. 
                                }) = board.spaces.get(ep_target) {
                                    if board.turn != *target_side {
                                        push_with_promotions(Move {
                                            source: coords,
                                            target: Coords { x: ep_target.x, y: y + y_dir },
                                            kind: MoveKind::EnPassant,
                                            promotion: None
                                        });
                                    }
                                }
                            }
                        }

                        for target in [Coords{ x: x-1, y: y + y_dir }, Coords{ x: x+1, y: y + y_dir }] {
                            if let Some(Square { piece: Some(Piece { side: target_side, .. }), .. }) = board.spaces.get(target) {
                                if board.turn != *target_side {
                                    push_with_promotions(Move {
                                        source: coords,
                                        target,
                                        kind: MoveKind::Capture,
                                        promotion: None
                                    });
                                }
                            }
                        }

                        moves.into_iter()
                            .filter(filter_checks_fn)
                            .collect()
                    }
                }
            )
        })
        .collect()
}

pub fn update_possible_moves(
    mut possible_moves: ResMut<PossibleMoves>,
    board: Res<Board>,
) {
    if !board.is_changed() { return; }
    possible_moves.0 = compute_possible_moves(&*board, true);
}