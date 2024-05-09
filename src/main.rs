mod board;
mod ui;
mod logic;
mod turns;
#[cfg(target_family = "wasm")]
mod io_wasm;
#[cfg(not(target_family = "wasm"))]
mod io_standard;

use board::*;
use ui::*;
use logic::*;
use bevy::{asset::AssetMetaCheck, prelude::*};
use turns::*;
#[cfg(target_family = "wasm")]
use io_wasm::*;
#[cfg(not(target_family = "wasm"))]
use io_standard::*;

fn setup_initial_board(
    mut turns: ResMut<Turns>,
    mut display_state: ResMut<BoardDisplayState>
) {
    let initial_board = match Board::from_strings(&get_pieces_string(), &get_promotions_string()) {
        Ok(board) => board,
        Err(err_str) => { alert(err_str); return; }
    };

    turns.history.clear();
    turns.history.push_back(Turn {
        possible_moves: compute_possible_moves(&initial_board, true),
        board: initial_board,
        previous_move: Move {
            kind: MoveKind::Skip,
            ..default()
        },
    });

    *display_state = BoardDisplayState {
        displayed_turn: 0,
        bottom_side: match get_bottom_side().to_lowercase().as_str() { "white" => Side::White, _ => Side::Black },
        ..default()
    };
}

fn poll_io(
    turns: ResMut<Turns>,
    display_state: ResMut<BoardDisplayState>
) {
    if poll_restart() {
        setup_initial_board(turns, display_state);
    }
}

pub fn main() {
    App::new()
        .insert_resource(AssetMetaCheck::Never)
        .add_plugins((
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some(String::from("#bevy_game")),
                        //fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }
            ),
            UIPlugin,
            LogicPlugin,
            TurnsPlugin
        ))
        .add_systems(PreStartup, setup_initial_board)
        .add_systems(Update, poll_io)
        .run();
}
