// The mothership.

use bevy::{ prelude::*, winit::* };

use sickle_ui::{ prelude::*, ui_commands::SetCursorExt, SickleUiPlugin };

//use winit::window::Icon;

use beverage::{
    framework::*,
    input::InputPlugin,
    l10n::EditorLocalePlugin,
    layout::editor,
    remote::camera_control::CameraControlRemotePlugin,
    setup::{ self, spawn_footer },
    theme::*,
    widget::camera_control::*,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Beverage - Also Available As A T-Shirt".into(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            }),
            EditorPlugin,
        ))
        .run();
}

struct EditorPlugin;

impl Plugin for EditorPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins((EditorLocalePlugin, SickleUiPlugin))
            // Only run the app when there is user input. This will significantly reduce CPU/GPU use.
            .insert_resource(WinitSettings::desktop_app())

            // This plugin maps inputs to an input-type agnostic action-state
            // We need to provide it with an enum which stores the possible actions a player could take
            .add_plugins(InputPlugin)

            // page widgets (i.e. "main" content)

            // sickle widget plugin for the remote camera demo
            .add_plugins(CameraControlPlugin)

            // BRP plugin to sync server camera with local viewport
            .add_plugins(CameraControlRemotePlugin)

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

            // also this should go away once we can hot reload localized strings in text labels
            .add_systems(OnExit(EditorState::SwitchLocale), setup::on_rebuild)

            // for now just make sure the footer refreshes every time the app enters the running state
            .add_systems(OnEnter(EditorState::Running), spawn_footer.after(setup::on_rebuild))

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
                OnEnter(RemoteConnectionState::Connected),
                spawn_footer.run_if(in_state(EditorState::Running))
            )

            // TODO these 2 things belong in the router
            // layout the editor content when a page is selected
            .add_systems(OnEnter(Page::CameraControl), editor::layout)

            // clean up after a different page is selected
            .add_systems(OnExit(Page::CameraControl), clear_content_on_menu_change)
            // TODO needs to be a way to just change the content area and not the entire editor

            // at minimum, need to figure out if a hierarchy view and scene view both represent the same data,
            // and the scene editor is swapped with another page, what happens?

            // the basic idea of a page is defining a collection of widgets that can go in each pane

            // the mechanics of how to deal with then despawning a page with shared widgets is unclear

            // also need to support adding new tabs to the containers and removing them
            //.add_systems(OnEnter(Page::SceneEditor), editor::layout)
            //.add_systems(OnExit(Page::SceneEditor), clear_content_on_menu_change)
            // handle selecting Exit from the Editor menu
            .add_systems(PreUpdate, exit_app_on_menu_item)
            // need a better way to group views in different containers that work together

            // also need a view to save a snapshot of the current editor arrangement as a "preset"
            .add_systems(
                PreUpdate,
                despawn_camera_tree_view
                    /*, spawn_camera_tree_view*/ .after(SpawnCameraControlPreUpdate)
                    .run_if(in_state(EditorState::Running))
            )

            // update_current_page checks the menu for updates while the rest handle radios and dropdowns
            .add_systems(
                Update,
                (
                    update_current_page,
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

// from https://bevy-cheatbook.github.io/window/icon.html
/*
fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>
) {
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        // FIXME this may not work, especially when packaged for release
        //let path = std::env::current_dir().unwrap().join("assets/textures/ic_launcher.png");
        //warn!("PATH: {}", path.display());
        let path = "assets/textures/ic_launcher.png";
        let image = image::open(path).expect("Failed to open icon path").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
*/

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
    let root_entity = root_node.single();
    commands.entity(root_entity).despawn_descendants();
    commands.set_cursor(CursorIcon::Default);
}

/*
fn spawn_camera_tree_view(
    q_added_camera_control: Query<&CameraControl, Added<CameraControl>>,
    q_tree_view_panel: Query<Entity, With<TreeViewPanel>>,

    mut commands: Commands
) {
    if let Some(camera_control) = (&q_added_camera_control).into_iter().next() {
        let Ok(container) = q_tree_view_panel.get_single() else {
            return;
        };

        commands.entity(container).despawn_descendants();
        commands.ui_builder(container).tree_for(camera_control.asset_root());
    }
}
*/

fn despawn_camera_tree_view(
    q_hierarchy_panel: Query<Entity, With<TreeViewPanel>>,
    q_removed_scene_view: RemovedComponents<CameraControl>,
    mut commands: Commands
) {
    let Ok(container) = q_hierarchy_panel.get_single() else {
        return;
    };

    if !q_removed_scene_view.is_empty() {
        commands.entity(container).despawn_descendants();
    }
}
// END: sickle editor example internal systems
