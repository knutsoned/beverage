// Example themed widgets, generated with snipped

use bevy::prelude::*;

use sickle_ui::prelude::*;

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
