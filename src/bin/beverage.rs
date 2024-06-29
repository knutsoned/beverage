// The mothership.

use bevy::prelude::*;

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

use beverage::{
    framework::*,
    l10n::{ self, handle_locale_select },
    layout::root::layout_editor,
    prelude::DEFAULT_LOCALE,
    setup,
    theme::{ handle_theme_contrast_select, handle_theme_data_update, handle_theme_switch },
    EditorState,
};

fn main() {
    let default_li = "fr-FR".parse::<LanguageIdentifier>().expect(
        //DEFAULT_LOCALE.parse::<LanguageIdentifier>().expect(
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
        .init_resource::<IconCache>()
        .insert_resource(Locale::new(default_li))
        .init_state::<EditorState>()
        .init_state::<Page>()
        .add_plugins(HierarchyTreeViewPlugin)
        .add_plugins(SceneViewPlugin)
        .add_systems(OnEnter(EditorState::Loading), l10n::setup)
        .add_systems(Update, l10n::update.run_if(in_state(EditorState::Loading)))
        .add_systems(OnEnter(EditorState::Running), setup::on_load.in_set(UiStartupSet))
        .add_systems(OnEnter(Page::SceneEditor), layout_editor)
        .add_systems(OnExit(Page::SceneEditor), clear_content_on_menu_change)
        //.add_systems(OnEnter(Page::Playground), interaction_showcase)
        //.add_systems(OnExit(Page::Playground), clear_content_on_menu_change)
        .add_systems(PreUpdate, exit_app_on_menu_item)
        .add_systems(
            PreUpdate,
            (spawn_hierarchy_view, despawn_hierarchy_view)
                .after(SpawnSceneViewPreUpdate)
                .run_if(in_state(EditorState::Running))
        )
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

/*
fn interaction_showcase(root_node: Query<Entity, With<EditorContainer>>, mut commands: Commands) {
    let root_entity = root_node.single();

    commands.ui_builder(root_entity).column(|_column| {
        // Test here simply by calling methods on the `column`
    });
}
*/
// END: sickle editor example internal systems
