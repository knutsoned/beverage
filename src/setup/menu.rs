use bevy::prelude::*;

use sickle_ui::prelude::*;

use super::{ EditorContainer, ExitAppButton, Page, ThemeContrastSelect, ThemeSwitch };

pub fn build_menu(root_entity: Entity, commands: &mut Commands) {
    commands.ui_builder(root_entity).column(|column| {
        column.style().width(Val::Percent(100.0)).background_color(Color::srgb(0.15, 0.155, 0.16));

        column.menu_bar(|bar| {
            bar.menu(
                MenuConfig {
                    name: "Editor".into(),
                    alt_code: KeyCode::KeyS.into(),
                },
                |menu| {
                    menu.menu_item(MenuItemConfig {
                        name: "Layout".into(),
                        shortcut: vec![KeyCode::KeyL].into(),
                        alt_code: KeyCode::KeyL.into(),
                        ..default()
                    }).insert(Page::Layout);
                    menu.menu_item(MenuItemConfig {
                        name: "Interactions".into(),
                        shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyI].into(),
                        alt_code: KeyCode::KeyI.into(),
                        ..default()
                    }).insert(Page::Playground);

                    menu.separator();

                    let icons = ThemeData::default().icons;
                    menu.menu_item(MenuItemConfig {
                        name: "Exit".into(),
                        leading_icon: icons.exit_to_app,
                        ..default()
                    }).insert(ExitAppButton);
                }
            );
            bar.menu(
                MenuConfig {
                    name: "Use case".into(),
                    alt_code: KeyCode::KeyS.into(),
                },
                |menu| {
                    menu.menu_item(MenuItemConfig {
                        name: "Standard menu item".into(),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with leading icon".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });

                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with both icons".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE
                        ),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item".into(),
                        shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyT].into(),
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Already toggled item".into(),
                        initially_checked: true,
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.submenu(
                        SubmenuConfig {
                            name: "Submenu".into(),
                            ..default()
                        },
                        |submenu| {
                            submenu.menu_item(MenuItemConfig {
                                name: "Standard menu item".into(),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with leading icon".into(),
                                leading_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/details_menu.png".into(),
                                    Color::WHITE
                                ),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with trailing icon".into(),
                                trailing_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/tiles_menu.png".into(),
                                    Color::WHITE
                                ),
                                ..default()
                            });
                        }
                    );
                }
            );

            bar.menu(
                MenuConfig {
                    name: "Test case".into(),
                    alt_code: KeyCode::KeyS.into(),
                },
                |menu| {
                    menu.menu_item(MenuItemConfig {
                        name: "Standard menu item".into(),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with leading icon".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });
                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });

                    menu.menu_item(MenuItemConfig {
                        name: "Menu item with both icons".into(),
                        leading_icon: IconData::Image(
                            "embedded://sickle_ui/icons/details_menu.png".into(),
                            Color::WHITE
                        ),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item".into(),
                        shortcut: vec![KeyCode::ControlLeft, KeyCode::KeyT].into(),
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Already toggled item".into(),
                        initially_checked: true,
                        ..default()
                    });
                    menu.toggle_menu_item(ToggleMenuItemConfig {
                        name: "Toggle item with trailing icon".into(),
                        trailing_icon: IconData::Image(
                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                            Color::WHITE
                        ),
                        ..default()
                    });

                    menu.separator();

                    menu.submenu(
                        SubmenuConfig {
                            name: "Submenu".into(),
                            ..default()
                        },
                        |submenu| {
                            submenu.menu_item(MenuItemConfig {
                                name: "Standard menu item".into(),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with leading icon".into(),
                                leading_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/details_menu.png".into(),
                                    Color::WHITE
                                ),
                                ..default()
                            });
                            submenu.menu_item(MenuItemConfig {
                                name: "Menu item with trailing icon".into(),
                                trailing_icon: IconData::Image(
                                    "embedded://sickle_ui/icons/tiles_menu.png".into(),
                                    Color::WHITE
                                ),
                                ..default()
                            });

                            submenu.submenu(
                                SubmenuConfig {
                                    name: "Submenu with lead icon".into(),
                                    leading_icon: IconData::Image(
                                        "embedded://sickle_ui/icons/details_menu.png".into(),
                                        Color::WHITE
                                    ),
                                    ..default()
                                },
                                |submenu| {
                                    submenu.menu_item(MenuItemConfig {
                                        name: "Standard menu item".into(),
                                        ..default()
                                    });
                                    submenu.menu_item(MenuItemConfig {
                                        name: "Menu item with leading icon".into(),
                                        leading_icon: IconData::Image(
                                            "embedded://sickle_ui/icons/details_menu.png".into(),
                                            Color::WHITE
                                        ),
                                        ..default()
                                    });
                                    submenu.menu_item(MenuItemConfig {
                                        name: "Menu item with trailing icon".into(),
                                        trailing_icon: IconData::Image(
                                            "embedded://sickle_ui/icons/tiles_menu.png".into(),
                                            Color::WHITE
                                        ),
                                        ..default()
                                    });
                                }
                            );
                        }
                    );
                }
            );

            bar.separator();

            bar.extra_menu(|extra| {
                extra.radio_group(vec!["Light", "Dark"], 1, false).insert(ThemeSwitch);
                extra
                    .dropdown(vec!["Standard", "Medium Contrast", "High Contrast"], 0)
                    .insert(ThemeContrastSelect)
                    .style()
                    .width(Val::Px(150.0));
            });
        });

        column
            .row(|_| {})
            .insert((EditorContainer, UiContextRoot))
            .style()
            .height(Val::Percent(100.0))
            .background_color(Color::NONE);
    });
}
