use bevy::prelude::*;

use beverage::{
    prelude::camera_control::widget::{ CameraControl, CameraControlPlugin },
    remote::camera_control::CameraControlRemotePlugin,
    EditorPlugin,
};

fn main() {
    App::new()
        // to make your own editor app, just cut and paste this section
        // into your own app and optionally configure the window title and log filters

        // note that there are in game editor widgets that are designed to work with BRP

        // the current demo can be explored by: cargo run --example server

        // this spawns a BRP server window with a demo scene

        // you may then connect to that using the standalone editor (i.e. this app)
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Quill Demo".into(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            }),
            // the EditorPlugin provides the default sickle_ui, bevy_fluent, leafwing-input-manager, and BRP functionality

            // it is designed to respond to marker components in order to know how to configure itself (see documentation)

            // TODO implement this fully and write the documentation
            EditorPlugin,
        ))
        // sickle widget plugin for the remote camera demo
        //.add_plugins(CameraControlPlugin)
        .add_plugins(CameraControlPlugin)
        // BRP plugin to sync server camera with local viewport (needs type of camera control component to be able to find camera to sync)
        .add_plugins(CameraControlRemotePlugin::<CameraControl>::default())

        .run();
}
