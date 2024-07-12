// define each standard editor widget as a plugin in the widget folder and add the mod here.

use bevy::prelude::*;

/// The widget service provices all registered widgets and allows plugins to register their own.
#[derive(Resource, Default, Debug)]
pub struct WidgetService {}
