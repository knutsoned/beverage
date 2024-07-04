// from pcwalton's bevy/brp branch

//! An implementation of the Bevy Remote Protocol over HTTP and JSON, to allow
//! for remote control of a Bevy app.
//!
//! Adding the [`RemotePlugin`] to your [`App`] causes Bevy to accept
//! connections over HTTP (by default, on port 15702) while your app is running.
//! These *remote clients* can inspect and alter the state of the
//! entity-component system. Clients are expected to `POST` JSON requests to the
//! root URL; see the `client` example for a trivial example of use.
//!
//! ## Requests
//!
//! A typical client request might look like this:
//!
//! ```json
//! {
//!     "request": "GET",
//!     "id": 0,
//!     "params": {
//!         "data": {
//!             "entity": 4294967298,
//!             "components": [
//!                 "bevy_transform::components::transform::Transform"
//!             ]
//!         }
//!     }
//! }
//! ```
//!
//! The `id`, `request`, and `params` fields are all required:
//!
//! * `id` is arbitrary JSON data. The server completely ignores its contents,
//!   and the client may use it for any purpose.  It will be copied via
//!   serialization and deserialization (so object property order, etc. can't be
//!   relied upon to be identical) and sent back to the client as part of the
//!   response.
//!
//! * `request` is a string that specifies one of the possible [`BrpRequest`]
//!   variants: `QUERY`, `GET`, `INSERT`, etc. It's case-sensitive and must be in
//!   all caps.
//!
//! * `params` is parameter data specific to the request.
//!
//! For more information, see the documentation for [`BrpRequest`].
//! [`BrpRequest`] is serialized to JSON via `serde`, so [the `serde`
//! documentation] may be useful to clarify the correspondence between the Rust
//! structure and the JSON format.
//!
//! ## Responses
//!
//! A response from the server to the client might look like this:
//!
//! ```json
//! {
//!     "status": "OK",
//!     "id": 0,
//!     "components": {
//!         "bevy_transform::components::transform::Transform": {
//!             "rotation": { "x": 0.0, "y": 0.0, "z": 0.0, "w": 1.0 },
//!             "scale": { "x": 1.0, "y": 1.0, "z": 1.0 },
//!             "translation": { "x": 0.0, "y": 0.5, "z": 0.0 }
//!         }
//!     },
//!     "entity": 4294967298
//! }
//! ```
//!
//! The `status` and `id` fields will always be present:
//!
//! * `id` is the arbitrary JSON data that was sent as part of the request. It
//!   will be identical to the `id` data sent during the request, modulo
//!   serialization and deserialization.
//!
//! * `status` will be either the string `"OK"` or `"ERROR"`, reflecting whether
//!   the request succeeded.
//!
//! TODO: Fill in more here.
//!
//! [the `serde` documentation]: https://serde.rs/

use std::sync::{ Arc, Mutex };

use anyhow::{ anyhow, Result as AnyhowResult };
use bevy::{ ecs::system::SystemId, prelude::*, tasks::IoTaskPool, utils::HashMap };
use http_body_util::{ BodyExt as _, Full };
use hyper::{ body::{ Bytes, Incoming }, server::conn::http1, service, Request, Response };
use serde::{ Deserialize, Serialize };
use serde_json::{ value, Map, Value };
use smol::{ channel::{ self, Receiver, Sender }, Async };
use smol_hyper::rt::{ FuturesIo, SmolTimer };
use std::net::{ TcpListener, TcpStream };

pub mod builtin_verbs;

/// The default port that Bevy will listen on.
///
/// This value was chosen randomly.
pub const DEFAULT_PORT: u16 = 15702;

const CHANNEL_SIZE: usize = 16;

/// Add this plugin to your [`App`] to allow remote connections to inspect and modify entities.
///
/// By default, this is [`DEFAULT_PORT`]: 15702.
pub struct EditorRemotePlugin {
    /// The port that Bevy will listen on.
    pub port: u16,
}

/// A resource containing the port number that Bevy will listen on.
#[derive(Resource, Reflect)]
pub struct RemotePort(pub u16);

