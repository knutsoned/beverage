// The mothership.

use bevy::{ log::LogPlugin, prelude::* }; //, winit::WinitSettings };

//use winit::window::Icon;

use beverage::EditorPlugin;

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
                    title: "Beverage - Also Available As A T-Shirt".into(),
                    resolution: (1280.0, 720.0).into(),
                    ..default()
                }),
                ..default()
            }).set(LogPlugin {
                filter: "info,wgpu_core=info,wgpu_hal=info,beverage=debug".into(),
                level: bevy::log::Level::DEBUG,
                custom_layer: |_| None,
            }),
            // the EditorPlugin provides the default sickle_ui, bevy_fluent, leafwing-input-manager, and BRP functionality

            // it is designed to respond to marker components in order to know how to configure itself (see documentation)

            // TODO implement this fully and write the documentation
            EditorPlugin,
        ))

        .run();
}

// from https://bevy-cheatbook.github.io/window/icon.html
/*
fn set_window_icon(
    // we have to use `NonSend` here
    windows: NonSend<WinitWindows>
) {
    // here we use the `image` crate to load our icon data from a png file
    // this is not a very bevy-native solution, but it will do
    let (icon_rgba, icon_width, icon_height) = {
        // FIXME this may not work, especially when packaged for release
        //let path = std::env::current_dir().unwrap().join("assets/textures/ic_launcher.png");
        //warn!("PATH: {}", path.display());
        let path = "assets/textures/ic_launcher.png";
        let image = image::open(path).expect("Failed to open icon path").into_rgba8();
        let (width, height) = image.dimensions();
        let rgba = image.into_raw();
        (rgba, width, height)
    };
    let icon = Icon::from_rgba(icon_rgba, icon_width, icon_height).unwrap();

    // do it for all windows
    for window in windows.windows.values() {
        window.set_window_icon(Some(icon.clone()));
    }
}
*/
