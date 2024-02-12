/*
  This is a "Plugin Proxy" that runs functions in the 
  Node workers remotely via the NodeInstance, translating
  the requests/responses to match the interface of the 
  internal "Resolver" trait
*/
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;

use crate::public::Resolver;

use crate::worker_farm::LoadResolverRequest;
use crate::worker_farm::NodeWorkerFarm;
use crate::worker_farm::PluginRequest;
use crate::worker_farm::PluginResponse;
use crate::worker_farm::RunResolverRequest;

#[derive(Debug)]
pub struct ResolverNodeProxy {
  key: String,
  worker_farm: Arc<NodeWorkerFarm>,
}

impl ResolverNodeProxy {
  pub fn new(
    worker_farm: Arc<NodeWorkerFarm>,
    specifier: &str,
  ) -> Self {
    worker_farm
      .send_all(PluginRequest::LoadResolver(LoadResolverRequest {
        specifier: specifier.to_string(),
      }))
      .unwrap();

    Self {
      worker_farm,
      key: specifier.to_string(),
    }
  }
}

impl Resolver for ResolverNodeProxy {
  fn resolve(
    &self,
    from_path: &Path,
    specifier: &str,
  ) -> Option<PathBuf> {
    let response = self
      .worker_farm
      .send_blocking(PluginRequest::RunResolver(
        self.key.clone(),
        RunResolverRequest {
          from_path: from_path.to_path_buf(),
          specifier: specifier.to_string(),
        },
      ))
      .unwrap();

    let PluginResponse::RunResolver(response) = response else {
      panic!("should not");
    };

    return Some(response.file_path);
  }
}
