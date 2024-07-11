use core::fmt;

use bevy::{ prelude::*, utils::HashMap };

use crate::framework::*;

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
/// The activity service provides a persistent context for a particular editor function like
/// "mmanage an asset" or "preview a scene." It can track multiple instances of the same
/// activity and manages a stack of all activities.
#[derive(Resource, Debug)]
pub struct ActivityService {
    pub current_activity: Box<dyn Activity>,
}

impl ActivityService {
    pub fn start(&mut self) -> EditorId {
        self.current_activity.start()
    }

    pub fn stop(&mut self) {
        self.current_activity.stop();
    }

    pub fn restart(&mut self, mut activity: impl Activity) -> EditorId {
        activity.stop();
        activity.start()
    }
}

impl Default for ActivityService {
    fn default() -> Self {
        Self { current_activity: Box::new(DefaultActivity) }
    }
}

pub trait Activity: Reflect + fmt::Debug {
    fn start(&mut self) -> EditorId;
    fn stop(&mut self);
}

#[derive(Reflect, Debug, Default)]
pub struct DefaultActivity;

impl Activity for DefaultActivity {
    fn start(&mut self) -> EditorId {
        EditorId::default()
    }

    fn stop(&mut self) {}
}

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

#[derive(Clone, Debug)]
pub struct EditorAtlas {
    id: EditorId,
}

impl EditorAtlas {
    pub fn id(&self) -> EditorId {
        let id = self.id.clone_value();
        <EditorId as FromReflect>::from_reflect(&*id).unwrap()
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
