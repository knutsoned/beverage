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
        builtin_verbs::{
            BrpInsertRequest,
            BrpQuery,
            BrpQueryRequest,
            BrpQueryRow,
            //BrpSpawnRequest,
        },
        BrpRequest,
        DEFAULT_PORT,
    },
    render::primitives::Aabb,
    tasks::{ IoTaskPool, Task },
    utils::HashMap,
};

use anyhow::anyhow;
use argh::FromArgs;
use ehttp::Request;
use futures_lite::future;
use serde::{ Deserialize, Serialize };
use serde_json::Value;

use sickle_ui::ui_commands::UpdateStatesExt;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BrpResource>()
        .init_state::<CameraState>()
        .init_state::<RemoteState>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, on_press.run_if(in_state(CameraState::Disconnected)))
        // step 1
        .add_systems(
            Update,
            (check_remote_scene, connect_to_remote_server).run_if(in_state(RemoteState::Connecting))
        )
        // step 2
        .add_systems(Update, (init_remote_scene,).run_if(in_state(RemoteState::Uninitialized)))

        // step 3
        .add_systems(
            Update,
            (connect_to_camera, poll_responses).run_if(in_state(CameraState::Connecting))
        )
        // step 4
        .add_systems(Update, sync_camera.run_if(in_state(CameraState::Connected)))
        .run();
}

// The response to a `QUERY` request.
// (this was private in the bevy_remote crate)
#[derive(Serialize, Deserialize, Clone)]
struct BrpQueryResponse {
    /// All results of the query: the entities and the requested components.
    rows: Vec<BrpQueryRow>,
}

// ehttp builder
struct EhttpBuilder {}

impl RemoteRequestBuilder for EhttpBuilder {
    fn post(&self, url: String, body: String) -> ehttp::Request {
        ehttp::Request::post(url, body.into())
    }
}

trait RemoteRequestBuilder: Send + Sync + 'static {
    // TODO accept callback closure instead of returning ehttp::Request
    fn post(&self, url: String, body: String) -> ehttp::Request;
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

// marker for an entity with updates that can't be sent yet
// (probably because the previous update is still running)
#[derive(Component)]
pub struct Pending;

#[derive(Component, Debug)]
pub struct RunningRequest {
    pub task: Task<()>,
}

// query args to find meshes to be sent to the server
type SpawnRemoteMeshArgs<'a> = (
    &'a Transform,
    &'a GlobalTransform,
    &'a Visibility,
    &'a InheritedVisibility,
    &'a ViewVisibility,
    &'a Handle<Mesh>,
    &'a Handle<StandardMaterial>,
    &'a Aabb,
);

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

// these states allow querying the server and sending local meshes over the wire if need be
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum RemoteState {
    #[default]
    Disconnected,
    // querying the remote state
    Connecting,
    // connected to the server, no data returned, so send some
    Uninitialized,
    // meshes detected or scene sent to remote
    Initialized,
}

// container for HTTP request task spawner
#[derive(Resource)]
struct BrpResource {
    // id seq
    last_id: u32,
    // a list of components that identify a PbrBundle or compatible
    mesh_components: Vec<String>,
    // where we store the bits of the remote camera EntityId
    remote_entity_dungeon: Arc<Mutex<Option<Entity>>>,
    // in case we want to do this another way
    request_builder: Box<dyn RemoteRequestBuilder>,
    // server URL http://host:port
    url: String,

    // TODO can we derive the next two from the RemoteState?
    // (probably not because these are for HTTP responses and we can't change state inside a task)
    // (I mean we could return the command in a queue and then execute it on return...)

    // has the remote server been fully initialized, either by this program or existing data
    remote_initialized: Arc<Mutex<bool>>,
    // the remote server needs to be initialized, so spawn the task to make that happen
    remote_needs_init: Arc<Mutex<bool>>,
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
            // this needs to match the list of types in SpawnRemoteMeshArgs
            mesh_components: vec![
                "bevy_transform::components::transform::Transform".to_string(),
                "bevy_transform::components::global_transform::GlobalTransform".to_string(),
                "bevy_render::view::visibility::Visibility".to_string(),
                "bevy_render::view::visibility::InheritedVisibility".to_string(),
                "bevy_render::view::visibility::ViewVisibility".to_string(),
                "bevy_asset::handle::Handle<bevy_render::mesh::mesh::Mesh>".to_string(),
                "bevy_asset::handle::Handle<bevy_pbr::pbr_material::StandardMaterial>".to_string(),
                "bevy_render::primitives::Aabb".to_string()
            ],
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

