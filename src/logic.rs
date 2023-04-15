use bevy::prelude::*;
use core::panic;
use std::collections::HashMap;
use std::iter::from_fn;
use std::ops::Not;

use crate::board::*;
use crate::board::Side::*;
use crate::board::Space::*;
use crate::board::PieceModel::*;

const CAN_CASTLE_WITH_PROMOTED_ROOK: bool = true;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PossibleMoves{0: default()})
            .add_system(update_possible_moves);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum MoveKind {
    Standard,
    Dash,
    Capture,
    Castle { rook_coords: Coords },
    EnPassant,
    Skip
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Move {
    pub source: Coords,
    pub target: Coords,
    pub kind: MoveKind,
    pub promotion: Option<PieceModel>
}

impl Move {
    pub const fn skip() -> Self {
        Move {
            source: Coords { x: 0, y: 0 },
            target: Coords { x: 0, y: 0 },
            kind: MoveKind::Skip,
            promotion: None
        }
    }
}

// Returns a new board with a move applied
// Panics on impossible moves or out of bounds coords
pub fn get_next_board(board: &Board, move_: &Move) -> Board {
    let mut next_board = board.clone();

    next_board.turn = board.turn.other();

    if move_.kind == MoveKind::Skip {
        return next_board;
    }

    let Square { slot: ref mut source_slot, .. } = next_board.spaces[move_.source] else {
        panic!("Invalid move, no source square");
    };
    let Some(mut source_piece) = *source_slot else {
        panic!("Invalid move, no source piece");
    };

    *source_slot = None;

    if let Piece { model: Pawn { ref mut can_dash, ref mut just_dashed }, .. } = source_piece {
        *can_dash = false;
        *just_dashed = false;
    }

    match move_.kind {
        MoveKind::Capture => {
            let Square { slot: Some(captured_piece), .. } = next_board.spaces[move_.target] else {
                panic!("Invalid capture, no target piece");
            };

            next_board.captured.push(captured_piece);
        },
        MoveKind::Dash => {
            let Piece { model: Pawn { ref mut just_dashed, .. }, .. } = source_piece else {
                panic!("Invalid dash, no source pawn");
            };

            *just_dashed = true;
        }
        MoveKind::EnPassant => {
            let captured_coords = Coords {
                x: move_.target.x,
                y: move_.source.y
            };

            let Square { slot: ref mut captured_slot, .. } = next_board.spaces[captured_coords] else {
                panic!("Invalid en passant, no captured square - google it!");
            };
            let Some(captured_piece) = *captured_slot else {
                panic!("Invalid en passant, no captured piece");
            };

            next_board.captured.push(captured_piece);
            *captured_slot = None;
        },
        MoveKind::Castle { rook_coords } => {
            let Square { slot: ref mut rook_source_slot, .. } = next_board.spaces[rook_coords] else {
                panic!("Invalid castle, no rook source square");
            };
            let Some(rook) = *rook_source_slot else {
                panic!("Invalid castle, no rook");
            };
            
            *rook_source_slot = None;

            let Square { slot: ref mut rook_target_slot, .. } = next_board.spaces[Coords {
                x: move_.target.x,
                y: move_.target.y,
            }] else {
                panic!("Invalid castle, no rook target square");
            };

            *rook_target_slot = Some(Piece {
                model: Rook { can_castle: false },
                ..rook
            });

            let Piece { model: King { ref mut can_castle }, .. } = source_piece else {
                panic!("Invalid castle, no source king");
            };

            *can_castle = false;
        },
        _ => ()
    }

    let Square { slot: ref mut target_slot, .. } = next_board.spaces[move_.target] else {
        panic!("Invalid move, no target square");
    };

    if let Some(model) = move_.promotion {
        *target_slot = Some(Piece {
            model,
            ..source_piece
        });
    }
    else {
        *target_slot = Some(source_piece);
    }

    next_board
}

pub fn is_in_check_after_move(board: &Board, move_: &Move) -> bool {
    let next_board = get_next_board(board, move_);

    let king_coords: Vec<_> = next_board.spaces.indexed_iter()
        .filter_map(|((x, y), space)| {
            if let Space::Square { slot: Some(Piece { side: piece_side, model: King { .. } }), .. } = space {
                if *piece_side == board.turn {
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
        .not()
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
            if let Square { slot: Some(piece), .. } = space {
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
                    Some(Square { slot: None, .. }) => Some(Move {
                        source: coords,
                        target,
                        kind: MoveKind::Standard,
                        promotion: None
                    }),
                    Some(Square { slot: Some(target_piece), .. }) => {
                        if target_piece.side != piece.side {
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
                        Some(Square { slot: None, .. }) => Some(Move {
                            source: coords,
                            target: tc,
                            kind: MoveKind::Standard,
                            promotion: None
                        }),
                        Some(Square { slot: Some(target_piece), .. }) => {
                            if target_piece.side != piece.side {
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

            let filter_checks_fn = |move_: &Move| {
                return !filter_checks || !is_in_check_after_move(board, move_);
            };

            (
                coords,

                // Pattern matching to build a vec of valid moves depending on source piece type
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
                                if !can_castle || !filter_checks_fn(&Move::skip()) { return None; }
                                let mut rook_offset = 1;
                                loop {
                                    let rook_coords = Coords{ x: x + x_dir * rook_offset, y };
                                    match board.spaces.get(rook_coords) {
                                        Some(Square { slot: None, .. }) => {
                                            if filter_checks_fn(&Move {
                                                source: coords,
                                                target: rook_coords,
                                                kind: MoveKind::Standard,
                                                promotion: None
                                            }) {
                                                rook_offset += 1;
                                            }
                                            else { return None; }
                                        },
                                        Some(Square { slot: Some(Piece { side: rook_side, model: Rook { can_castle: true } }), .. }) => {
                                            if rook_offset > 2 && board.turn == *rook_side {
                                                return Some(Move {
                                                    source: coords,
                                                    target: Coords { x: x + x_dir * 2, y },
                                                    kind: MoveKind::Castle { rook_coords },
                                                    promotion: None
                                                });
                                            }
                                            else { return None; }
                                        },
                                        _ => { return None; }
                                    }
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
                                        [Queen, Bishop, Knight, Rook { can_castle: CAN_CASTLE_WITH_PROMOTED_ROOK }].into_iter()
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

                        if matches!(board.spaces.get(forward), Some(Square { slot: None, .. })) {
                            push_with_promotions(Move {
                                source: coords,
                                target: forward,
                                kind: MoveKind::Standard,
                                promotion: None
                            });
                        
                            let dash = Coords{ x, y: y + 2 * y_dir };
                            if can_dash && matches!(board.spaces.get(dash), Some(Square { slot: None, .. })) {
                                push_with_promotions(Move {
                                    source: coords,
                                    target: dash,
                                    kind: MoveKind::Dash,
                                    promotion: None
                                });
                            }

                            for ep_target in [Coords{ x: x-1, y }, Coords{ x: x+1, y }] {
                                if let Some(Square {
                                    slot: Some(Piece { side: target_side, model: Pawn { just_dashed: true, .. } }),
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
                            if let Some(Square { slot: Some(Piece { side: target_side, .. }), .. }) = board.spaces.get(target) {
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