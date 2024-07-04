//! A Bevy app that you can connect to with the BRP and edit.

use bevy::{ prelude::*, remote::RemotePlugin };

use sickle_ui::{ prelude::*, ui_commands::UpdateStatesExt };

use sickle_example::prelude::*;

use beverage::framework::*;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(RemotePlugin::default())
        .init_state::<FpsVisibility>()
        .add_systems(Startup, (lights_camera, mesh))
        .add_systems(Update, (update_camera, update_fps_visibility))
        .add_systems(Update, update_fps.run_if(in_state(FpsVisibility::Visible)))
        .run();
}

#[derive(States, Clone, Copy, Debug, Default, Eq, Hash, PartialEq)]
enum FpsVisibility {
    Hidden,
    #[default]
    Visible,
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

    // set up UI camera
    let main_ui_camera = commands
        .spawn((
            Camera3dBundle {
                camera: Camera {
                    clear_color: Color::BLACK.into(),
                    ..default()
                },
                transform: Transform::from_translation(Vec3::new(-2.5, 4.5, 9.0)).looking_at(
                    Vec3::ZERO,
                    Vec3::Y
                ),
                ..default()
            },
            UiCamera,
        ))
        .id();

    // TODO use 3D cam as viewport cam inside UI
    commands.ui_builder(UiRoot).container(
        (
            NodeBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::SpaceBetween,
                    ..default()
                },
                ..default()
            },
            TargetCamera(main_ui_camera),
        ),
        |_| {}
    );
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

    // uncomment to spawn an FpsCounter by default (simulate the client pressing the F key)
    //commands.spawn(RemoteFpsCounter);
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

fn update_fps_visibility(
    show_marker: Query<Entity, With<RemoteFpsCounter>>,
    hide_marker: Query<Entity, Added<DespawnRemoteFpsCounter>>,
    widget: Query<Entity, With<FpsWidget>>,
    mut commands: Commands
) {
    let hide_marker = hide_marker.get_single();
    let show_marker = show_marker.get_single();
    let widget = widget.get_single();

    // if both the show and hide marker are present, the hide marker wins
    if hide_marker.is_ok() {
        // despawn the FpsCounter and any RemoteFpsCounter and DespawnRemoteFpsCounter markers
        if let Ok(widget) = widget {
            error!("despawn FpsWdiget");
            commands.entity(widget).despawn_recursive();
        }
        if let Ok(show_marker) = show_marker {
            error!("despawn RemoteFpsCounter");
            commands.entity(show_marker).despawn();
        }
        commands.entity(hide_marker.unwrap()).despawn();
        error!("despawn the despawner");
        commands.next_state(FpsVisibility::Hidden);
        error!("found DespawnRemoteFpsCounter (hiding FPS)");
    } else if show_marker.is_ok() && widget.is_err() {
        // otherwise, if the client spawned a RemoteFpsCounter and there is no existing widget,
        // then spawn a new FpsWidget
        commands.ui_builder(UiRoot).fps();
        commands.next_state(FpsVisibility::Visible);
        warn!("spawning FPS")
    } else if show_marker.is_err() && widget.is_ok() {
        // if there is no RemoteFpsCounter, then despawn the FpsCounter
        commands.entity(widget.unwrap()).despawn_recursive();
        commands.next_state(FpsVisibility::Hidden);
        warn!("missing RemoteFpsCounter (hiding FPS)");
    }
}