/// The type of a function that implements a remote verb (`GET`, `QUERY`, etc.)
///
/// The first parameter is the JSON value of the `params`. Typically, an
/// implementation will deserialize these as the first thing they do.
///
/// The returned JSON value will be returned as the response. Bevy will
/// automatically populate the `status` and `id` fields before sending.
pub type RemoteVerb = SystemId<Value, AnyhowResult<Value>>;

/// Holds all implementations of verbs known to the server.
///
/// You can add your own custom verbs to this list.
#[derive(Resource, Default)]
pub struct RemoteVerbs(HashMap<String, RemoteVerb>);

/// A single request from a Bevy Remote Protocol client to the server,
/// serialized in JSON.
///
/// The JSON payload is expected to look like this:
///
/// ```json
/// {
///     "request": "GET",
///     "id": 0,
///     "params": {
///         "data": {
///             "entity": 4294967298,
///             "components": [
///                 "bevy_transform::components::transform::Transform"
///             ]
///         }
///     }
/// }
/// ```
#[derive(Serialize, Deserialize, Clone)]
pub struct BrpRequest {
    /// The verb: i.e. the action to be performed.
    pub request: String,

    /// Arbitrary data that will be returned verbatim to the client as part of
    /// the response.
    pub id: Value,

    /// The parameters, specific to each verb.
    ///
    /// These are passed as the first argument to the verb handler.
    pub params: Value,
}

/// A message from the Bevy Remote Protocol server thread to the main world.
///
/// This is placed in the [`BrpMailbox`].
#[derive(Clone)]
pub struct BrpMessage {
    /// The deserialized request from the client.
    request: BrpRequest,

    /// The channel on which the response is to be sent.
    ///
    /// The value sent here is serialized and sent back to the client.
    sender: Arc<Mutex<Option<Sender<AnyhowResult<Value>>>>>,
}

/// A resource that receives messages sent by Bevy Remote Protocol clients.
///
/// Every frame, the `process_remote_requests` system drains this mailbox, and
/// processes the messages within.
#[derive(Resource, Deref, DerefMut)]
pub struct BrpMailbox(Receiver<BrpMessage>);

impl Default for EditorRemotePlugin {
    fn default() -> Self {
        EditorRemotePlugin { port: DEFAULT_PORT }
    }
}

impl Plugin for EditorRemotePlugin {
    fn build(&self, app: &mut App) {
        let mut remote_verbs = RemoteVerbs::new();
        remote_verbs.insert(
            "GET".to_owned(),
            app.register_system(builtin_verbs::process_remote_get_request)
        );
        remote_verbs.insert(
            "QUERY".to_owned(),
            app.register_system(builtin_verbs::process_remote_query_request)
        );
        remote_verbs.insert(
            "SPAWN".to_owned(),
            app.register_system(builtin_verbs::process_remote_spawn_request)
        );
        remote_verbs.insert(
            "INSERT".to_owned(),
            app.register_system(builtin_verbs::process_remote_insert_request)
        );
        remote_verbs.insert(
            "REMOVE".to_owned(),
            app.register_system(builtin_verbs::process_remote_remove_request)
        );
        remote_verbs.insert(
            "DESTROY".to_owned(),
            app.register_system(builtin_verbs::process_remote_destroy_request)
        );
        remote_verbs.insert(
            "REPARENT".to_owned(),
            app.register_system(builtin_verbs::process_remote_reparent_request)
        );
        remote_verbs.insert(
            "LIST".to_owned(),
            app.register_system(builtin_verbs::process_remote_list_request)
        );

        app.insert_resource(RemotePort(self.port))
            .insert_resource(remote_verbs)
            .add_systems(Startup, start_server)
            .add_systems(Update, process_remote_requests);
    }
}

impl RemoteVerbs {
    /// Creates a new [`RemoteVerbs`] resource with no verbs registered in it.
    pub fn new() -> Self {
        default()
    }

    /// Adds a new verb, replacing any existing verb with that name.
    ///
    /// If there was an existing verb with that name, returns its handler.
    pub fn insert(
        &mut self,
        verb_name: impl Into<String>,
        handler: RemoteVerb
    ) -> Option<RemoteVerb> {
        self.0.insert(verb_name.into(), handler)
    }
}

