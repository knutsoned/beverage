// The mothership.

use bevy::prelude::*; //, winit::WinitWindows };

use bevy_fluent::{ FluentPlugin, Locale };
use unic_langid::LanguageIdentifier;

use sickle_ui::{
    dev_panels::{
        hierarchy::{ HierarchyTreeViewPlugin, UiHierarchyExt },
        scene_view::{ SceneView, SceneViewPlugin, SpawnSceneViewPreUpdate },
    },
    prelude::*,
    ui_commands::SetCursorExt,
    SickleUiPlugin,
};

//use winit::window::Icon;

use beverage::{
    framework::*,
    l10n::{ self, handle_locale_select },
    layout::{ editor, page::camera_control::CameraControlPlugin },
    prelude::DEFAULT_LOCALE,
    setup,
    theme::{ handle_theme_contrast_select, handle_theme_data_update, handle_theme_switch },
};

fn main() {
    let default_li = DEFAULT_LOCALE.parse::<LanguageIdentifier>().expect(
        "Invalid default LanguageIdentifier"
    );
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
            FluentPlugin,
            SickleUiPlugin,
        ))
        .init_resource::<CurrentPage>()
        .insert_resource(Locale::new(default_li))
        .init_state::<EditorState>()
        .init_state::<Page>()
        // sickle plugin for the remote camera demo
        .add_plugins(CameraControlPlugin)
        // sickle plugin for the FPO tree view on the left side
        .add_plugins(HierarchyTreeViewPlugin)
        // sickle plugin for the FPO scene viewer
        .add_plugins(SceneViewPlugin)
        // FIXME why doesn't this work?
        //.add_systems(PreStartup, set_window_icon)
        // init fluent l10n
        .add_systems(OnEnter(EditorState::Loading), l10n::setup)
        // spawn UI camera and top-level UI container
        .add_systems(OnExit(EditorState::Loading), setup::on_load.in_set(UiStartupSet))
        // handle selecting a new locale from the language switcher
        .add_systems(OnEnter(EditorState::SwitchLocale), l10n::switch_locale)
        // rebuild the entire contents of the top-level UI container after changing the locale
        .add_systems(OnExit(EditorState::SwitchLocale), setup::on_rebuild)
        // check to see if the AssetServer is done loading the locales folder
        .add_systems(Update, l10n::update.run_if(in_state(EditorState::Loading)))
        // layout the editor content when a page is selected
        .add_systems(OnEnter(Page::CameraControl), editor::layout)
        // clean up after a different page is selected
        .add_systems(OnExit(Page::CameraControl), clear_content_on_menu_change)
        // TODO needs to be a way to just layout the content area and not the entire editor
        // at minimum, need to figure out if a hierarchy view and scene view both represent the same data,
        // if the scene editor is swapped with another widget, what happens?
        // also need to support adding new tabs to the containers and removing them
        .add_systems(OnEnter(Page::SceneEditor), editor::layout)
        .add_systems(OnExit(Page::SceneEditor), clear_content_on_menu_change)
        // handle selecting Exit from the Editor menu
        .add_systems(PreUpdate, exit_app_on_menu_item)
        // sickle internals
        .add_systems(
            PreUpdate,
            (spawn_hierarchy_view, despawn_hierarchy_view)
                .after(SpawnSceneViewPreUpdate)
                .run_if(in_state(EditorState::Running))
        )
        // update_current_page checks the menu for updates while the rest handle radios and dropdowns
        .add_systems(
            Update,
            (
                update_current_page,
                handle_locale_select,
                handle_theme_data_update,
                handle_theme_switch,
                handle_theme_contrast_select,
            )
                .chain()
                .after(WidgetLibraryUpdate)
                .run_if(in_state(EditorState::Running))
        )
        .run();
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

// BEGIN: sickle editor example systems
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

fn spawn_hierarchy_view(
    q_added_scene_view: Query<&SceneView, Added<SceneView>>,
    q_hierarchy_panel: Query<Entity, With<HierarchyPanel>>,

    mut commands: Commands
) {
    if let Some(scene_view) = (&q_added_scene_view).into_iter().next() {
        let Ok(container) = q_hierarchy_panel.get_single() else {
            return;
        };

        commands.entity(container).despawn_descendants();
        commands.ui_builder(container).hierarchy_for(scene_view.asset_root());
    }
}

fn despawn_hierarchy_view(
    q_hierarchy_panel: Query<Entity, With<HierarchyPanel>>,
    q_removed_scene_view: RemovedComponents<SceneView>,
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
