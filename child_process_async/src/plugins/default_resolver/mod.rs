/*
  Not exactly a plugin, this is the default resolver implementation
  which is statically compiled into the bundler and selected dynamically
  as if were a dynamically loaded plugin

  I have included it to demonstrate the ability to use the Resolver
  trait abstraction
*/
use std::path::Path;
use std::path::PathBuf;

use async_trait::async_trait;

use crate::public::Resolver;

#[derive(Debug)]
pub struct DefaultResolver {}

impl DefaultResolver {
  pub fn new() -> Self {
    Self {}
  }
}

#[async_trait]
impl Resolver for DefaultResolver {
  async fn resolve(
    &self,
    _from_path: &Path,
    _specifier: &str,
  ) -> Option<PathBuf> {
    return None;
  }
}