/// A system that starts up the Bevy Remote Protocol server.
fn start_server(mut commands: Commands, remote_port: Res<RemotePort>) {
    // Create the channel and the mailbox.
    let (request_sender, request_receiver) = channel::bounded(CHANNEL_SIZE);
    commands.insert_resource(BrpMailbox(request_receiver));

    IoTaskPool::get().spawn(server_main(remote_port.0, request_sender)).detach();
}

/// A system that receives requests placed in the [`BrpMailbox`] and processes
/// them.
///
/// This needs exclusive access to the [`World`] because clients can manipulate
/// anything in the ECS.
fn process_remote_requests(world: &mut World) {
    if !world.contains_resource::<BrpMailbox>() {
        return;
    }

    while let Ok(message) = world.resource_mut::<BrpMailbox>().try_recv() {
        let Ok(mut sender) = message.sender.lock() else {
            continue;
        };
        let Some(sender) = sender.take() else {
            continue;
        };

        // Fetch the handler for the verb. If there's no such handler
        // registered, return an error.
        let verbs = world.resource::<RemoteVerbs>();
        let Some(handler) = verbs.0.get(&message.request.request) else {
            let _ = sender.send_blocking(
                Err(anyhow!("Unknown verb: `{}`", message.request.request))
            );
            continue;
        };

        // Execute the handler, and send the result back to the client.
        let result = match world.run_system_with_input(*handler, message.request.params) {
            Ok(result) => result,
            Err(error) => {
                let _ = sender.send_blocking(Err(anyhow!("Failed to run handler: {}", error)));
                continue;
            }
        };

        let _ = sender.send_blocking(result);
    }
}

/// The Bevy Remote Protocol server main loop.
async fn server_main(port: u16, sender: Sender<BrpMessage>) -> AnyhowResult<()> {
    listen(Async::<TcpListener>::bind(([127, 0, 0, 1], port))?, sender).await?;
    Ok(())
}

async fn listen(listener: Async<TcpListener>, sender: Sender<BrpMessage>) -> AnyhowResult<()> {
    loop {
        let (client, _) = listener.accept().await?;

        let sender = sender.clone();
        IoTaskPool::get()
            .spawn(async move {
                let _ = handle_client(client, sender).await;
            })
            .detach();
    }
}

async fn handle_client(client: Async<TcpStream>, sender: Sender<BrpMessage>) -> AnyhowResult<()> {
    http1::Builder
        ::new()
        .timer(SmolTimer::new())
        .serve_connection(
            FuturesIo::new(client),
            service::service_fn(|request| process_request(request, sender.clone()))
        ).await?;

    Ok(())
}

/// A helper function for the Bevy Remote Protocol server that handles a single
/// request coming from a client.
async fn process_request(
    request: Request<Incoming>,
    sender: Sender<BrpMessage>
) -> AnyhowResult<Response<Full<Bytes>>> {
    let request_bytes = request.into_body().collect().await?.to_bytes();
    let request: BrpRequest = serde_json::from_slice(&request_bytes)?;

    // Save the `id` field so we can echo it back.
    let id = request.id.clone();

    let mut value = match process_request_body(request, &sender).await {
        Ok(mut value) => {
            value.insert("status".to_owned(), "OK".into());
            value
        }
        Err(err) => {
            let mut response = Map::new();
            response.insert("status".to_owned(), "ERROR".into());
            response.insert("message".to_owned(), err.to_string().into());
            response
        }
    };

    // Echo the same `id` value back to the client.
    value.insert("id".to_owned(), id);

    // Serialize and return the JSON as a response.
    let string = serde_json::to_string(&value)?;
    Ok(Response::new(Full::new(Bytes::from(string.as_bytes().to_owned()))))
}

/// A helper function for the Bevy Remote Protocol server that parses a single
/// request coming from a client and places it in the [`BrpMailbox`].
async fn process_request_body(
    request: BrpRequest,
    sender: &Sender<BrpMessage>
) -> AnyhowResult<Map<String, Value>> {
    let (response_sender, response_receiver) = channel::bounded(1);

    let _ = sender.send(BrpMessage {
        request,
        sender: Arc::new(Mutex::new(Some(response_sender))),
    }).await;

    let response = response_receiver.recv().await??;
    match value::to_value(response)? {
        Value::Object(map) => Ok(map),
        _ => Err(anyhow!("Response wasn't an object")),
    }
}

pub mod brp_client;
pub mod camera_control;

// insert editor BRP client API here
