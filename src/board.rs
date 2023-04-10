use bevy::prelude::*;
use ndarray::*;
use crate::sets::*;

pub struct BoardPlugin {
    pub board_string: &'static str
}

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Board::default())
            .insert_resource(SavedGame { board_string: self.board_string })
            .insert_resource(Side::White)
            .add_startup_system(setup_board.in_set(GameSet::BoardSetup));
    }
}

// Signed coordinates, useful for computations before filtering out of bounds squares
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Coords {
    pub x: isize,
    pub y: isize
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
#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Piece {
    WKing {
        can_castle: bool
    },
    WQueen,
    WBishop,
    WKnight,
    WRook {
        can_castle: bool
    },
    WPawn {
        can_dash: bool,
        just_dashed: bool
    },
    BKing {
        can_castle: bool
    },
    BQueen,
    BBishop,
    BKnight,
    BRook {
        can_castle: bool
    },
    BPawn {
        can_dash: bool,
        just_dashed: bool
    }
}
use Piece::*;

impl Piece {
    pub fn side(self: &Self) -> Side {
        match self {
            WKing{..} | WQueen | WBishop | WKnight | WRook{..} | WPawn{..} => Side::White,
            BKing{..} | BQueen | BBishop | BKnight | BRook{..} | BPawn{..} => Side::Black
        }
    }

    pub fn texture_index(self: &Self) -> usize {
        match self {
            WKing{..} => 0,
            WQueen    => 1,
            WBishop   => 2,
            WKnight   => 3,
            WRook{..} => 4,
            WPawn{..} => 5,
            BKing{..} => 6,
            BQueen    => 7,
            BBishop   => 8,
            BKnight   => 9,
            BRook{..} => 10,
            BPawn{..} => 11
        }
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Space {
    Hole,
    Square,
    Piece(Piece)
}

#[derive(Resource, Clone, PartialEq)]
pub struct Board {
    pub spaces: Array2<Space>
}

impl Default for Board {
    fn default() -> Self {
        Board { spaces: Array2::<Space>::from_elem((0, 0), Space::Hole) }
    }
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
    board.spaces = Array2::from_elem((bw, bh), Space::Hole);

    for (coords, &byte) in rows.iter()
        .enumerate()
        .map(|(row_n, row)| {
            let y = rows.len() - row_n - 1;
            row.as_bytes()
                .iter()
                .enumerate()
                .map(move |(col_n, byte)| (
                    Coords { x: col_n as isize, y: y as isize},
                    byte
                ))
        })
        .flatten()
    {
        if byte != b'X' {
            board.spaces[coords] = Space::Square;
        }

        if let Some(piece) = match byte {
            b'K' => Some(WKing { can_castle: true }),
            b'Q' => Some(WQueen),
            b'R' => Some(WRook { can_castle: true }),
            b'B' => Some(WBishop),
            b'N' => Some(WKnight),
            b'P' => Some(WPawn { can_dash: true, just_dashed: false }),
            b'k' => Some(BKing { can_castle: true }),
            b'q' => Some(BQueen),
            b'r' => Some(BRook { can_castle: true }),
            b'b' => Some(BBishop),
            b'n' => Some(BKnight),
            b'p' => Some(BPawn { can_dash: true, just_dashed: false }),
            _ => None
        } {
            board.spaces[coords] = Space::Piece(piece);
        }
    }
}