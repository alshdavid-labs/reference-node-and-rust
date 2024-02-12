/*
  Types that go to/from Node workers
*/
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub enum PluginRequest {
  LoadResolver(LoadResolverRequest),
  RunResolver(String, RunResolverRequest),
}

#[derive(Clone, Debug)]
pub enum PluginResponse {
  LoadResolver,
  RunResolver(RunResolverResponse),
}

#[derive(Clone, Debug)]
pub struct LoadResolverRequest {
  pub specifier: String,
}

#[derive(Clone, Debug)]
pub struct RunResolverRequest {
  pub from_path: PathBuf,
  pub specifier: String,
}

#[derive(Clone, Debug)]
pub struct RunResolverResponse {
  pub file_path: PathBuf,
}
