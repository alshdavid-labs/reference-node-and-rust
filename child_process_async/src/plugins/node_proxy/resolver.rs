/*
  This is a "Plugin Proxy" that runs functions in the
  Node workers remotely via the NodeInstance, translating
  the requests/responses to match the interface of the
  internal "Resolver" trait
*/
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::node_adapter::NodeInstance;
use crate::public::Resolver;

use async_trait::async_trait;
use serde::Deserialize;
use serde::Serialize;

#[derive(Debug)]
pub struct ResolverNodeProxy {
  resolver_key: String,
  node_instance: Arc<NodeInstance>,
}

impl ResolverNodeProxy {
  pub async fn new(
    node_instance: Arc<NodeInstance>,
    specifier: &str,
  ) -> Self {
    let req = LoadResolverRequest {
      specifier: specifier.to_string(),
    };

    node_instance.send_all("load_resolver", &req).await.unwrap();

    Self {
      resolver_key: specifier.to_string(),
      node_instance,
    }
  }
}

#[async_trait]
impl Resolver for ResolverNodeProxy {
  async fn resolve(
    &self,
    from_path: &Path,
    specifier: &str,
  ) -> Option<PathBuf> {
    let req = RunResolverRequest {
      resolver_key: self.resolver_key.clone(),
      from_path: from_path.to_path_buf(),
      specifier: specifier.to_string(),
    };

    let response: RunResolverResponse = self
      .node_instance
      .send("run_resolver", &req)
      .await
      .unwrap();

    return Some(response.file_path);
  }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct LoadResolverRequest {
  pub specifier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunResolverRequest {
  pub resolver_key: String,
  pub from_path: PathBuf,
  pub specifier: String,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RunResolverResponse {
  pub file_path: PathBuf,
}
