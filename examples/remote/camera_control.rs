//! A Bevy app that sends camera position commands to a BRP server.

use std::sync::{ Arc, Mutex };

use bevy::{
    ecs::event::EventWriter,
    prelude::*,
    remote::{
        builtin_verbs::{ BrpInsertRequest, BrpQuery, BrpQueryRequest, BrpQueryRow },
        BrpRequest,
        DEFAULT_PORT,
    },
    tasks::{ IoTaskPool, Task },
    utils::HashMap,
};

use anyhow::anyhow;
use argh::FromArgs;
use ehttp::Request;
use futures_lite::future;
use serde::{ Deserialize, Serialize };
use serde_json::Value;

mod scene;
use scene::*;

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
    // TODO accept closure instead of returning ehttp::Request
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

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<BrpResource>()
        .init_state::<RemoteState>()
        .add_systems(Startup, setup_scene)
        .add_systems(Update, connect_to_camera.run_if(in_state(RemoteState::Disconnected)))
        .add_systems(Update, check_connection.run_if(in_state(RemoteState::Connecting)))
        .add_systems(Update, sync_camera.run_if(in_state(RemoteState::Connected)))
        .run();
}

// query args to help remotely query or update an entity's transform
type TransformRequestArgs<'a> = (
    Entity,
    &'a mut Transform,
    Option<&'a mut RunningRequest>,
    Option<&'a Pending>,
);

// need states to prevent updates from sending before the remote entity is known
#[derive(States, Default, Debug, Clone, PartialEq, Eq, Hash)]
enum RemoteState {
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
    last_id: u32,
    url: String,
    remote_entity_dungeon: Arc<Mutex<Option<Entity>>>,
    request_builder: Box<dyn RemoteRequestBuilder>,
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
        let balloon = self.remote_entity_dungeon.clone();
        let thread_pool = IoTaskPool::get();

        // spawn an async task for the long network op
        let task = thread_pool.spawn(async move {
            ehttp::fetch(request, move |result: ehttp::Result<ehttp::Response>| {
                let result = result.unwrap();
                println!("Request ID: {}, status code: {:?}", request_id, result.status);

                let result = result.text().unwrap();

                // Just print the JSON to stdout.
                println!("{}", serde_json::to_string(&result).unwrap());

                if store_remote_entity {
                    // get an entity ID
                    let remote_entity = match serde_json::from_str::<BrpQueryResponse>(result) {
                        Ok(value) => value.rows[0].entity,
                        _ => Entity::PLACEHOLDER,
                    };
                    *balloon.lock().unwrap() = Some(remote_entity);
                }
            });
        });

        commands.entity(local_entity).insert(RunningRequest { task });
    }
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
            url,
            remote_entity_dungeon: Arc::new(Mutex::new(Option::<Entity>::None)),
            request_builder: Box::new(EhttpBuilder {}),
        }
    }
}

// run a query to get the entity ID of the remote camera
fn connect_to_camera(
    mut camera: Query<TransformRequestArgs, With<Camera>>,
    mut brp: ResMut<BrpResource>,
    mut next_state: ResMut<NextState<RemoteState>>,
    mut commands: Commands
) {
    if let Ok(camera) = camera.get_single_mut() {
        // spawn a task to connect to the remote server
        match brp.fetch_remote_camera(camera.0, &mut commands) {
            Ok(_) => {
                // change the RemoteState to Connecting
                next_state.set(RemoteState::Connecting);
            }
            Err(_) => error!("could not spawn task to get remote camera"),
        }
    }
}

// see if the Camera entity has returned yet
fn check_connection(
    mut camera: Query<TransformRequestArgs, With<Camera>>,
    mut next_state: ResMut<NextState<RemoteState>>,
    mut commands: Commands
) {
    if let Ok(camera) = camera.get_single_mut() {
        // check to see if running task has completed
        if let Some(mut request) = camera.2 {
            if future::block_on(future::poll_once(&mut request.task)).is_some() {
                let entity = camera.0;
                commands.entity(entity).remove::<RunningRequest>();

                // change the RemoteState to Connected
                next_state.set(RemoteState::Connected);
            }
        }
    }
}

// read inputs, update local, propagate to remote
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
            point_at_origin(*transform);

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
