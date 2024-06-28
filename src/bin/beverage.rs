// The mothership.

use bevy::prelude::*;

//use ease::Ease;
use sickle_ui::{
    dev_panels::{
        hierarchy::{ HierarchyTreeViewPlugin, UiHierarchyExt },
        scene_view::{ SceneView, SceneViewPlugin, SpawnSceneViewPreUpdate, UiSceneViewExt },
    },
    prelude::*,
    ui_commands::SetCursorExt,
    SickleUiPlugin,
};

use beverage::{ framework::*, setup::setup };

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Beverage - Also Available As A T-Shirt".into(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            })
        )
        .add_plugins(SickleUiPlugin)
        .init_resource::<CurrentPage>()
        .init_resource::<IconCache>()
        .init_state::<Page>()
        .add_plugins(HierarchyTreeViewPlugin)
        .add_plugins(SceneViewPlugin)
        .add_systems(Startup, setup.in_set(UiStartupSet))
        .add_systems(OnEnter(Page::Layout), layout_editor)
        .add_systems(OnExit(Page::Layout), clear_content_on_menu_change)
        //.add_systems(OnEnter(Page::Playground), interaction_handler)
        .add_systems(OnExit(Page::Playground), clear_content_on_menu_change)
        .add_systems(PreUpdate, exit_app_on_menu_item)
        .add_systems(
            PreUpdate,
            (spawn_hierarchy_view, despawn_hierarchy_view).after(SpawnSceneViewPreUpdate)
        )
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
        )
        .run();
}

// BEGIN: sickle_editor systems
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
// END: sickle_editor systems

fn layout_editor(root_node: Query<Entity, With<EditorContainer>>, mut commands: Commands) {
    let root_entity = root_node.single();

    commands
        .ui_builder(root_entity)
        .row(|row| {
            row.docking_zone_split(
                SizedZoneConfig {
                    size: 75.0,
                    ..default()
                },
                |left_side| {
                    left_side.docking_zone_split(
                        SizedZoneConfig {
                            size: 75.0,
                            ..default()
                        },
                        |left_side_top| {
                            left_side_top.docking_zone(
                                SizedZoneConfig {
                                    size: 25.0,
                                    ..default()
                                },
                                true,
                                |tab_container| {
                                    tab_container.add_tab("Hierarchy".into(), |panel| {
                                        panel.insert(HierarchyPanel);
                                    });
                                    tab_container.add_tab("Tab 3".into(), |panel| {
                                        panel.label(LabelConfig {
                                            label: "Panel 3".into(),
                                            ..default()
                                        });
                                    });
                                }
                            );
                            left_side_top.docking_zone(
                                SizedZoneConfig {
                                    size: 75.0,
                                    ..default()
                                },
                                false,
                                |tab_container| {
                                    tab_container.add_tab("Scene View".into(), |panel| {
                                        panel.scene_view("examples/Low_poly_scene.gltf#Scene0");
                                    });
                                    tab_container.add_tab("Tab 2".into(), |panel| {
                                        panel.label(LabelConfig {
                                            label: "Panel 2".into(),
                                            ..default()
                                        });
                                    });
                                    tab_container.add_tab("Tab 3".into(), |panel| {
                                        panel.label(LabelConfig {
                                            label: "Panel 3".into(),
                                            ..default()
                                        });
                                    });
                                }
                            );
                        }
                    );

                    left_side.docking_zone(
                        SizedZoneConfig {
                            size: 25.0,
                            ..default()
                        },
                        true,
                        |tab_container| {
                            tab_container.add_tab("Systems".into(), |panel| {
                                panel.label(LabelConfig {
                                    label: "Systems".into(),
                                    ..default()
                                });
                            });
                            tab_container.add_tab("Tab 6".into(), |panel| {
                                panel.label(LabelConfig {
                                    label: "Panel 6".into(),
                                    ..default()
                                });
                            });
                        }
                    );
                }
            );

            row.docking_zone_split(
                SizedZoneConfig {
                    size: 25.0,
                    ..default()
                },
                |right_side| {
                    right_side.docking_zone(
                        SizedZoneConfig {
                            size: 25.0,
                            ..default()
                        },
                        true,
                        |tab_container| {
                            tab_container.add_tab("Placeholder".into(), |placeholder| {
                                placeholder.style().padding(UiRect::all(Val::Px(10.0)));

                                placeholder.row(|row| {
                                    row.checkbox(None, false);
                                    row.radio_group(vec!["Light", "Dark"], 1, false);
                                });

                                placeholder.row(|row| {
                                    row.style().justify_content(JustifyContent::SpaceBetween);
                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast"
                                        ],
                                        None
                                    );

                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast"
                                        ],
                                        None
                                    );
                                });

                                /*
                                placeholder.outlined_block();
                                placeholder.atlas_example();
                                */

                                placeholder.row(|row| {
                                    row.style().justify_content(JustifyContent::SpaceBetween);
                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast"
                                        ],
                                        None
                                    );
                                    row.checkbox(None, false);
                                    row.dropdown(
                                        vec![
                                            "Standard",
                                            "Medium Contrast",
                                            "High Contrast - High Contrast"
                                        ],
                                        None
                                    );
                                });
                            });

                            tab_container.add_tab("Sliders".into(), |slider_tab| {
                                slider_tab
                                    .row(|row| {
                                        row.slider(
                                            SliderConfig::vertical(
                                                String::from("Slider"),
                                                0.0,
                                                5.0,
                                                2.0,
                                                true
                                            )
                                        );

                                        row.slider(
                                            SliderConfig::vertical(None, 0.0, 5.0, 2.0, true)
                                        );

                                        row.slider(
                                            SliderConfig::vertical(
                                                String::from("Slider"),
                                                0.0,
                                                5.0,
                                                2.0,
                                                false
                                            )
                                        );

                                        row.slider(
                                            SliderConfig::vertical(None, 0.0, 5.0, 2.0, false)
                                        );
                                    })
                                    .style()
                                    .height(Val::Percent(50.0));

                                slider_tab
                                    .column(|row| {
                                        row.slider(
                                            SliderConfig::horizontal(
                                                String::from("Slider"),
                                                0.0,
                                                5.0,
                                                2.0,
                                                true
                                            )
                                        );
                                        row.slider(
                                            SliderConfig::horizontal(None, 0.0, 5.0, 2.0, true)
                                        );
                                        row.slider(
                                            SliderConfig::horizontal(
                                                String::from("Slider"),
                                                0.0,
                                                5.0,
                                                2.0,
                                                false
                                            )
                                        );
                                        row.slider(
                                            SliderConfig::horizontal(None, 0.0, 5.0, 2.0, false)
                                        );
                                    })
                                    .style()
                                    .justify_content(JustifyContent::End)
                                    .height(Val::Percent(50.0))
                                    .width(Val::Percent(100.0));
                            });
                        }
                    );
                }
            );
        })
        .style()
        .height(Val::Percent(100.0));
}

