use bevy::prelude::*;

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        // configure_set is applied to the default schedule if called directly by app (will not apply to startup systems),
        // thus the need to call it via edit_schedule
        app.edit_schedule(Startup, |s| {
            s.configure_set(UISetup.after(BoardSetup));
        });
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    BoardSetup,
    UISetup,
}
use GameSet::*;
