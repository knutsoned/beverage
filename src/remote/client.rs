use std::{ any::Any, sync::{ Arc, Mutex } };

use bevy::{
    prelude::*,
    remote::{ builtin_verbs::*, BrpRequest, DEFAULT_PORT },
    tasks::IoTaskPool,
    utils::HashMap,
};

use anyhow::anyhow;
use ehttp::Request;
use serde_json::Value;

use crate::prelude::{ DespawnRemoteFpsCounter, RemoteFpsCounter, RemoteRequest };

// ehttp builder
struct EhttpBuilder;
pub trait RemoteRequestBuilder: Send + Sync + 'static {
    // TODO accept callback closure instead of returning ehttp::Request
    fn post(&self, url: String, body: String) -> ehttp::Request;
}

impl RemoteRequestBuilder for EhttpBuilder {
    fn post(&self, url: String, body: String) -> ehttp::Request {
        ehttp::Request::post(url, body.into())
    }
}

// container for HTTP request task spawner
#[derive(Resource)]
pub struct BrpClient {
    // id seq
    pub last_id: u32,

    // where we store the bits of the remote camera EntityId
    pub remote_entity_dungeon: Arc<Mutex<Option<Entity>>>,

    // in case we want to do this another way
    pub request_builder: Box<dyn RemoteRequestBuilder>,

    // server URL http://host:port
    pub url: String,
}

impl Default for BrpClient {
    fn default() -> Self {
        // Create the URL. We're going to need it to issue the HTTP request.
        let url = format!("http://{}:{}", "127.0.0.1", DEFAULT_PORT);
        info!("BRP server URL: {}", url);

        Self {
            last_id: 0,
            remote_entity_dungeon: Arc::new(Mutex::new(Option::<Entity>::None)),
            request_builder: Box::new(EhttpBuilder),
            url,
        }
    }
}

// convenience BRP entry point resource
impl BrpClient {
    pub fn fetch_remote_camera(
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
        let request = self.ehttp_request_from(request_id, request, "QUERY", "fetch_remote_camera")?;

        self.spawn_task(request_id, entity, true, request, commands);

        Ok(())
    }

    // increment the id counter and return the next value
    pub fn next_id(&mut self) -> u32 {
        self.last_id += 1;
        self.last_id
    }

    // insert the provided Transform into the specified remote Entity
    pub fn post_transform(
        &mut self,
        entity: Entity,
        transform: Transform,
        commands: &mut Commands
    ) -> anyhow::Result<()> {
        let mut components = HashMap::<String, Value>::new();
        let request_id = self.next_id();
        let value = serde_json::to_value(transform)?;

        // must use full type path
        components.insert("bevy_transform::components::transform::Transform".to_string(), value);
        match *self.remote_entity_dungeon.lock().unwrap() {
            Some(remote_entity) => {
                trace!("remote_entity (post_transform): {}", remote_entity);
                let request = BrpInsertRequest {
                    entity: remote_entity,
                    components,
                };
                let request = serde_json::to_value(request)?;
                let request = self.ehttp_request_from(
                    request_id,
                    request,
                    "INSERT",
                    "post_transform"
                )?;
                self.spawn_task(request_id, entity, false, request, commands);

                Ok(())
            }
            None => Err(anyhow!("no remote camera entity found")),
        }
    }

    pub fn set_url(&mut self, url: String) {
        self.url = url;
    }

    // convenience function to spawn or despawn the remote FPS counter widget
    pub fn spawn_fps_marker(
        &mut self,
        visibility: bool,
        type_registry: &Res<AppTypeRegistry>,
        commands: &mut Commands
    ) -> anyhow::Result<()> {
        let mut marker = DespawnRemoteFpsCounter.type_id();
        if visibility {
            marker = RemoteFpsCounter.type_id();
        }
        if let Some(marker) = type_registry.read().get_type_info(marker) {
            let marker = marker.type_path();
            warn!("spawning {:#?}", marker);
            let request_id = self.next_id();
            let mut components = HashMap::<String, Value>::new();
            // I _think_ this is how you send an empty struct component
            components.insert(marker.to_string(), Value::Null);
            let request = BrpSpawnRequest { components };
            let request = serde_json::to_value(request)?;
            info!("{:#?}", request);
            let request = self.ehttp_request_from(
                request_id,
                request,
                "SPAWN",
                "spawn_fps_marker"
            )?;
            // FIXME not sure error handling works for spawn requests, so look at server code
            self.spawn_task(request_id, Entity::PLACEHOLDER, false, request, commands);
        }
        Ok(())
    }

    // convenience function to use ehttp to spawn an HTTP request in a Bevy task
    // TODO replace request arg with closure that generates and handles a request
    // (basically the outer closure of the task)
    pub fn spawn_task(
        &self,
        request_id: u32,
        local_entity: Entity,
        store_remote_entity: bool,
        request: Request,
        commands: &mut Commands
    ) {
        // can't write to the resource from within a thread, so we use this
        let camera_balloon = self.remote_entity_dungeon.clone();
        let thread_pool = IoTaskPool::get();

        // spawn an async task for the long network op
        let task = thread_pool.spawn(async move {
            ehttp::fetch(request, move |response: ehttp::Result<ehttp::Response>| {
                match response {
                    Ok(response) => {
                        info!("Request ID: {}, status code: {:?}", request_id, response.status);
                        let response = response.text().unwrap();
                        info!("Response: {}", serde_json::to_string(&response).unwrap());

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
                        }
                    }
                    // FIXME go to Disconnected state if there's an error
                    Err(error) => {
                        error!("BRP error: {}", error);
                    }
                }
            });
        });

        if local_entity == Entity::PLACEHOLDER {
            // this just means we aren't storing any data about the running task
            warn!("making transient BRP request");
        } else {
            commands.entity(local_entity).insert(RemoteRequest { task });
        }
    }

    fn ehttp_request_from(
        &self,
        request_id: u32,
        value: Value,
        verb: &str,
        label: &str
    ) -> anyhow::Result<Request> {
        let request = BrpRequest {
            request: verb.to_string(),
            id: request_id.into(),
            params: value,
        };

        let request = serde_json::to_string(&request)?;
        trace!("{}: {}", label, request);
        Ok(self.request_builder.as_ref().post(self.url.to_string(), request))
    }
}
