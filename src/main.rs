mod utils;
mod board; use board::*;
mod sets; use sets::*;
mod ui; use ui::*;
mod logic; use logic::*;
use bevy::prelude::*;

const BOARD_STR: &str = "
rkbqkbkr
pppppppp
________
___XX___
___XX___
________
PPPPPPPP
RKBQKBKR
";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SetsPlugin)
        .add_plugin(BoardPlugin { board_string: BOARD_STR })
        .add_plugin(UIPlugin)
        .run();
}