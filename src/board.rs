use bevy::prelude::*;
use ndarray::*;

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, _app: &mut App) {}
}

// Signed coordinates, useful for computations before filtering out of bounds squares
#[derive(Component, Clone, Copy, PartialEq, Eq, Hash, Default, Debug)]
pub struct Coords {
    pub x: isize,
    pub y: isize,
}

unsafe impl NdIndex<Ix2> for Coords {
    #[inline]
    fn index_checked(&self, dim: &Ix2, strides: &Ix2) -> Option<isize> {
        if self.x < 0 || self.y < 0 {
            None
        } else {
            (self.x as usize, self.y as usize).index_checked(dim, strides)
        }
    }
    #[inline]
    fn index_unchecked(&self, strides: &Ix2) -> isize {
        (self.x as usize, self.y as usize).index_unchecked(strides)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Side {
    #[default]
    White,
    Black,
}
use Side::*;

impl Side {
    pub fn other(self: &Self) -> Self {
        match self {
            White => Black,
            Black => White,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum PieceModel {
    King { can_castle: bool },
    Queen,
    Bishop,
    Knight,
    Rook { can_castle: bool },
    Pawn { can_dash: bool, just_dashed: bool },
}
use PieceModel::*;

// Marks an entity as that of a piece on the board.
#[derive(Component, Clone, Copy, PartialEq, Eq, Debug)]
pub struct Piece {
    pub side: Side,
    pub model: PieceModel,
}

impl Piece {
    pub fn texture_index(self: &Self) -> usize {
        (match self.model {
            King { .. } => 0,
            Queen => 1,
            Bishop => 2,
            Knight => 3,
            Rook { .. } => 4,
            Pawn { .. } => 5,
        }) + if self.side == Black { 6 } else { 0 }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Default, Debug)]
pub enum Space {
    #[default]
    Hole,
    Square {
        slot: Option<Piece>,
        promotes: [bool; 2],
    },
}
use Space::*;

#[derive(Clone, PartialEq, Eq, Default, Debug)]
pub struct Board {
    pub spaces: Array2<Space>,
    pub side: Side,
    pub captured: Vec<Piece>,
}

impl Board {
    pub fn from_strings<'a>(board_string: &'a str, promotion_string: &'a str) -> Result<Self, &'static str> {
        let get_byte_rows = |s: &'a str| {
            s.lines()
                .rev()
                .map(|row| row.trim())
                .filter(|row| !row.is_empty())
                .map(|row| row.as_bytes())
        };

        if !(board_string.is_ascii() && promotion_string.is_ascii()) {
            return Err("Non-ASCII characters in board strings");
        }
        let (b_rows, p_rows) = (get_byte_rows(board_string), get_byte_rows(promotion_string));
        let bh = b_rows.clone().count();

        if bh != p_rows.clone().count() {
            return Err("Board strings have different row counts");
        }
            
        let bw = if let Some(first_row) = b_rows.clone().next() {
            first_row.len()
        } else {
            return Err("Board strings are empty");
        };

        let rows = b_rows.zip(p_rows);
        if !rows.clone().all(
            |(b_row, p_row)| b_row.len() == bw && b_row.len() == p_row.len()
        ) {
            return Err("Inconsistent row sizes across board strings");
        }

        Ok(Board {
            spaces: Array2::from_shape_vec(
                (bh, bw),
                rows.flat_map(|(b_row, p_row)| {
                    b_row
                        .iter()
                        .zip(p_row.iter())
                        .map(|(&square, &prom)| match square {
                            b'X' => Hole,
                            square => Square {
                                slot: match square {
                                    b'K' => Some(Piece {
                                        side: White,
                                        model: King { can_castle: true },
                                    }),
                                    b'Q' => Some(Piece {
                                        side: White,
                                        model: Queen,
                                    }),
                                    b'R' => Some(Piece {
                                        side: White,
                                        model: Rook { can_castle: true },
                                    }),
                                    b'B' => Some(Piece {
                                        side: White,
                                        model: Bishop,
                                    }),
                                    b'N' => Some(Piece {
                                        side: White,
                                        model: Knight,
                                    }),
                                    b'P' => Some(Piece {
                                        side: White,
                                        model: Pawn {
                                            can_dash: true,
                                            just_dashed: false,
                                        },
                                    }),
                                    b'k' => Some(Piece {
                                        side: Black,
                                        model: King { can_castle: true },
                                    }),
                                    b'q' => Some(Piece {
                                        side: Black,
                                        model: Queen,
                                    }),
                                    b'r' => Some(Piece {
                                        side: Black,
                                        model: Rook { can_castle: true },
                                    }),
                                    b'b' => Some(Piece {
                                        side: Black,
                                        model: Bishop,
                                    }),
                                    b'n' => Some(Piece {
                                        side: Black,
                                        model: Knight,
                                    }),
                                    b'p' => Some(Piece {
                                        side: Black,
                                        model: Pawn {
                                            can_dash: true,
                                            just_dashed: false,
                                        },
                                    }),
                                    _ => None,
                                },
                                promotes: match prom {
                                    b'P' | b'w' | b'W' => [true, false],
                                    b'p' | b'b' | b'B' => [false, true],
                                    b'*' => [true, true],
                                    _ => [false, false],
                                },
                            },
                        })
                })
                .collect(),
            )
            .unwrap()
            .reversed_axes(),
            captured: vec![],
            side: White,
        })
    }
}
