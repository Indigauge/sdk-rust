use std::ops::{Deref, DerefMut};

use bevy::{
  ecs::system::{EntityCommands, IntoObserverSystem, SystemParam},
  prelude::*,
  tasks::IoTaskPool,
};

use bevy::log::{debug, error, info};
pub use reqwest;
pub use reqwest::StatusCode;
pub use reqwest::header::HeaderMap;

#[cfg(target_family = "wasm")]
use crossbeam_channel::{Receiver, bounded};

#[cfg(not(target_family = "wasm"))]
use {bevy::tasks::Task, futures_lite::future};

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
pub struct HttpRequestSet;

pub struct ReqwestPlugin {
  pub automatically_name_requests: bool,
}

impl Default for ReqwestPlugin {
  fn default() -> Self {
    Self {
      automatically_name_requests: true,
    }
  }
}

impl Plugin for ReqwestPlugin {
  fn build(&self, app: &mut App) {
    app.init_resource::<ReqwestClient>();

    if self.automatically_name_requests {
      app
        .world_mut()
        .register_component_hooks::<ReqwestInflight>()
        .on_insert(|mut world, ctx| {
          let request_url = world.get::<ReqwestInflight>(ctx.entity).unwrap().url.clone();

          if world.get::<Name>(ctx.entity).is_none() {
            let mut commands = world.commands();
            let mut entity = commands.get_entity(ctx.entity).unwrap();
            entity.insert(Name::new(format!("http: {request_url}")));
          }
        });
    }

    app.add_systems(
      PreUpdate,
      (Self::cleanup_finished_request_entities, Self::poll_inflight_requests)
        .chain()
        .in_set(HttpRequestSet),
    );
  }
}

impl ReqwestPlugin {
  fn cleanup_finished_request_entities(
    mut commands: Commands,
    q: Query<Entity, (With<DespawnReqwestEntity>, Without<ReqwestInflight>)>,
  ) {
    for entity in q.iter() {
      if let Ok(mut ec) = commands.get_entity(entity) {
        ec.despawn();
      }
    }
  }

  fn poll_inflight_requests(mut commands: Commands, mut requests: Query<(Entity, &mut ReqwestInflight)>) {
    for (entity, mut inflight) in requests.iter_mut() {
      debug!("polling request: {entity:?}");

      let Some((result, parts)) = inflight.poll() else {
        continue;
      };

      match result {
        Ok(body) => {
          let response = parts.expect("response parts should exist when request succeeds");
          commands.trigger(ReqwestResponseEvent::new(entity, body, response.status, response.headers));
        },
        Err(error) => {
          commands.trigger(ReqwestErrorEvent { entity, error });
        },
      }

      if let Ok(mut ec) = commands.get_entity(entity) {
        ec.remove::<ReqwestInflight>();
      }
    }
  }
}

pub struct BevyReqwestBuilder<'a>(EntityCommands<'a>);

impl<'a> BevyReqwestBuilder<'a> {
  pub fn on_response<RB: Bundle, RM, OR: IntoObserverSystem<ReqwestResponseEvent, RB, RM>>(
    mut self,
    on_response: OR,
  ) -> Self {
    self.0.observe(on_response);
    self
  }

  pub fn on_json_response<
    T: std::marker::Sync + std::marker::Send + serde::de::DeserializeOwned + 'static,
    RB: Bundle,
    RM,
    OR: IntoObserverSystem<JsonResponse<T>, RB, RM>,
  >(
    mut self,
    on_response: OR,
  ) -> Self {
    self.0.observe(|evt: On<ReqwestResponseEvent>, mut commands: Commands| {
      let entity = evt.event().entity;

      match evt.event().deserialize_json::<T>() {
        Ok(data) => {
          commands.trigger(JsonResponse { entity, data });
        },
        Err(error) => {
          error!("deserialization error: {error}");
          debug!("response body: {}", evt.event().as_str().unwrap_or("failed to read response body"));
        },
      }
    });

    self.0.observe(on_response);
    self
  }

  pub fn on_error<EB: Bundle, EM, OE: IntoObserverSystem<ReqwestErrorEvent, EB, EM>>(mut self, on_error: OE) -> Self {
    self.0.observe(on_error);
    self
  }
}

#[derive(SystemParam)]
pub struct BevyReqwest<'w, 's> {
  commands: Commands<'w, 's>,
  client: Res<'w, ReqwestClient>,
}

