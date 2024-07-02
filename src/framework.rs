use bevy::{ asset::LoadedFolder, prelude::* };
use leafwing_input_manager::Actionlike;

pub trait Translator {
    fn lbl(&self, str: &str) -> String;
    fn t(&self, string: String) -> String;
}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct UiMainRootNode;

#[derive(SystemSet, Clone, Hash, Debug, Eq, PartialEq)]
pub struct UiStartupSet;

#[derive(Clone, Copy, Debug, Default, Eq, Hash, PartialEq, States)]
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

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum InputAction {
    CameraRotateYIncrease,
    CameraRotateYDecrease,
}

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, States, Hash)]
#[reflect(Component)]
pub enum Page {
    #[default]
    None,
    CameraControl,
    SceneEditor,
    Playground,
}

#[derive(Component, Clone, Copy, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct ExitAppButton;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct EditorContainer;

#[derive(Component, Debug, Default, Reflect)]
#[reflect(Component)]
pub struct HierarchyPanel;

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

// theme handling widgets
#[derive(Component, Debug)]
pub struct ThemeSwitch;

#[derive(Component, Debug)]
pub struct ThemeContrastSelect;
