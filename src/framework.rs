use std::borrow::Cow;

use bevy::prelude::*;

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
    About,
    CameraControl,
    Help,
    Playground,
    QuillDemo,
    SceneEditor,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ExitAppButton;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct NewProjectButton;

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct OpenFileButton;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct EditorContainer;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct TreeViewPanel;

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct CurrentPage(Page);

#[derive(Clone, Debug)]
pub struct EditorAtlas {
    pub id: EditorId,
}