impl<'w, 's> BevyReqwest<'w, 's> {
  pub fn send(&mut self, request: reqwest::Request) -> BevyReqwestBuilder<'_> {
    let inflight = self.create_inflight_task(request);
    BevyReqwestBuilder(self.commands.spawn((inflight, DespawnReqwestEntity)))
  }

  pub fn send_using_entity(
    &mut self,
    entity: Entity,
    request: reqwest::Request,
  ) -> Result<BevyReqwestBuilder<'_>, Box<dyn std::error::Error>> {
    let inflight = self.create_inflight_task(request);
    let mut ec = self.commands.get_entity(entity)?;
    info!("inserting request on entity: {entity:?}");
    ec.insert(inflight);
    Ok(BevyReqwestBuilder(ec))
  }

  pub fn client(&self) -> &reqwest::Client {
    &self.client.0
  }

  fn create_inflight_task(&self, request: reqwest::Request) -> ReqwestInflight {
    let pool = IoTaskPool::get();
    let client = self.client.0.clone();
    let url = request.url().to_string();

    #[cfg(target_family = "wasm")]
    let task = {
      let (tx, receiver) = bounded(1);
      pool
        .spawn(async move {
          let outcome = perform_request(client, request).await;
          tx.send(outcome).ok();
        })
        .detach();
      receiver
    };

    #[cfg(not(target_family = "wasm"))]
    let task = { pool.spawn(async move { async_compat::Compat::new(perform_request(client, request)).await }) };

    ReqwestInflight::new(task, url)
  }
}

impl<'w, 's> Deref for BevyReqwest<'w, 's> {
  type Target = reqwest::Client;

  fn deref(&self) -> &Self::Target {
    self.client()
  }
}

#[derive(Component)]
pub struct DespawnReqwestEntity;

#[derive(Resource)]
pub struct ReqwestClient(pub reqwest::Client);

impl Default for ReqwestClient {
  fn default() -> Self {
    Self(reqwest::Client::new())
  }
}

impl Deref for ReqwestClient {
  type Target = reqwest::Client;

  fn deref(&self) -> &Self::Target {
    &self.0
  }
}

impl DerefMut for ReqwestClient {
  fn deref_mut(&mut self) -> &mut Self::Target {
    &mut self.0
  }
}

type InflightResult = (reqwest::Result<bytes::Bytes>, Option<ResponseParts>);

#[derive(Component)]
#[component(storage = "SparseSet")]
pub struct ReqwestInflight {
  pub(crate) url: String,
  #[cfg(not(target_family = "wasm"))]
  result: Task<InflightResult>,
  #[cfg(target_family = "wasm")]
  result: Receiver<InflightResult>,
}

impl ReqwestInflight {
  fn poll(&mut self) -> Option<InflightResult> {
    #[cfg(target_family = "wasm")]
    {
      self.result.try_recv().ok()
    }

    #[cfg(not(target_family = "wasm"))]
    {
      future::block_on(future::poll_once(&mut self.result))
    }
  }

  #[cfg(target_family = "wasm")]
  fn new(result: Receiver<InflightResult>, url: String) -> Self {
    Self { url, result }
  }

  #[cfg(not(target_family = "wasm"))]
  fn new(result: Task<InflightResult>, url: String) -> Self {
    Self { url, result }
  }
}

#[derive(Component, Debug)]
struct ResponseParts {
  status: StatusCode,
  headers: HeaderMap,
}

#[derive(Clone, EntityEvent, Debug)]
pub struct ReqwestResponseEvent {
  entity: Entity,
  bytes: bytes::Bytes,
  status: StatusCode,
  headers: HeaderMap,
}

#[derive(EntityEvent, Debug)]
pub struct ReqwestErrorEvent {
  pub entity: Entity,
  pub error: reqwest::Error,
}

impl ReqwestResponseEvent {
  #[inline]
  pub fn body(&self) -> &bytes::Bytes {
    &self.bytes
  }

  pub fn as_str(&self) -> anyhow::Result<&str> {
    Ok(std::str::from_utf8(&self.bytes)?)
  }

  pub fn as_string(&self) -> anyhow::Result<String> {
    Ok(self.as_str()?.to_string())
  }

  pub fn deserialize_json<'de, T: serde::Deserialize<'de>>(&'de self) -> anyhow::Result<T> {
    Ok(serde_json::from_str(self.as_str()?)?)
  }

  #[inline]
  pub fn status(&self) -> StatusCode {
    self.status
  }

  #[inline]
  pub fn response_headers(&self) -> &HeaderMap {
    &self.headers
  }

  fn new(entity: Entity, bytes: bytes::Bytes, status: StatusCode, headers: HeaderMap) -> Self {
    Self {
      entity,
      bytes,
      status,
      headers,
    }
  }
}

#[derive(EntityEvent)]
pub struct JsonResponse<T> {
  pub entity: Entity,
  pub data: T,
}

async fn perform_request(client: reqwest::Client, request: reqwest::Request) -> InflightResult {
  match client.execute(request).await {
    Ok(response) => {
      let parts = ResponseParts {
        status: response.status(),
        headers: response.headers().clone(),
      };
      (response.bytes().await, Some(parts))
    },
    Err(error) => (Err(error), None),
  }
}
