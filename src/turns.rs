use std::collections::HashMap;
use std::collections::VecDeque;

use bevy::prelude::*;

use crate::board::*;
use crate::logic::*;

pub struct TurnsPlugin;

impl Plugin for TurnsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Turns::default());
    }
}

#[derive(Default)]
pub struct Turn {
    pub previous_move: Move,
    pub board: Board,
    pub possible_moves: HashMap<Coords, Vec<Move>>,
}

// Queue of all the turns up to and including the one currently in play
#[derive(Resource, Default)]
pub struct Turns {
    pub history: VecDeque<Turn>
}