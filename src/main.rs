mod board;
use board::*;
mod sets;
use sets::*;
mod ui;
use ui::*;
mod logic;
use logic::*;
mod turns;
use bevy::prelude::*;
use turns::*;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);

    fn poll_restart() -> bool;
    fn get_pieces_string() -> String;
    fn get_promotions_string() -> String;
    fn get_bottom_side() -> bool;
}

fn setup_initial_board(
    mut history: ResMut<TurnHistory>,
    mut displayed_turn: ResMut<DisplayedTurn>
) {
    let initial_board = match Board::from_strings(&get_pieces_string(), &get_promotions_string()) {
        Ok(board) => board,
        Err(err_str) => { alert(err_str); return; }
    };

    history.clear();
    history.push_back(Turn {
        possible_moves: compute_possible_moves(&initial_board, true),
        board: initial_board,
        previous_move: Move {
            kind: MoveKind::Skip,
            ..default()
        },
    });

    *displayed_turn = DisplayedTurn(0);
}

fn poll_js(history: ResMut<TurnHistory>, displayed_turn: ResMut<DisplayedTurn>) {
    if poll_restart() {
        setup_initial_board(history, displayed_turn);
    }
}

pub fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(
                WindowPlugin {
                    primary_window: Some(Window {
                        canvas: Some(String::from("#bevy_game")),
                        fit_canvas_to_parent: true,
                        prevent_default_event_handling: false,
                        ..default()
                    }),
                    ..default()
                }
            ),
            SetsPlugin,
            UIPlugin,
            LogicPlugin,
            TurnsPlugin
        ))
        .add_systems(Startup, setup_initial_board.in_set(GameSet::BoardSetup))
        .add_systems(Main, poll_js)
        .run();
}
