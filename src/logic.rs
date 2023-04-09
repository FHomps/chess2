use std::collections::HashMap;

use crate::board::*;
use crate::board::Piece::*;

fn compute_possible_moves(
    board: &Board,
    to_play: Side
) -> HashMap<Coords, Vec<Coords>> {
    board.spaces
        .indexed_iter()
        // Filter on pieces of the right color
        .filter_map(|((x, y), space)| 
        {
            if let Space::Piece(piece) = space {
                if piece.side() == to_play {
                    Some((
                        Coords { x: x as i32, y: y as i32 },
                        *piece
                    ))
                }
                else { None }
            }
            else { None }
        })
        // Build piece moves
        .map(|(Coords { x, y }, piece)|
        {(
            Coords { x, y },
            match piece {
WKing => {
    (x-1..x+1)
        .flat_map(|dx| {
            (y-1..y+1).map(move |dy| Coords { x: dx, y: dy })  
        })
        .filter(|_| {
            true
        })
        .collect()
},
_ => vec!()    
            }
        )})
        .collect()
}