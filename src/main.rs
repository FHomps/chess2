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
PP___r_P
R___K__R
";

const PROM_STR: &str = "
WWWWWWWW
________
________
___XX___
___XX___
________
________
bbbbbbbb
";

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(SetsPlugin)
        .insert_resource(Board::from_strings(BOARD_STR, PROM_STR))
        .add_plugin(UIPlugin)
        .add_plugin(LogicPlugin)
        .run();
}