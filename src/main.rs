mod utils;
mod board; use board::*;
mod sets; use sets::*;
mod ui; use ui::*;
mod logic; use logic::*;
use bevy::prelude::*;

const BOARD_STR: &str = "
rnbqkbnr
pppppppp
________
___XX_Q_
___XX___
________
PP____PP
R___K__R
";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SetsPlugin)
        .add_plugin(BoardPlugin { board_string: BOARD_STR })
        .add_plugin(UIPlugin)
        .add_plugin(LogicPlugin)
        .run();
}