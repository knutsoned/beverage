//! A Bevy app that sends camera position commands to a BRP server.

/*
 0. start in Disconnected mode
 1. see if there are any remote entities with a Transform (Uninitialized mode)
 2. if no entities on the remote server, send all scene data to server (Initializing mode)
 3. check for a camera on the remote server, if so goto 4 (Connected mode), if not repeat (Connecting mode)
 4. in Connected mode, whenever the local camera changes, send the new transform to the server
 */

use std::sync::{ Arc, Mutex };

use bevy::{
    ecs::event::EventWriter,
    prelude::*,
    remote::{
        builtin_verbs::{ BrpInsertRequest, BrpQuery, BrpQueryRequest, BrpQueryResponse },
        BrpRequest,
        DEFAULT_PORT,
    },
    tasks::{ block_on, poll_once, IoTaskPool, Task },
    utils::HashMap,
};

use anyhow::anyhow;
use argh::FromArgs;
use ehttp::Request;
use serde_json::Value;

use sickle_ui::ui_commands::UpdateStatesExt;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BrpResource>()
        .init_state::<CameraState>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, on_press.run_if(in_state(CameraState::Disconnected)))
        .add_systems(
            Update,
            (connect_to_camera, poll_responses).run_if(in_state(CameraState::Connecting))
        )
        .add_systems(Update, sync_camera.run_if(in_state(CameraState::Connected)))
        .run();
}

/// TODO
#[derive(FromArgs)]
struct Args {
    /// the host to connect to
    #[argh(option, default = "\"127.0.0.1\".to_owned()")]
    host: String,
    /// the port to connect to
    #[argh(option, default = "DEFAULT_PORT")]
    port: u16,
}

// ehttp builder
struct EhttpBuilder {}
trait RemoteRequestBuilder: Send + Sync + 'static {
    // TODO accept callback closure instead of returning ehttp::Request
    fn post(&self, url: String, body: String) -> ehttp::Request;
}
impl RemoteRequestBuilder for EhttpBuilder {
    fn post(&self, url: String, body: String) -> ehttp::Request {
        ehttp::Request::post(url, body.into())
    }
}

// marker for an entity with updates that can't be sent yet
// (probably because the previous update is still running)
#[derive(Component)]
pub struct Pending;

#[derive(Component, Debug)]
pub struct RunningRequest {
    pub task: Task<()>,
}

// query args to help remotely query or update an entity's transform
type TransformRequestArgs<'a> = (
    Entity,
    &'a mut Transform,
    Option<&'a mut RunningRequest>,
    Option<&'a Pending>,
);

// need states to prevent updates from sending before the remote camera entity ID is known
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum CameraState {
    #[default]
    Disconnected,
    // getting the remote camera entity...
    Connecting,
    // not a persistent connection, but "connected" as in, able to map to the remote camera
    Connected,
}

// container for HTTP request task spawner
#[derive(Resource)]
struct BrpResource {
    // id seq
    last_id: u32,
    // where we store the bits of the remote camera EntityId
    remote_entity_dungeon: Arc<Mutex<Option<Entity>>>,

    // TODO can we derive the next two from the RemoteState?
    // (probably not because these are for HTTP responses and we can't change state inside a task)
    // (I mean we could return the command in a queue and then execute it on return...)

    // has the remote server been fully initialized, either by this program or existing data
    remote_initialized: Arc<Mutex<bool>>,
    // the remote server needs to be initialized, so spawn the task to make that happen
    remote_needs_init: Arc<Mutex<bool>>,

    // in case we want to do this another way
    request_builder: Box<dyn RemoteRequestBuilder>,
    // server URL http://host:port
    url: String,
}

impl Default for BrpResource {
    fn default() -> Self {
        // Parse the arguments.
        let args: Args = argh::from_env();

        // Create the URL. We're going to need it to issue the HTTP request.
        let url = format!("http://{}:{}", args.host, args.port);
        info!("Server URL: {}", url);

        Self {
            last_id: 0,
            remote_entity_dungeon: Arc::new(Mutex::new(Option::<Entity>::None)),
            remote_initialized: Arc::new(Mutex::new(false)),
            remote_needs_init: Arc::new(Mutex::new(false)),
            request_builder: Box::new(EhttpBuilder {}),
            url,
        }
    }
}

