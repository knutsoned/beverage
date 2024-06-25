//! A Bevy app that you can connect to with the BRP and edit.

use bevy::{ prelude::*, remote::RemotePlugin };

mod scene;
use scene::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePlugin::default())
        .add_systems(Startup, setup_scene)
        .add_systems(Update, update_camera)
        .run();
}

fn update_camera(mut camera: Query<(Entity, &mut Transform), With<Camera>>) {
    if let Ok(camera) = camera.get_single_mut() {
        //info!("Entity {} transform.translation: {}", camera.0, camera.1.translation);
        point_at_origin(*camera.1);
    }
}
