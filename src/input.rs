// for now, this is the One Place to map inputs to something

// short term, observers and/or signals will be added to propagate inputs elsewhere

// for beta, we're gonna need full control scheme support (input remapping including assistive technologies)

use bevy::prelude::*;

#[cfg(feature = "bevy_dev_tools")]
use bevy_dev_tools::ui_debug_overlay::*;

use leafwing_input_manager::Actionlike;
use leafwing_input_manager::plugin::InputManagerPlugin;

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputConfig>().add_plugins(
            InputManagerPlugin::<InputAction>::default()
        );

        // add the standard debug overlay if dev tools are enabled
        #[cfg(feature = "bevy_dev_tools")]
        {
            app.add_plugins(DebugUiPlugin).add_systems(Update, toggle_overlay);
        }
    }
}

// This is the list of "things in the game I want to be able to do based on input"
#[derive(Actionlike, PartialEq, Eq, Hash, Clone, Copy, Debug, Reflect)]
pub enum InputAction {
    CameraRotateYIncrease,
    CameraRotateYDecrease,
    ToggleRemoteFpsCounter,
}

// input config resource
#[derive(Resource, Default)]
pub struct InputConfig {
    pub remote_fps: bool,
}

// from Bevy UI examples
#[cfg(feature = "bevy_dev_tools")]
// The system that will enable/disable the debug outlines around the nodes
fn toggle_overlay(input: Res<ButtonInput<KeyCode>>, mut options: ResMut<UiDebugOptions>) {
    info_once!("The debug outlines are enabled, press Space to turn them on/off");
    if input.just_pressed(KeyCode::Space) {
        // The toggle method will enable the debug_overlay if disabled and disable if enabled
        options.toggle();
    }
}