// convenience BRP entry point resource
impl BrpResource {
    fn fetch_remote_camera(
        &mut self,
        entity: Entity,
        commands: &mut Commands
    ) -> anyhow::Result<()> {
        let request_id = self.next_id();

        let request = BrpQueryRequest {
            data: BrpQuery {
                // must use full type path
                components: vec!["bevy_render::camera::camera::Camera".to_string()],
                ..default()
            },
            filter: default(),
        };
        let request = serde_json::to_value(request)?;
        let request = BrpRequest {
            request: "QUERY".to_string(),
            id: request_id.into(),
            params: request,
        };
        let request = serde_json::to_string(&request)?;
        let request = self.request_builder.as_ref().post(self.url.to_string(), request);

        self.spawn_task(request_id, entity, true, request, commands);

        Ok(())
    }

    // increment the id counter and return the next value
    fn next_id(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }

    // insert the provided Transform into the specified remote Entity
    fn post_transform(
        &mut self,
        entity: Entity,
        transform: Transform,
        commands: &mut Commands
    ) -> anyhow::Result<()> {
        let request_id = self.next_id();
        let mut components = HashMap::<String, Value>::new();
        let value = serde_json::to_value(transform)?;

        // must use full type path
        components.insert("bevy_transform::components::transform::Transform".to_string(), value);
        match *self.remote_entity_dungeon.lock().unwrap() {
            Some(remote_entity) => {
                info!("remote_entity (post_transform): {}", remote_entity);
                let request = BrpInsertRequest {
                    entity: remote_entity,
                    components,
                };
                let request = serde_json::to_value(request)?;

                let request = BrpRequest {
                    request: "INSERT".to_string(),
                    id: request_id.into(),
                    params: request,
                };
                let request = serde_json::to_string(&request)?;
                info!("post_transform: {}", request);
                let request = self.request_builder.as_ref().post(self.url.to_string(), request);

                self.spawn_task(request_id, entity, false, request, commands);
                Ok(())
            }
            None => Err(anyhow!("no remote camera entity found")),
        }
    }

    // convenience function to use ehttp to spawn an HTTP request in a Bevy task
    // TODO replace request arg with closure that generates and handles a request
    // (basically the outer closure of the task)
    fn spawn_task(
        &self,
        request_id: u32,
        local_entity: Entity,
        store_remote_entity: bool,
        request: Request,
        commands: &mut Commands
    ) {
        // can't write to the resource from within a thread, so we use this
        let camera_balloon = self.remote_entity_dungeon.clone();
        let initialized_balloon = self.remote_initialized.clone();
        let needs_init_balloon = self.remote_needs_init.clone();
        let thread_pool = IoTaskPool::get();

        // spawn an async task for the long network op
        let task = thread_pool.spawn(async move {
            ehttp::fetch(request, move |response: ehttp::Result<ehttp::Response>| {
                match response {
                    Ok(response) => {
                        trace!("Request ID: {}, status code: {:?}", request_id, response.status);

                        let response = response.text().unwrap();

                        // Just print the JSON to stdout.
                        println!("{}", serde_json::to_string(&response).unwrap());

                        // FIXME shuld probably handle error conditions

                        // if this is a response to the camera query, we need to save it from within this closure
                        if store_remote_entity {
                            // get an entity ID
                            let remote_entity = match
                                serde_json::from_str::<BrpQueryResponse>(response)
                            {
                                Ok(value) => value.rows[0].entity,
                                _ => Entity::PLACEHOLDER,
                            };

                            // float the data back to the resource
                            *camera_balloon.lock().unwrap() = Some(remote_entity);
                        } else if local_entity == Entity::PLACEHOLDER {
                            // this is a response to the entities request
                            if let Ok(value) = serde_json::from_str::<BrpQueryResponse>(response) {
                                if value.rows.is_empty() {
                                    // if it's empty then we need init
                                    *needs_init_balloon.lock().unwrap() = true;
                                } else {
                                    // otherwise assume we should not send additional meshes
                                    *initialized_balloon.lock().unwrap() = true;
                                }
                            };
                        }
                    }
                    Err(error) => trace!("BRP error: {}", error),
                }
            });
        });

        if local_entity == Entity::PLACEHOLDER {
            warn!("making transient BRP request")
        } else {
            commands.entity(local_entity).insert(RunningRequest { task });
        }
    }
}

