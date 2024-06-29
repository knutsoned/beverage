use bevy::prelude::*;

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

#[derive(Component, Clone, Copy, Debug, Default, PartialEq, Eq, Reflect, States, Hash)]
#[reflect(Component)]
pub enum Page {
    #[default]
    None,
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

#[derive(Resource, Debug, Default, Reflect)]
#[reflect(Resource)]
pub struct IconCache(pub Vec<Handle<Image>>);

#[derive(Component, Debug)]
pub struct LocaleSelect;

#[derive(Component, Debug)]
pub struct ThemeSwitch;

#[derive(Component, Debug)]
pub struct ThemeContrastSelect;
