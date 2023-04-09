use bevy::prelude::*;
use ndarray::*;
use crate::sets::*;

pub struct BoardPlugin {
    pub board_string: &'static str
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board { spaces: Array2::<Space>::from_elem((0, 0), Space::Hole) })
            .insert_resource(SavedGame { board_string: self.board_string })
            .insert_resource(Side::White)
            .add_startup_system(setup_board.in_set(GameSet::BoardSetup));
    }
}

#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Coords {
    pub x: i32,
    pub y: i32
}

#[derive(Resource, Clone, Copy, PartialEq)]
pub enum Side {
    White,
    Black
}

// Marks an entity as that of a square on the board.
#[derive(Component)]
pub struct Square;

// Marks an entity as that of a piece on the board.
#[derive(Component, Clone, Copy, PartialEq, PartialOrd)]
pub enum Piece {
    WKing,
    WQueen,
    WBishop,
    WKnight,
    WRook,
    WPawn,
    BKing,
    BQueen,
    BBishop,
    BKnight,
    BRook,
    BPawn
}
use Piece::*;

impl Piece {
    pub fn side(self: &Self) -> Side {
        if *self < BKing { Side::White }
        else { Side::Black }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Space {
    Hole,
    Square,
    Piece(Piece)
}

// Lightweight board representation, updated from piece entities upon change
// Useful when computing possible moves etc.
#[derive(Resource, Clone, PartialEq)]
pub struct Board {
    pub spaces: Array2<Space>
}

unsafe impl NdIndex<Ix2> for Coords {
    #[inline]
    fn index_checked(&self, dim: &Ix2, strides: &Ix2) -> Option<isize> {
        if self.x < 0 || self.y < 0 { None }
        else { (self.x as usize, self.y as usize).index_checked(dim, strides) }
    }
    #[inline]
    fn index_unchecked(&self, strides: &Ix2) -> isize {
        (self.x as usize, self.y as usize).index_unchecked(strides)
    }
}

#[derive(Resource)]
struct SavedGame {
    pub board_string: &'static str
}

fn setup_board(
    mut commands: Commands,
    saved_game: Res<SavedGame>,
    mut board: ResMut<Board>,
    side: Res<Side>
) {
    let board_str = saved_game.board_string;
    assert!(board_str.is_ascii(), "Board string contains invalid characters");

    let rows: Vec<&str> = board_str
        .lines()
        .map(|row| row.trim())
        .filter(|row| !row.is_empty())
        .collect();

    assert!(rows.len() != 0, "No board string given");
    assert!(rows.iter().all(|row| row.len() == rows[0].len()),
            "Inconsistent board row length in board string");

    let bw = rows[0].len();
    let bh = rows.len();
    board.spaces = Array::from_elem((bw, bh), Space::Hole);

    for (coords, &byte) in rows.iter()
        .enumerate()
        .map(|(row_n, row)| {
            let y = rows.len() - row_n - 1;
            row.as_bytes()
                .iter()
                .enumerate()
                .map(move |(col_n, byte)| (
                    Coords { x: col_n as i32, y: y as i32},
                    byte
                ))
        })
        .flatten()
    {
        if byte != b'X' {
            commands.spawn((
                Square,
                coords
            ));

            board.spaces[coords] = Space::Square;
        }

        if let Some(piece) = match byte {
            b'K' => Some(WKing),
            b'Q' => Some(WQueen),
            b'R' => Some(WRook),
            b'B' => Some(WBishop),
            b'N' => Some(WKnight),
            b'P' => Some(WPawn),
            b'k' => Some(BKing),
            b'q' => Some(BQueen),
            b'r' => Some(BRook),
            b'b' => Some(BBishop),
            b'n' => Some(BKnight),
            b'p' => Some(BPawn),
            _ => None
        } {
            commands.spawn((
                piece,
                coords
            ));

            board.spaces[coords] = Space::Piece(piece);
        }
    }
}