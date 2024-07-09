// Example themed widgets, generated with snipped

use bevy::{ color::palettes, prelude::* };

use bevy_fluent::Localization;
use sickle_ui::prelude::*;

use crate::{ framework::*, prelude::{ RemoteConnectionState, UiFooterContainer } };

pub struct UiFooterRootNodePlugin;

impl Plugin for UiFooterRootNodePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(ComponentThemePlugin::<UiFooterRootNode>::default());
    }
}

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct UiFooterRootNode;

impl DefaultTheme for UiFooterRootNode {
    fn default_theme() -> Option<Theme<UiFooterRootNode>> {
        UiFooterRootNode::theme().into()
    }
}

impl UiFooterRootNode {
    pub fn theme() -> Theme<UiFooterRootNode> {
        let base_theme = PseudoTheme::deferred(None, UiFooterRootNode::primary_style);
        Theme::new(vec![base_theme])
    }

    fn primary_style(style_builder: &mut StyleBuilder, theme_data: &ThemeData) {
        let theme_spacing = theme_data.spacing;
        let colors = theme_data.colors();

        style_builder
            .justify_content(JustifyContent::SpaceBetween)
            .width(Val::Percent(100.0))
            .height(Val::Px(theme_spacing.areas.medium))
            .border(UiRect::top(Val::Px(theme_spacing.borders.extra_small)))
            .border_color(colors.accent(Accent::Shadow))
            .background_color(colors.container(Container::SurfaceMid));
    }

    fn frame() -> impl Bundle {
        (Name::new("UiFooterRootNode"), NodeBundle::default())
    }
}

#[derive(Component, Clone, Debug, Default, Reflect, UiContext)]
#[reflect(Component)]
pub struct UiFooterElement;

impl UiFooterElement {
    fn frame() -> impl Bundle {
        (Name::new("UiFooterElement"), NodeBundle::default())
    }
}

pub trait UiUiFooterRootNodeExt {
    fn ui_footer(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>)
    ) -> UiBuilder<Entity>;
}

impl UiUiFooterRootNodeExt for UiBuilder<'_, Entity> {
    fn ui_footer(
        &mut self,
        spawn_children: impl FnOnce(&mut UiBuilder<Entity>)
    ) -> UiBuilder<Entity> {
        self.container((UiFooterRootNode::frame(), UiFooterRootNode), spawn_children)
    }
}

pub fn spawn_footer(
    footer_container: Query<Entity, With<UiFooterContainer>>,
    footer_root: Query<Entity, With<UiFooterRootNode>>,
    l10n: Res<Localization>,
    remote_state: Res<State<RemoteConnectionState>>,
    mut commands: Commands
) {
    // FIXME trying to get the connection status label to center in itself but justify end as a whole
    warn!("spawn_footer");
    if let Ok(footer_container) = footer_container.get_single() {
        if let Ok(footer_root) = footer_root.get_single() {
            // despawn the footer that floats on top (at the bottom?)
            commands.entity(footer_root).despawn_recursive();
        }

        commands.ui_builder(footer_container).ui_footer(|builder| {
            builder
                .label(LabelConfig {
                    label: l10n.lbl("Status"),
                    ..default()
                })
                .style()
                .margin(UiRect::all(Val::Px(10.0)))
                .width(Val::Px(80.0));

            builder.spawn((UiFooterElement::frame(), UiFooterElement));

            builder
                .container((UiFooterElement::frame(), UiFooterElement), |container| {
                    container
                        .label(LabelConfig {
                            label: l10n.lbl(match remote_state.get() {
                                RemoteConnectionState::Disconnected => "Disconnected",
                                RemoteConnectionState::Connecting => "Connecting",
                                RemoteConnectionState::Checking => "Checking",
                                RemoteConnectionState::Connected => "Connected",
                            }),
                            ..default()
                        })
                        .style()
                        .font_color(Color::Srgba(palettes::css::BLUE_VIOLET))
                        .align_self(AlignSelf::Center)
                        .margin(UiRect::all(Val::Px(10.0)))
                        .width(Val::Px(180.0));
                })
                .style()
                .justify_content(JustifyContent::Center)
                .width(Val::Percent(100.0))
                .background_color(match remote_state.get() {
                    RemoteConnectionState::Disconnected => Color::Srgba(palettes::css::LIGHT_CORAL),
                    RemoteConnectionState::Connecting =>
                        Color::Srgba(palettes::css::PALE_GOLDENROD),
                    RemoteConnectionState::Checking => Color::Srgba(palettes::css::PALE_GOLDENROD),
                    RemoteConnectionState::Connected => Color::Srgba(palettes::css::CHARTREUSE),
                });
        });
    }
}
