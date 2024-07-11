use bevy::{ prelude::*, utils::HashMap };

use crate::{ activity::ActivityService, framework::* };

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

/// Core services.
///
/// See activity.rs for ActivityService.
///
/// The asset service provides integration between internal metadata used by the editor and the
/// regular Bevy asset infrastructure.
#[derive(Resource, Debug)]
pub struct AssetService {
    atlas_map: HashMap<EditorId, EditorAtlas>,
}

impl AssetService {
    pub fn get_atlas(&mut self, id: EditorId) -> EditorAtlas {
        if let Some(atlas) = self.atlas_map.get(&id) {
            return atlas.clone();
        }

        let atlas = EditorAtlas { id: id.clone() };
        self.atlas_map.insert(id.clone(), atlas.clone());
        atlas.clone()
    }
}

/// The history service manages action histories for activities.
#[derive(Resource, Debug)]
pub struct HistoryService {}

/// The input service allows plugins to inspect the control layout and provide mappings for when
/// they have focus.
#[derive(Resource, Debug)]
pub struct InputService {}

/// The layout service manages the overall editor layout and maps logical screen areas to Bevy UI
/// containers.
#[derive(Resource, Debug)]
pub struct LayoutService {}

/// The widget service provices all registered widgets and allows plugins to register their own.
#[derive(Resource, Debug)]
pub struct WidgetService {}

/// Optional services.
///
/// The construct service manages templates and instances for prefabs, blueprints, etc.
#[derive(Resource, Debug)]
pub struct ConstructService {}

/// The locale service provides localization for the core UI and allows plugins to register their
/// own string template assets.
#[derive(Resource, Debug)]
pub struct LocaleService {}

/// The remote service provides connectivity and manages syncing state with a remote server.
/// For now the remote server is the in-game portion of the editor in a separate window.
#[derive(Resource, Debug)]
pub struct RemoteService {}

/// The router service provides navigation and data transfer between plugins. It maps strings
/// to logical endpoints that could be navigation (like web pages) or update endpoints (like web
/// API endpoints)
#[derive(Resource, Debug)]
pub struct RouterService {}

/// The theme service provides user-selectable color palettes and lets plugins define their own.
#[derive(Resource, Debug)]
pub struct ThemeService {}
