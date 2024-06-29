use bevy::prelude::*;

use bevy_fluent::Localization;

use sickle_ui::prelude::*;

use crate::framework::*;
use super::{ ExitAppButton, LocaleSelect, Page, ThemeContrastSelect, ThemeSwitch };

pub fn build_menu(column: &mut UiBuilder<'_, Entity>, l10n: &Res<Localization>) {
    column.style().width(Val::Percent(100.0)).background_color(Color::srgb(0.15, 0.155, 0.16));

    column.menu_bar(|bar| {
        bar.menu(
            MenuConfig {
                name: l10n.lbl("Editor"),
                alt_code: KeyCode::KeyS.into(),
            },
            |menu| {
                menu.menu_item(MenuItemConfig {
                    name: l10n.lbl("Scene"),
                    shortcut: vec![KeyCode::KeyL].into(),
                    alt_code: KeyCode::KeyL.into(),
                    ..default()
                }).insert(Page::SceneEditor);

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
                        l10n.lbl("Medium Contrast"),
                        l10n.lbl("High Contrast")
                    ],
                    0
                )
                .insert(ThemeContrastSelect)
                .style()
                .width(Val::Px(150.0));
            /*
        });

        bar.extra_menu(|extra| {
            */
            extra
                .label(LabelConfig {
                    label: l10n.lbl("Language"),
                    ..default()
                })
                .style()
                .width(Val::Px(70.0))
                .margin(UiRect::new(Val::Px(50.0), Val::Px(0.0), Val::Px(0.0), Val::Px(0.0)));

            // do NOT translate this

            // the user should be able to find the native name of their language
            extra
                .dropdown(vec!["English", "Français"], 0)
                .insert(LocaleSelect)
                .style()
                .width(Val::Px(150.0));
        });
    });
}
