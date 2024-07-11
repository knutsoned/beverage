// cameras and containers.

use bevy::prelude::*;

use bevy_fluent::Localization;

use sickle_ui::{ prelude::*, ui_commands::UpdateStatesExt };

use crate::{ framework::*, layout::*, locale::* };

// TODO modularize the menu so it's a regular system and not a fn that has to be called
pub mod menu;
use menu::build_menu;

pub fn on_load(l10n: Res<Localization>, mut commands: Commands) {
    warn!("on_load");
    // The main camera which will render UI
    let main_ui_camera = commands
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

    // top level container
    let top_level = commands
        .ui_builder(UiRoot)
        .container(
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
                LocaleRoot,
                TargetCamera(main_ui_camera),
            ),
            |_| {}
        )
        .id();

    build(&mut commands, &l10n, &top_level, 0);
}

pub fn on_rebuild(
    footer_root: Query<Entity, With<UiFooterContainer>>,
    ui_main_root: Query<Entity, With<UiMainRootNode>>,
    locale_root: Query<Entity, With<LocaleRoot>>,
    locale_select: Query<&Dropdown, With<LocaleSelect>>,
    l10n: Res<Localization>,
    mut commands: Commands
) {
    warn!("rebuild");

    // trigger update of the UI text
    if let Ok(locale_root) = locale_root.get_single() {
        if let Ok(ui_main_root) = ui_main_root.get_single() {
            if let Ok(footer_root) = footer_root.get_single() {
                // despawn the footer that floats on top (at the bottom?)
                commands.entity(footer_root).despawn_recursive();
            }

            // despawn everything in the top level container
            commands.entity(ui_main_root).despawn_recursive();

            if let Ok(locale_select) = locale_select.get_single() {
                build(
                    &mut commands,
                    &l10n,
                    &locale_root,
                    locale_select.value().expect("No selected locale in dropdown")
                );
            } else {
                error!("No LocaleSelect");
            }
        } else {
            error!("No UiMainRootNode");
        }
    } else {
        error!("No LocaleRoot");
    }
}

fn build(commands: &mut Commands, l10n: &Res<Localization>, context: &Entity, locale_index: usize) {
    warn!("(build)");
    let context = *context;

    let root_entity = commands
        .ui_builder(context)
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

    commands
        .ui_builder(context)
        .container((UiFooterContainer, NodeBundle::default()), |_| {})
        .style()
        .width(Val::Percent(100.0));

    // Use the UI builder of the root entity with styling applied via commands
    commands.ui_builder(root_entity).column(|builder| {
        // add the menu bar
        build_menu(builder, l10n, locale_index);

        // set up the main editor container
        builder
            .row(|_| {})
            .insert((EditorContainer, UiContextRoot))
            .style()
            .height(Val::Percent(100.0))
            .background_color(Color::NONE);
    });

    info!("switching EditorState to Running");
    commands.next_state(EditorState::Running);

    // this is where to set the default start page for the editor content area (also place marker in layout::editor::layout)
    commands.next_state(Page::QuillDemo);
}
