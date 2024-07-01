//! A Bevy app that you can connect to with the BRP and edit.

use bevy::{ prelude::*, remote::RemotePlugin };

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePlugin::default())
        .add_systems(Startup, (lights_camera, mesh))
        .add_systems(Update, update_camera)
        .run();
}

fn lights_camera(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });

    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>
) {
    // circular base
    commands.spawn(PbrBundle {
        mesh: meshes.add(Circle::new(4.0)),
        material: materials.add(Color::WHITE),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });

    // cube
    commands.spawn(PbrBundle {
        mesh: meshes.add(Cuboid::new(1.0, 1.0, 1.0)),
        material: materials.add(Color::srgb_u8(124, 144, 255)),
        transform: Transform::from_xyz(0.0, 0.5, 0.0),
        ..default()
    });
}

fn look_at_origin(transform: &mut Transform) -> Transform {
    transform.looking_at(Vec3::ZERO, Vec3::Y)
}

fn update_camera(mut transform: Query<&mut Transform, With<Camera>>) {
    if let Ok(transform) = transform.get_single_mut() {
        let transform = transform.into_inner();
        *transform = look_at_origin(transform);
    }
}
