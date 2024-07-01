use bevy::prelude::*;

use bevy_fluent::Localization;

use sickle_ui::prelude::*;

use crate::{ framework::*, layout::page::camera_control::UiCameraControlExt };

pub fn layout(
    root_node: Query<Entity, With<EditorContainer>>,
    l10n: Res<Localization>,
    mut commands: Commands
) {
    warn!("root::layout_editor");
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
                                    tab_container.add_tab(l10n.lbl("Relationships"), |panel| {
                                        panel.insert(HierarchyPanel);
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
                                    tab_container.add_tab(l10n.lbl("CameraControl"), |panel| {
                                        panel.camera_control();
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
                            tab_container.add_tab(l10n.lbl("Systems"), |_panel| {});
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
                            tab_container.add_tab(l10n.lbl("Placeholder"), |placeholder| {
                                placeholder.style().padding(UiRect::all(Val::Px(10.0)));
                            });
                        }
                    );
                }
            );
        })
        .style()
        .height(Val::Percent(100.0));
}
