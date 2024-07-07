// basically every "full stack" framework has a router.

use bevy::prelude::*;

use crate::{
    demo::camera_control::widget::{ despawn_camera_tree_view, SpawnCameraControlPreUpdate },
    framework::*,
};

// this is where we respond to navigation changes from a top-level menu or dialog click.
pub struct EditorRouterPlugin {}

impl Plugin for EditorRouterPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app
            // TODO the router must manage the lifecycle of `Page` (activity) transitions

            // an activity plugin should be able to register its own routes
            //.add_systems(OnEnter(Page::SceneEditor), editor::layout)
            //.add_systems(OnExit(Page::SceneEditor), clear_content_on_menu_change)

            // also need a workflow to save a snapshot of the current editor arrangement as a "preset" (activity + instance)
            .add_systems(
                PreUpdate,
                despawn_camera_tree_view
                    /*, spawn_camera_tree_view*/ .after(SpawnCameraControlPreUpdate)
                    // need a run condition to check current page value against registered plugins
                    .run_if(in_state(EditorState::Running))
            );
    }
}
