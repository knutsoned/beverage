use bevy::prelude::*;

use bevy_fluent::Localization;

use sickle_ui::prelude::*;

use crate::{ framework::*, ThemeContrastSelect, ThemeSwitch };
use super::{ ExitAppButton, LocaleSelect, Page };

pub fn build_menu(
    column: &mut UiBuilder<'_, Entity>,
    l10n: &Res<Localization>,
    locale_index: usize
) {
    warn!("build_menu");
    column.style().width(Val::Percent(100.0)).background_color(Color::srgb(0.15, 0.155, 0.16));

    column.menu_bar(|bar| {
        bar.menu(
            MenuConfig {
                name: l10n.lbl("Editor"),
                alt_code: KeyCode::KeyE.into(),
            },
            |menu| {
                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("CameraControl"),
                    shortcut: vec![KeyCode::KeyC].into(),
                    alt_code: KeyCode::KeyC.into(),
                    ..default()
                }).insert(Page::CameraControl);

                menu.separator();

                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("QuillDemo"),
                    shortcut: vec![KeyCode::KeyQ].into(),
                    alt_code: KeyCode::KeyQ.into(),
                    ..default()
                }).insert(Page::QuillDemo);

                menu.separator();

                let icons = ThemeData::default().icons;
                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("Exit"),
                    leading_icon: icons.exit_to_app,
                    ..default()
                }).insert(ExitAppButton);
            }
        );

        bar.separator();

        bar.menu(
            MenuConfig {
                name: l10n.lbl("File"),
                alt_code: KeyCode::KeyL.into(),
            },
            |menu| {
                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("Open"),
                    shortcut: vec![KeyCode::KeyO].into(),
                    alt_code: KeyCode::KeyO.into(),
                    ..default()
                }).insert(Page::CameraControl);

                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("Save"),
                    shortcut: vec![KeyCode::KeyS].into(),
                    alt_code: KeyCode::KeyS.into(),
                    ..default()
                }).insert(ExitAppButton);

                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("Close"),
                    shortcut: vec![KeyCode::KeyC].into(),
                    alt_code: KeyCode::KeyC.into(),
                    ..default()
                }).insert(ExitAppButton);
            }
        );

        bar.separator();

        bar.extra_menu(|extra| {
            extra
                .label(LabelConfig {
                    label: l10n.lbl("Theme"),
                    ..default()
                })
                .style()
                .width(Val::Px(50.0));
            extra
                .radio_group(vec![l10n.lbl("Light"), l10n.lbl("Dark")], 1, false)
                .insert(ThemeSwitch);
            extra
                .dropdown(
                    vec![
                        l10n.lbl("Standard"),
                        l10n.lbl("MediumContrast"),
                        l10n.lbl("HighContrast")
                    ],
                    0
                )
                .insert(ThemeContrastSelect)
                .style()
                .width(Val::Px(150.0));

            extra
                .label(LabelConfig {
                    label: l10n.lbl("Language"),
                    ..default()
                })
                .style()
                .width(Val::Px(70.0))
                .margin(UiRect::new(Val::Px(50.0), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0)));

            // UX: do NOT translate this -- the user should always be able to find the native name of their language
            extra
                .dropdown(vec!["English", "Français"], locale_index)
                .insert(LocaleSelect)
                .style()
                .width(Val::Px(150.0));
        });
    });
}