/*
fn interaction_showcase(root_node: Query<Entity, With<EditorContainer>>, mut commands: Commands) {
    let root_entity = root_node.single();

    commands.ui_builder(root_entity).column(|_column| {
        // Test here simply by calling methods on the `column`
    });
}
*/

fn handle_theme_data_update(
    theme_data: Res<ThemeData>,
    mut q_theme_switch: Query<&mut RadioGroup, With<ThemeSwitch>>,
    mut q_theme_contrast_select: Query<&mut Dropdown, With<ThemeContrastSelect>>
) {
    if theme_data.is_changed() {
        let Ok(mut theme_switch) = q_theme_switch.get_single_mut() else {
            return;
        };

        let Ok(mut theme_contrast_select) = q_theme_contrast_select.get_single_mut() else {
            return;
        };

        match theme_data.active_scheme {
            Scheme::Light(contrast) => {
                theme_switch.select(0);
                match contrast {
                    Contrast::Standard => theme_contrast_select.set_value(0),
                    Contrast::Medium => theme_contrast_select.set_value(1),
                    Contrast::High => theme_contrast_select.set_value(2),
                }
            }
            Scheme::Dark(contrast) => {
                theme_switch.select(1);
                match contrast {
                    Contrast::Standard => theme_contrast_select.set_value(0),
                    Contrast::Medium => theme_contrast_select.set_value(1),
                    Contrast::High => theme_contrast_select.set_value(2),
                }
            }
        };
    }
}

fn handle_theme_switch(
    mut theme_data: ResMut<ThemeData>,
    q_theme_switch: Query<&RadioGroup, (With<ThemeSwitch>, Changed<RadioGroup>)>,
    q_theme_contrast_select: Query<&Dropdown, With<ThemeContrastSelect>>
) {
    let Ok(theme_switch) = q_theme_switch.get_single() else {
        return;
    };

    let Ok(theme_contrast_select) = q_theme_contrast_select.get_single() else {
        return;
    };

    if let Some(scheme) = get_selected_scheme(theme_switch, theme_contrast_select) {
        if theme_data.active_scheme != scheme {
            theme_data.active_scheme = scheme;
        }
    }
}

fn handle_theme_contrast_select(
    mut theme_data: ResMut<ThemeData>,
    q_theme_switch: Query<&RadioGroup, With<ThemeSwitch>>,
    q_theme_contrast_select: Query<&Dropdown, (With<ThemeContrastSelect>, Changed<Dropdown>)>
) {
    let Ok(theme_contrast_select) = q_theme_contrast_select.get_single() else {
        return;
    };

    let Ok(theme_switch) = q_theme_switch.get_single() else {
        return;
    };

    if let Some(scheme) = get_selected_scheme(theme_switch, theme_contrast_select) {
        if theme_data.active_scheme != scheme {
            theme_data.active_scheme = scheme;
        }
    }
}

fn get_selected_scheme(
    theme_switch: &RadioGroup,
    theme_contrast_select: &Dropdown
) -> Option<Scheme> {
    let contrast = match theme_contrast_select.value() {
        Some(index) =>
            match index {
                0 => Contrast::Standard,
                1 => Contrast::Medium,
                2 => Contrast::High,
                _ => Contrast::Standard,
            }
        None => Contrast::Standard,
    };

    if let Some(index) = theme_switch.selected() {
        let scheme = match index {
            0 => Scheme::Light(contrast),
            1 => Scheme::Dark(contrast),
            _ => Scheme::Light(contrast),
        };

        Some(scheme)
    } else {
        None
    }
}
