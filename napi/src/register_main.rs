/*
  This is run once on the main thread and functions as the
  true "main()" function.

  This will wait for all workers to register themselves before
  it starts 
  
  It then handles orchestration logic
*/
use std::env;
use std::sync::Arc;

use neon::prelude::*;

use crate::worker_farm::NodeWorkerFarm;
use crate::public::Resolver;
use crate::plugins::DefaultResolver;
use crate::plugins::ResolverNodeProxy;

pub fn register_main(mut cx: FunctionContext) -> JsResult<JsUndefined> {
  let arg0: Handle<JsNumber> = cx.argument(0)?;
  let worker_count = arg0.value(&mut cx) as usize;

  // Connect to the Node workers
  let worker_farm = Arc::new(NodeWorkerFarm::new(worker_count));

  // Mimic loading plugins in from config
  let mut resolvers = Vec::<Box<dyn Resolver>>::new();
  resolvers.push(Box::new(DefaultResolver::new()));
  resolvers.push(Box::new(ResolverNodeProxy::new(worker_farm.clone(), "../../plugin")));

  // Mimic running resolvers
  let from_path = env::current_dir().unwrap();

  for resolver in &resolvers {
    let Some(file_path) = resolver.resolve(&from_path, "hi") else {
      continue;
    };
    println!("resolved: {:?}", file_path);
  }

  return Ok(cx.undefined());
}
