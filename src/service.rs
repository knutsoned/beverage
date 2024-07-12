use bevy::prelude::*;

use crate::{
    activity::ActivityService,
    asset::AssetService,
    construct::ConstructService,
    history::HistoryService,
    input::InputService,
    layout::LayoutService,
    locale::LocaleService,
    router::RouterService,
    widget::WidgetService,
    remote::RemoteService,
    theme::ThemeService,
};

/// The service resource provides a set of objects representing the internal capabilities of the editor framework.
///
/// The following services are considered part of the core and are mandatory:
/// - activity
/// - asset
/// - history
/// - input
/// - layout
/// - widget
///
/// The following optional servies may also be present:
/// - construct
/// - locale
/// - remote
/// - router
/// - theme
///
/// If you're making your own editor, you'd probably make your own resource that only has the
/// services you really use.
#[derive(Resource, Debug)]
pub struct EditorService {
    pub activity: ActivityService,
    pub asset: AssetService,
    pub history: HistoryService,
    pub input: InputService,
    pub layout: LayoutService,
    pub widget: WidgetService,
    pub construct: Option<ConstructService>,
    pub locale: Option<LocaleService>,
    pub remote: Option<RemoteService>,
    pub router: Option<RouterService>,
    pub theme: Option<ThemeService>,
}

impl FromWorld for EditorService {
    fn from_world(_world: &mut World) -> Self {
        // the default resource provides defaults for all required services and None for options
        EditorService {
            activity: ActivityService::default(),
            asset: AssetService::default(),
            history: HistoryService::default(),
            input: InputService::default(),
            layout: LayoutService::default(),
            widget: WidgetService::default(),
            construct: None,
            locale: None,
            remote: None,
            router: None,
            theme: None,
        }
    }
}
