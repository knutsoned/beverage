use std::borrow::Cow;

use bevy::{ asset::LoadedFolder, prelude::*, tasks::Task };

// eventually this will be whatever we use for permanent IDs that can be passed around a network

// for now it's just an arbitrary string
#[derive(Debug, PartialEq, Eq, Hash, Clone, Default, Reflect)]
pub struct EditorId(pub String);

// this allows an EditorId to be used as a bevy_core::name::Name
impl From<EditorId> for Cow<'static, str> {
    fn from(val: EditorId) -> Self {
        val.0.into()
    }
}

pub trait Translator {
    fn lbl(&self, str: &str) -> String;
    fn t(&self, string: String) -> String;
}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct UiFooterContainer;

#[derive(Component)]
pub struct UiMainRootNode;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UiStartupSet;

#[derive(States, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
pub enum EditorState {
    #[default]
    Loading,
    SwitchLocale,
    // this state is a "virtual" state not used directly with schedules

    // this state exists so there is a non-Running state for use by OnExit detection

    // e.g. when changing locales:
    // 1) we enter the SwitchLocale state
    // 2) the OnEnter(SwitchLocale) system set for that state switches to the Building state
    // 3) the actual system set that needs to run next uses OnExit<SwitchLocale> for scheduling
    // 4) that system switches to the Running state after completing its work
    Building,
    Running,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, States, Hash)]
#[reflect(Component)]
pub enum Page {
    #[default]
    None,
    CameraControl,
    Playground,
    QuillDemo,
    SceneEditor,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ExitAppButton;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct EditorContainer;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct TreeViewPanel;

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct CurrentPage(Page);

// l10n stuff
#[derive(Resource)]
pub struct LocaleFolder(pub Handle<LoadedFolder>);

#[derive(Component)]
pub struct LocaleRoot;

#[derive(Component, Debug)]
pub struct LocaleSelect;

// marker for a local entity whose transform may control a remote camera
#[derive(Component)]
pub struct RemoteCamera;

// marker for an FPS counter on a remote server
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct RemoteFpsCounter;

// marker to remove an FPS counter on a remote server
#[derive(Component, Reflect)]
#[reflect(Component)]
pub struct DespawnRemoteFpsCounter;

// marker for an entity with updates that can't be sent yet
// (probably because the previous update is still running)
#[derive(Component)]
pub struct RemotePending;

#[derive(Component, Debug)]
pub struct RemoteRequest {
    pub task: Task<()>,
}

// query args to help remotely query or update an entity's transform
pub type RemoteTransformArgs<'a> = (
    Entity,
    &'a mut Transform,
    Option<&'a mut RemoteRequest>,
    Option<&'a RemotePending>,
);

// need states to prevent updates from sending before the remote camera entity ID is known
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
pub enum RemoteConnectionState {
    #[default]
    Disconnected,
    // getting the remote camera entity...
    Connecting,
    // checking the response, do not pass go, do not collect $200
    Checking,
    // not a persistent connection, but "connected" as in, able to map to the remote camera
    Connected,
}
