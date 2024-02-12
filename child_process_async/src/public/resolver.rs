/*
  This is the common interface for "Resolver" plugins
*/
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

use async_trait::async_trait;

#[async_trait]
pub trait Resolver: Sync + Send + Debug {
  async fn resolve(
    &self,
    from_path: &Path,
    specifier: &str,
  ) -> Option<PathBuf>;
}
