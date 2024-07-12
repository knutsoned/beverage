// support for undo/redo (probably lifted from space_editor)

use bevy::prelude::*;

/// The history service manages action histories for activities.
#[derive(Resource, Default, Debug)]
pub struct HistoryService {}
