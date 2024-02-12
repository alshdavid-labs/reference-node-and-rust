/*
  This is the common interface for "Resolver" plugins
*/
use std::fmt::Debug;
use std::path::Path;
use std::path::PathBuf;

pub trait Resolver: Sync + Send + Debug {
  fn resolve(
    &self,
    from_path: &Path,
    specifier: &str,
  ) -> Option<PathBuf>;
}
