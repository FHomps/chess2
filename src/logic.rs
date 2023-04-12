use bevy::prelude::*;
use std::collections::HashMap;
use std::iter::{once, successors, from_fn};

use crate::board::*;
use crate::board::Piece::*;

pub struct LogicPlugin;

impl Plugin for LogicPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PossibleMoves{0: default()})
            .add_system(update_possible_moves);
    }
}

#[derive(Resource)]
pub struct PossibleMoves(pub HashMap<Coords, Vec<Coords>>);

fn compute_possible_moves(
    board: &Board,
    side: Side
) -> HashMap<Coords, Vec<Coords>> {
    board.spaces
        .indexed_iter()
        // Filter to keep only source pieces of the right color
        .filter_map(|((x, y), space)| 
        {
            if let Space::Piece(piece) = space {
                if piece.side() == side {
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
            (
                coords,

                // Pattern matching to build a vec of valid destination coords depending on source piece type
                match piece {
                    WKing { can_castle } | BKing { can_castle } => {
                        [
                            Coords{x: x-1, y: y+1}, Coords{x, y: y+1}, Coords{x: x+1, y: y+1},
                            Coords{x: x-1, y     },                    Coords{x: x+1, y     },
                            Coords{x: x-1, y: y-1}, Coords{x, y: y-1}, Coords{x: x+1, y: y-1}
                        ].into_iter()
                        // Keep only valid targets
                        .filter(|target| {
                            match board.spaces.get(*target) {
                                Some(Space::Square) => true,
                                Some(Space::Hole) => false,
                                Some(Space::Piece(piece)) => piece.side() != side,
                                _ => false
                            }
                        })
                        // Add castling moves
                        .chain(
                            ([-1, 1isize]).into_iter()
                            .filter_map(|dir| {
                                let mut rook_offset = 1;
                                loop {
                                    match board.spaces.get(Coords{x: x + dir * rook_offset, y}) {
                                        Some(Space::Square) => rook_offset += 1,
                                        Some(Space::Piece(Piece::WRook { can_castle: true })) => {
                                            if rook_offset > 2 && side == Side::White {
                                                return Some(Coords {x: x + dir * 2, y});
                                            }
                                            else { return None; }
                                        },
                                        Some(Space::Piece(Piece::BRook { can_castle: true })) => {
                                            if rook_offset > 2 && side == Side::Black {
                                                return Some(Coords {x: x + dir * 2, y});
                                            }
                                            else { return None; }
                                        },
                                        _ => return None
                                    };
                                }
                            })
                        )
                        .collect()
                    },
                    WQueen | BQueen => {
                        // Movement directions
                        [
                            [-1,  1], [0,  1], [1,  1],
                            [-1,  0],          [1,  0],
                            [-1, -1], [0, -1], [1, -1isize],
                        ].into_iter()
                        // Expand directions until pieces are encountered
                        .map(|[x_step, y_step]| {
                            let (mut tx, mut ty) = (x, y);
                            let mut end = false;
                            from_fn(move || {
                                if end { return None; }

                                tx += x_step;
                                ty += y_step;
                                let tc = Coords { x: tx, y: ty };
                                match board.spaces.get(tc) {
                                    Some(Space::Square) => Some(tc),
                                    Some(Space::Hole) => None,
                                    Some(Space::Piece(piece)) => {
                                        if piece.side() != side {
                                            end = true;
                                            Some(tc)
                                        }
                                        else { None }
                                    },
                                    _ => None
                                }
                            })
                        })
                        .flatten()
                        .collect()
                    }
                    _ => vec!()
                }
            )
        })
        .collect()
}

pub fn update_possible_moves(
    mut possible_moves: ResMut<PossibleMoves>,
    board: Res<Board>,
    side: Res<Side>
) {
    if !board.is_changed() { return; }
    possible_moves.0 = compute_possible_moves(&*board, *side);
}