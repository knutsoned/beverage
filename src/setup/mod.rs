use bevy::prelude::*;

use bevy_fluent::Localization;

use sickle_ui::{ prelude::*, ui_commands::UpdateStatesExt };

use crate::{ framework::*, layout::footer::UiUiFooterRootNodeExt };

pub mod menu;
use menu::build_menu;

pub fn on_load(
    asset_server: Res<AssetServer>,
    mut icon_cache: ResMut<IconCache>,
    l10n: Res<Localization>,
    mut commands: Commands
) {
    // Workaround for disappearing icons when they are despawned and spawned back in during the same frame
    // Should be fixed in Bevy > 0.13
    let icons_to_cache: Vec<&str> = vec![
        "embedded://sickle_ui/icons/checkmark.png",
        "embedded://sickle_ui/icons/chevron_down.png",
        "embedded://sickle_ui/icons/chevron_left.png",
        "embedded://sickle_ui/icons/chevron_right.png",
        "embedded://sickle_ui/icons/chevron_up.png",
        "embedded://sickle_ui/icons/close.png",
        "embedded://sickle_ui/icons/exit_white.png",
        "embedded://sickle_ui/icons/popout_white.png",
        "embedded://sickle_ui/icons/redo_white.png",
        "embedded://sickle_ui/icons/submenu_white.png"
    ];

    for icon in icons_to_cache.iter() {
        icon_cache.0.push(asset_server.load(*icon));
    }

    // The main camera which will render UI
    let main_camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    order: 1,
                    clear_color: Color::BLACK.into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(0.0, 30.0, 0.0)).looking_at(
                    Vec3::ZERO,
                    Vec3::Y
                ),
                ..Default::default()
            },
            UiCamera,
        ))
        .id();

    // Use the UI builder with plain bundles and direct setting of bundle props
    let mut root_entity = Entity::PLACEHOLDER;

    // top level container
    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            TargetCamera(main_camera),
        ),
        |container| {
            root_entity = container
                .spawn((
                    NodeBundle {
                        style: Style {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Row,
                            justify_content: JustifyContent::SpaceBetween,
                            ..default()
                        },
                        ..default()
                    },
                    UiMainRootNode,
                ))
                .id();

            container.ui_footer(
                |_builder| {
                    /*
                    builder
                        .label(LabelConfig {
                            label: "Footer".into(),
                            ..default()
                        })
                        .style()
                        .width(Val::Px(80.0));
                    */
                }
            );
        }
    );

    // Use the UI builder of the root entity with styling applied via commands
    commands.ui_builder(root_entity).column(|builder| {
        // add the menu bar
        build_menu(builder, &l10n);

        // set up the main editor container
        builder
            .row(|_| {})
            .insert((EditorContainer, UiContextRoot))
            .style()
            .height(Val::Percent(100.0))
            .background_color(Color::NONE);
    });

    commands.next_state(Page::SceneEditor);
}
