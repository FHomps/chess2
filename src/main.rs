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

const BOARD_STR: &str = "
rnbqkbnr
pppppppp
________
___XX___
________
PPPPPPPP
RNBQKBNR
";

const PROM_STR: &str = "
WWWWWWWW
________
________
___XX___
________
________
bbbbbbbb
";

fn setup_initial_board(mut history: ResMut<TurnHistory>) {
    let initial_board = Board::from_strings(BOARD_STR, PROM_STR);

    history.push_back(Turn {
        possible_moves: compute_possible_moves(&initial_board, true),
        board: initial_board,
        previous_move: Move {
            kind: MoveKind::Skip,
            ..default()
        },
    });
}

fn main() {
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
        .run();
}
