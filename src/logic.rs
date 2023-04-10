use std::collections::HashMap;
use std::iter::once;

use crate::board::*;
use crate::board::Piece::*;

fn filter_captures(target: &Coords, board: &Board, side: Side) -> bool {
    // get() will return None if target space is out of bounds
    let Some(space) = board.spaces.get(*target) else { return false; };
    match space {
        Space::Square => true,
        Space::Hole => false,
        Space::Piece(piece) => piece.side() != side
    }
}

pub fn compute_possible_moves(
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
                            Coords{x: x-1, y: y+1}, Coords{x,      y: y+1}, Coords{x: x+1, y: y+1},
                            Coords{x: x-1, y     },                         Coords{x: x+1, y     },
                            Coords{x: x-1, y: y-1}, Coords{x,      y: y-1}, Coords{x: x+1, y: y-1}
                        ].into_iter()
                        .filter(|target| filter_captures(target, board, side))
                        // Add castling moves
                        .chain(
                            ([-1 as isize, 1]).into_iter()
                            .filter_map(|dir| {
                                let mut rook_offset = 1;
                                loop {
                                    match board.spaces.get(Coords{x: x + dir * rook_offset, y}) {
                                        Some(Space::Square) => rook_offset += 1,
                                        Some(Space::Piece(Piece::WRook { can_castle: true })) =>
                                            if rook_offset > 2 && side == Side::White { return Some(Coords {x: x + dir * 2, y}); },
                                        Some(Space::Piece(Piece::BRook { can_castle: true })) =>
                                            if rook_offset > 2 && side == Side::Black { return Some(Coords {x: x + dir * 2, y}); },
                                        _ => return None
                                    }
                                }
                            })
                        )
                        .collect()
                    },
                    _ => vec!()
                }
            )
        })
        .collect()
}