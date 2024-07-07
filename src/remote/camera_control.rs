use bevy::{ prelude::*, tasks::{ block_on, poll_once } };

use leafwing_input_manager::action_state::ActionState;

use sickle_ui::ui_commands::UpdateStatesExt;

use crate::{
    input::{ InputAction, InputConfig },
    prelude::*,
    remote::brp_client::BrpClient,
    widget::camera_control::CameraControl,
};

pub struct CameraControlRemotePlugin;

// TODO create in_running_state which also checks for EditorState::Running
impl Plugin for CameraControlRemotePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<BrpClient>()
            .init_state::<RemoteConnectionState>()
            .add_systems(Update, init_connect.run_if(in_state(RemoteConnectionState::Disconnected)))
            .add_systems(
                Update,
                connect_to_camera.run_if(in_state(RemoteConnectionState::Connecting))
            )
            .add_systems(Update, poll_responses.run_if(in_state(RemoteConnectionState::Checking)))
            .add_systems(
                Update,
                (check_toggle_fps, check_toggle_fps_response, sync_camera).run_if(
                    in_state(RemoteConnectionState::Connected)
                )
            );
    }
}

type RemoteActionQuery<'a> = (Entity, &'a ActionState<InputAction>, Option<&'a RemoteRequest>);

fn check_toggle_fps(
    q_action: Query<RemoteActionQuery, With<CameraControl>>,
    type_registry: Res<AppTypeRegistry>,
    mut input: ResMut<InputConfig>,
    mut brp: ResMut<BrpClient>,
    mut commands: Commands
) {
    for (entity, action_state, request) in &q_action {
        // if not already sending a toggle and the F key was just pressed...
        if request.is_none() && action_state.just_pressed(&InputAction::ToggleRemoteFpsCounter) {
            // toggle and send over the wire
            input.remote_fps = !input.remote_fps;
            if
                let Err(error) = brp.spawn_fps_marker(
                    &entity,
                    input.remote_fps,
                    &type_registry,
                    &mut commands
                )
            {
                error!("BRP error toggling FPS widget: {}", error);
            }
        }
    }
}

fn check_toggle_fps_response(
    mut q_action: Query<(Entity, &mut RemoteRequest), With<CameraControl>>,
    mut commands: Commands
) {
    for (entity, mut request) in q_action.iter_mut() {
        if block_on(poll_once(&mut request.task)).is_some() {
            commands.entity(entity).remove::<RemoteRequest>();
        }
    }
}

// step 1: wait for a key press
fn init_connect(
    q_action: Query<&ActionState<InputAction>, (With<CameraControl>, Without<RemoteRequest>)>,
    mut commands: Commands
) {
    for action_state in &q_action {
        if
            action_state.pressed(&InputAction::CameraRotateYDecrease) ||
            action_state.pressed(&InputAction::CameraRotateYIncrease) ||
            action_state.just_pressed(&InputAction::ToggleRemoteFpsCounter)
        {
            // try to sync client camera to server
            commands.next_state(RemoteConnectionState::Connecting);
        }
    }
}

// step 2: run a query to get the entity ID of the remote camera
fn connect_to_camera(
    mut camera: Query<RemoteTransformArgs, With<CameraControl>>,
    mut brp: ResMut<BrpClient>,
    mut commands: Commands
) {
    if let Ok(camera) = camera.get_single_mut() {
        // spawn a task to connect to the remote server
        match brp.fetch_remote_camera(camera.0, &mut commands) {
            Ok(_) => {
                trace!("spawning fetch_remote_camera task");

                // change the RemoteConnectionState to Checking
                commands.next_state(RemoteConnectionState::Checking);
            }
            Err(error) => { error!("Could not spawn task to get remote camera: {}", error) }
        }
    }
}

// step 3: see if the Camera entity has returned yet
fn poll_responses(
    mut camera: Query<RemoteTransformArgs, With<CameraControl>>,
    brp: Res<BrpClient>,
    mut commands: Commands
) {
    match camera.get_single_mut() {
        Ok(camera) => {
            // check to see if running task has completed
            if let Some(mut request) = camera.2 {
                if block_on(poll_once(&mut request.task)).is_some() {
                    let entity = camera.0;
                    commands.entity(entity).remove::<RemoteRequest>();

                    // check to see if we have an entity
                    let entity = match brp.remote_entity_dungeon.lock() {
                        Ok(mutex) => mutex.as_ref().is_some(),
                        Err(_) => false,
                    };

                    // change the RemoteConnectionState to Connected
                    info!("connected to BRP server");
                    if entity {
                        info!("...and found a camera!");
                        commands.next_state(RemoteConnectionState::Connected);
                    } else {
                        info!("[...]");
                        commands.next_state(RemoteConnectionState::Disconnected);
                    }
                }
            }
        }
        Err(error) => error!("Error loading camera: {}", error),
    }
}

// step 4: propagate local camera to remote
fn sync_camera(
    mut camera: Query<RemoteTransformArgs, With<RemoteCamera>>,
    mut brp: ResMut<BrpClient>,
    mut commands: Commands
) {
    match camera.get_single_mut() {
        Ok(camera) => {
            // send update or mark pending
            let entity = camera.0;
            let transform = camera.1;
            if transform.is_changed() {
                let mut running = camera.2.is_some();
                let pending = camera.3.is_some();
                let mut result: anyhow::Result<()> = Ok(());

                // check to see if running task has completed
                if let Some(mut request) = camera.2 {
                    if block_on(poll_once(&mut request.task)).is_some() {
                        commands.entity(entity).remove::<RemoteRequest>();
                        running = false;
                    }
                }

                // then we send the serialized Transform to the server
                // -if a request is already running, mark with RemotePending so we get to it later
                // -if no request is running, send one if either we moved or already pending

                // see if there is a running request
                // if so, then mark with RemotePending if not already
                if running && !pending {
                    // (if the request just finished it will still be marked running, which is fine)
                    // (next tick it will have no RunningRequest but will be marked RemotePending)
                    commands.entity(entity).insert(RemotePending);
                } else if !running && pending {
                    // if no running request and an update is pending, kick off a new request
                    result = brp.post_transform(entity, *transform, &mut commands);

                    // remove RemotePending
                    commands.entity(entity).remove::<RemotePending>();
                } else if !running && !pending {
                    // if no running request, kick off a new request
                    result = brp.post_transform(entity, *transform, &mut commands);

                    // don't mark with Pending until there is data to send
                }

                // handle error caused while spawning request task
                if let Err(error) = result {
                    error!("BRP error: {}", error);
                }
            }
        }
        Err(_error) => {
            //error!("camera error: {}", error)
        }
    }
}
