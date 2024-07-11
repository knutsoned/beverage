// this is where the "main" layouts go.

use bevy::prelude::*;

pub mod editor;
pub mod footer;

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct UiFooterContainer;

#[derive(Component)]
pub struct UiMainRootNode;
