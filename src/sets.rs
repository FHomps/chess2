use bevy::prelude::*;

pub struct SetsPlugin;

impl Plugin for SetsPlugin {
    fn build(&self, app: &mut App) {
        // configure_set is applied to the default schedule if called directly by app (will not apply to startup systems),
        // thus the need to call it via edit_schedule
        app.edit_schedule(CoreSchedule::Startup, 
            |s| { s.configure_set(UISetup.after(BoardSetup)); }
        );

        // apply commands between board setup and UI setup
        // This populates the piece entities so that UI setup may give them sprite components
        app.add_startup_system(
            apply_system_buffers
            .after(BoardSetup)
            .before(UISetup)
        );
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
pub enum GameSet {
    BoardSetup,
    UISetup,
    PieceSelection,
}
use GameSet::*;