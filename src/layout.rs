// this is where the "main" layouts go.

use bevy::prelude::*;

pub mod editor;
pub mod footer;

/// The layout service manages the overall editor layout and maps logical screen areas to Bevy UI
/// containers.
#[derive(Resource, Default, Debug)]
pub struct LayoutService {}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct UiFooterContainer;

#[derive(Component)]
pub struct UiMainRootNode;
