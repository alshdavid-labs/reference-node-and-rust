/*
  Runs on main
  
  This is an abstraction that provides a nice interface to talk
  to the Node workers from and handles load balancing between them
*/
use std::sync::Arc;
use std::sync::Mutex;

use crate::register_worker::WorkerSender;
use crate::register_worker::WORKER_LOADED;

use super::PluginRequest;
use super::PluginResponse;

#[derive(Debug)]
pub struct NodeWorkerFarm {
  send_to: Arc<Mutex<usize>>,
  workers: Vec<WorkerSender>,
}

impl NodeWorkerFarm {
  pub fn new(worker_count: usize) -> Self {
    let onload = WORKER_LOADED.lock().unwrap().1.take().unwrap();
    let mut workers = Vec::<WorkerSender>::new();

    for _ in 0..worker_count {
      let tx_to_worker = onload.recv().unwrap();
      workers.push(tx_to_worker)
    }

    return NodeWorkerFarm {
      workers,
      send_to: Arc::new(Mutex::new(0)),
    };
  }

  pub fn send_all(
    &self,
    req: PluginRequest,
  ) -> Result<Vec<PluginResponse>, ()> {
    let mut responses = Vec::<PluginResponse>::new();

    for worker in &self.workers {
      let (res, on_response) = tokio::sync::oneshot::channel::<PluginResponse>();
      worker.send((req.clone(), res)).unwrap();
      let Ok(response) = on_response.blocking_recv() else {
        return Err(());
      };
      responses.push(response)
    }
    return Ok(responses);
  }

  pub fn send_blocking(
    &self,
    req: PluginRequest,
  ) -> Result<PluginResponse, ()> {
    // Round robin
    let send_to = {
      let mut send_to = self.send_to.lock().unwrap();

      if *send_to >= self.workers.len() {
        *send_to = 0;
      }

      let send_index = (*send_to).clone();
      *send_to += 1;
      send_index
    };

    let (res, on_response) = tokio::sync::oneshot::channel::<PluginResponse>();
    self.workers[send_to].send((req, res)).unwrap();
    let Ok(response) = on_response.blocking_recv() else {
      return Err(());
    };
    return Ok(response);
  }
}
