mod plugins;
mod register_worker;
mod register_main;
mod public;
mod worker_farm;

use register_main::register_main;
use register_worker::register_worker;

use neon::prelude::*;

// Runs multiple times for each node worker
#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
  cx.export_function("register_worker", register_worker)?;

  // This is the true "main()" function
  cx.export_function("register_main", register_main)?;
  Ok(())
}