    fn fetch_remote_entities(&mut self, commands: &mut Commands) -> anyhow::Result<()> {
        let request_id = self.next_id();

        let request = BrpQueryRequest {
            data: BrpQuery {
                components: self.mesh_components.clone(),
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

        self.spawn_task(request_id, Entity::PLACEHOLDER, false, request, commands);

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

    // convenience function to make a SPAWN request
    /*
    fn spawn_entity(&mut self, components: HashMap<String, Value>) -> anyhow::Result<()> {
        let request_id = self.next_id();
        let thread_pool = IoTaskPool::get();

        let request = BrpSpawnRequest {
            components,
        };
        let request = serde_json::to_value(request)?;

        let request = BrpRequest {
            request: "SPAWN".to_string(),
            id: request_id.into(),
            params: request,
        };
        let request = serde_json::to_string(&request)?;
        info!("spawn request: {}", request);
        let request = self.request_builder.as_ref().post(self.url.to_string(), request);

        thread_pool
            .spawn(async move {
                ehttp::fetch(request, move |response: ehttp::Result<ehttp::Response>| {
                    match response {
                        Ok(response) => info!("Received response: {:?}", response),
                        Err(error) => error!("BRP error response sending entity: {:?}", error),
                    }
                });
            })
            .detach();

        Ok(())
    }
    */

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

// step 0: wait for a key press
fn on_press(keyboard_input: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard_input.pressed(KeyCode::KeyA) || keyboard_input.pressed(KeyCode::KeyD) {
        // try to sync client camera to server
        commands.next_state(CameraState::Connecting);
    }
}

// step 1: see if there are any remote entities
fn connect_to_remote_server(mut brp: ResMut<BrpResource>, mut commands: Commands) {
    // spawn a task to connect to the remote server
    match brp.fetch_remote_entities(&mut commands) {
        Ok(_) => {
            // the HTTP client sets transfer values in the BrpResource

            // all we do here is aadvance to the next state where a system checks for those values to change
            commands.next_state(RemoteState::Uninitialized);
        }
        Err(error) => error!("Could not spawn task to get remote camera: {}", error),
    }
}

// step 2a: update the state to reflect resource changes from BRP responses
fn check_remote_scene(brp: ResMut<BrpResource>, mut commands: Commands) {
    let mut initialized = *brp.remote_initialized.lock().unwrap();
    let needs_init = *brp.remote_needs_init.lock().unwrap();
    if needs_init {
        // start sending the scene to the server
        *brp.remote_needs_init.lock().unwrap() = false;

        // we don't do any retrying or anything here
        initialized = true;
    }

    // either the HTTP client or this fn sets this to true above
    if initialized {
        commands.next_state(RemoteState::Initialized);
    }
}

// step 2b: stop everything and send any local entity with a Transform to the server
// (this won't work because we can't really serialize a handle)
fn init_remote_scene(world: &mut World, _transforms: &mut QueryState<SpawnRemoteMeshArgs>) {
    /*
    warn!("sending local entities to the server");
    world.resource_scope(|world, mut brp: Mut<BrpResource>| {
        for (
            transform,
            global_transform,
            _visibility,
            _inherited_visibility,
            _view_visibility,
            _handle_mesh,
            _handle_material,
            _aabb,
        ) in transforms.iter(world) {
            let mut hash = HashMap::<String, Value>::new();
            let keys = brp.mesh_components.clone();
            hash.insert(keys[0].clone(), serde_json::to_value(transform).unwrap());
            hash.insert(keys[1].clone(), serde_json::to_value(global_transform).unwrap());
            hash.insert(keys[2].clone(), serde_json::to_value(visibility).unwrap());
            hash.insert(keys[3].clone(), serde_json::to_value(inherited_visibility).unwrap());
            hash.insert(keys[4].clone(), serde_json::to_value(view_visibility).unwrap());
            hash.insert(keys[5].clone(), serde_json::to_value(handle_mesh).unwrap());
            hash.insert(keys[6].clone(), serde_json::to_value(handle_material).unwrap());
            hash.insert(keys[7].clone(), serde_json::to_value(aabb).unwrap());

            let _ = brp.spawn_entity(hash).map_err(|error| {
                error!("BRP error sending an entity: {:?}", error);
            });
        }
    });
    */
    world.resource_scope(|_world, mut next_state: Mut<NextState<RemoteState>>| {
        next_state.set(RemoteState::Connecting);
    });
}

// step 3a: run a query to get the entity ID of the remote camera
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

// step 3b: see if the Camera entity has returned yet
fn poll_responses(mut camera: Query<TransformRequestArgs, With<Camera>>, mut commands: Commands) {
    match camera.get_single_mut() {
        Ok(camera) => {
            // check to see if running task has completed
            if let Some(mut request) = camera.2 {
                if future::block_on(future::poll_once(&mut request.task)).is_some() {
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
            if future::block_on(future::poll_once(&mut request.task)).is_some() {
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
    // step 7: next tick
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
