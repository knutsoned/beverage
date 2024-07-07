use bevy::prelude::*;

use sickle_ui::{ prelude::*, ui_commands::SetCursorExt, SickleUiPlugin };

use framework::*;
use input::EditorInputPlugin;
use layout::{ editor, footer::spawn_footer };
use locale::EditorLocalePlugin;
use theme::*;

pub mod construct;
pub mod framework;
pub mod input;
pub mod locale;
pub mod layout;
pub mod logging;
pub mod remote;
pub mod router;
pub mod setup;
pub mod signals;
pub mod theme;
pub mod undo;
pub mod widget;

// plugins will want to have the domain objects available
pub mod prelude {
    pub use crate::{ EditorPlugin, framework::* };
}

pub struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EditorLocalePlugin, SickleUiPlugin))
            // this section sets up all the primary functionality of the editor

            // the convention of beginning a symbol with `Editor` signifies it is provided internally

            // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
            //.insert_resource(WinitSettings::desktop_app()) // this line causes SEVERE lag

            // This plugin maps inputs to an input-type agnostic action-state
            // We need to provide it with an enum which stores the possible actions a player could take
            .add_plugins(EditorInputPlugin)

            // page widgets (i.e. "main" content)

            // the next few are tracking navigation
            .init_resource::<CurrentPage>()
            .init_state::<EditorState>()
            .init_state::<Page>()
            // initialize custom types for reflection
            // (anything that needs to go over BRP, be saved to a file, or be otherwise serialized)
            .register_type::<RemoteFpsCounter>()
            .register_type::<DespawnRemoteFpsCounter>()

            // FIXME why doesn't this work?
            //.add_systems(PreStartup, set_window_icon)

            // spawn UI camera and top-level UI container
            .add_systems(OnExit(EditorState::Loading), setup::on_load.in_set(UiStartupSet))

            // rebuild the entire contents of the top-level UI container after changing the locale

            // this goes here instead of the plugin because the plugin shouldn't need to know how to rebuild the UI
            .add_systems(OnEnter(EditorState::SwitchLocale), clear_content_on_menu_change)

            // also this should go away once we can hot reload localized strings in text labels
            .add_systems(OnExit(EditorState::SwitchLocale), setup::on_rebuild)

            // for now just make sure the footer refreshes every time the app enters the running state
            .add_systems(OnEnter(EditorState::Running), spawn_footer)

            // is there a better way to do this?
            .add_systems(
                OnEnter(RemoteConnectionState::Disconnected),
                spawn_footer.run_if(in_state(EditorState::Running))
            )
            .add_systems(
                OnEnter(RemoteConnectionState::Connecting),
                spawn_footer.run_if(in_state(EditorState::Running))
            )
            .add_systems(
                OnEnter(RemoteConnectionState::Checking),
                spawn_footer.run_if(in_state(EditorState::Running))
            )
            .add_systems(
                OnEnter(RemoteConnectionState::Connected),
                spawn_footer.run_if(in_state(EditorState::Running))
            )

            // TODO these 2 things belong in the router
            // layout the editor content when a page is selected
            .add_systems(OnEnter(Page::CameraControl), editor::layout)

            // clean up after a different page is selected
            .add_systems(OnExit(Page::CameraControl), clear_content_on_menu_change)
            // TODO needs to be a way to just change the content area and not the entire editor

            // the basic idea of an activity is defining a collection of widgets that can go in each pane

            // example activities: edit a scene, design a UI widget, import and arrange a glTF file from Blender

            // at minimum, need to figure out if a tree view and scene view both represent the same data,
            // and the scene view is swapped with another activity, what happens?

            // if the new widget doesn't use a tree view, what happens to the old one?

            // if it does use a tree view, does each pane maintain a stack? how to navigate back?

            // the mechanics of how to deal with then despawning an activity with shared widgets is unclear

            // also need to support adding new tabs to the containers and removing them

            // handle selecting Exit from the Editor menu
            .add_systems(PreUpdate, exit_app_on_menu_item)

            // update_current_page checks the menu for updates while the rest handle radios and dropdowns
            .add_systems(
                Update,
                (
                    update_current_page,
                    // TOODO these should go in the theme module
                    handle_theme_data_update,
                    handle_theme_switch,
                    handle_theme_contrast_select,
                )
                    .chain()
                    .after(WidgetLibraryUpdate)
                    .run_if(in_state(EditorState::Running))
            );
    }
}

// BEGIN: sickle editor example systems (menu navigation)
fn exit_app_on_menu_item(
    q_menu_items: Query<&MenuItem, (With<ExitAppButton>, Changed<MenuItem>)>,
    q_windows: Query<Entity, With<Window>>,
    mut commands: Commands
) {
    let Ok(item) = q_menu_items.get_single() else {
        return;
    };

    if item.interacted() {
        for entity in &q_windows {
            commands.entity(entity).remove::<Window>();
        }
    }
}

fn update_current_page(
    mut next_state: ResMut<NextState<Page>>,
    q_menu_items: Query<(&Page, &MenuItem), Changed<MenuItem>>
) {
    for (menu_type, menu_item) in &q_menu_items {
        if menu_item.interacted() {
            next_state.set(*menu_type);
        }
    }
}

fn clear_content_on_menu_change(
    root_node: Query<Entity, With<EditorContainer>>,
    mut commands: Commands
) {
    // need to run this when we enter SwitchLocale I think
    let root_entity = root_node.single();
    commands.entity(root_entity).despawn_descendants();
    commands.set_cursor(CursorIcon::Default);
}
// END: sickle editor example internal systems