// step 1: wait for a key press
fn on_press(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD) {
        // try to sync client camera to server
        commands.next_state(CameraState::Connecting);
    }
}

// step 2: run a query to get the entity ID of the remote camera
fn connect_to_camera(
    mut camera: Query<TransformRequestArgs, With<Camera>>,
    mut brp: ResMut<BrpResource>,
    mut commands: Commands
) {
    if let Ok(camera) = camera.get_single_mut() {
        // spawn a task to connect to the remote server
        match brp.fetch_remote_camera(camera.0, &mut commands) {
            Ok(_) => {
                // change the RemoteState to Connecting
                commands.next_state(CameraState::Connecting);
            }
            Err(_) => error!("Could not spawn task to get remote camera"),
        }
    }
}

// step 3: see if the Camera entity has returned yet
fn poll_responses(mut camera: Query<TransformRequestArgs, With<Camera>>, mut commands: Commands) {
    match camera.get_single_mut() {
        Ok(camera) => {
            // check to see if running task has completed
            if let Some(mut request) = camera.2 {
                if block_on(poll_once(&mut request.task)).is_some() {
                    let entity = camera.0;
                    commands.entity(entity).remove::<RunningRequest>();

                    // change the RemoteState to Connected
                    commands.next_state(CameraState::Connected);
                }
            }
        }
        Err(error) => error!("Error loading camera: {}", error),
    }
}

// step 4: read inputs, update local, propagate to remote
fn sync_camera(
    mut camera: Query<TransformRequestArgs, With<Camera>>,
    mut brp: ResMut<BrpResource>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut exit: EventWriter<AppExit>,
    mut commands: Commands
) {
    let units_per_tick = 0.1;

    // read A or D keys
    let mut dir = 0.0;

    // if A, strafe left
    if keyboard_input.pressed(KeyCode::KeyA) {
        dir = -1.0;
    }

    // if D, strafe right
    if keyboard_input.pressed(KeyCode::KeyD) {
        dir = 1.0;
    }

    if let Ok(camera) = camera.get_single_mut() {
        // move camera if needed and send update or mark pending
        let moved = dir != 0.0;
        let entity = camera.0;
        let mut transform = camera.1;
        let running = camera.2.is_some();
        let pending = camera.3.is_some();
        let mut result: anyhow::Result<()> = Ok(());

        // check to see if running task has completed
        if let Some(mut request) = camera.2 {
            if block_on(poll_once(&mut request.task)).is_some() {
                commands.entity(entity).remove::<RunningRequest>();
            }
        }

        // check if a key has been pressed
        if moved {
            // no matter what, update the local camera transform
            transform.translation.x += dir * units_per_tick;
            *transform = transform.looking_at(Vec3::ZERO, Vec3::Y);

            // then we send the serialized Transform to the server
            // -if a request is already running, mark with Pending so we get to it later
            // -if no request is running, send one if either we moved or already pending

            // see if there is a running request
            if running {
                // if so, then mark with Pending if not already
                // (if the request just finished it will still be marked running, which is fine)
                // (next tick it will have no RunningRequest but will be marked Pending)
                if !pending {
                    commands.entity(entity).insert(Pending);
                }
            } else {
                // if no running request, kick off a new request
                result = brp.post_transform(entity, *transform, &mut commands);

                // don't mark with Pending until there is data to send
            }
        } else if pending && !running {
            // if pending and no running request, kick off a new request
            result = brp.post_transform(entity, *transform, &mut commands);

            // remove Pending
            commands.entity(entity).remove::<Pending>();
        }

        // handle error caused while spawning request task
        if let Err(error) = result {
            die(&mut exit, error);
        }
    }
}

fn die(exit: &mut EventWriter<AppExit>, error: anyhow::Error) {
    error!("sync_camera: BRP error: {:?}", error);
    exit.send(AppExit::error());
}

pub fn setup_scene(
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
