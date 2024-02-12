mod node_adapter;
mod plugins;
mod public;

use std::env;
use std::sync::Arc;

use node_adapter::NodeInstance;
use plugins::DefaultResolver;
use public::Resolver;

use crate::plugins::ResolverNodeProxy;

async fn main_async() {
  // Parse CLI args
  let args: Vec<String> = env::args().collect();
  let node_worker_count = args.get(1).unwrap_or(&"4".to_string()).parse::<usize>().unwrap();

  // Create a Node.js child process, spawn worker threads within it and connect to them
  let node_instance = Arc::new(NodeInstance::new(node_worker_count).await);

  // Mimic loading plugins in from config
  let mut resolvers = Vec::<Box<dyn Resolver>>::new();
  resolvers.push(Box::new(DefaultResolver::new()));
  resolvers.push(Box::new(ResolverNodeProxy::new(node_instance.clone(), "../plugin").await));

  // Mimic running resolvers
  let from_path = env::current_dir().unwrap();

  for resolver in &resolvers {
    let Some(file_path) = resolver.resolve(&from_path, "hi").await else {
      continue;
    };
    println!("resolved: {:?}", file_path);
  }
}

fn main() {
  let args: Vec<String> = env::args().collect();
  let node_worker_count = args.get(1).unwrap_or(&"4".to_string()).parse::<usize>().unwrap();

  tokio::runtime::Builder::new_multi_thread()
    .worker_threads(node_worker_count as usize)
    .enable_all()
    .build()
    .unwrap()
    .block_on(main_async());
}
