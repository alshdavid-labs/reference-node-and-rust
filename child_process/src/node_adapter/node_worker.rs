/*
  This is a wrapper around the Node worker thread instance
  that facilitates sending messages to/from the worker
*/
use std::collections::HashMap;
use std::sync::mpsc::Sender;
use std::sync::Arc;
use std::sync::Mutex;

use serde::de::DeserializeOwned;
use serde::Serialize;

use tokio::sync::oneshot;

#[derive(Debug)]
pub struct NodeWorker {
  pub tx_to_child: Sender<(String, String, String)>,
  pub pending_messages: Arc<Mutex<HashMap<String, tokio::sync::oneshot::Sender<String>>>>,
}

impl NodeWorker {
  pub fn send<T>(
    &self,
    action: &str,
    data: &T,
  ) -> NodeResponse
  where
    T: ?Sized + Serialize,
  {
    let (tx, rx) = oneshot::channel::<String>();
    let msg_ref: String = snowflake::ProcessUniqueId::new().to_string();
    let mut pending_messages = self.pending_messages.lock().unwrap();
    pending_messages.insert(msg_ref.clone(), tx);

    let data = serde_json::to_string::<T>(data).unwrap();
    self
      .tx_to_child
      .send((msg_ref, action.to_string(), data))
      .unwrap();

    return NodeResponse { rx: Some(rx) };
  }
}

pub struct NodeResponse {
  rx: Option<oneshot::Receiver<String>>,
}

impl NodeResponse {
  pub fn recv<T>(&mut self) -> Result<T, ()>
  where
    T: DeserializeOwned,
  {
    let Some(rx) = self.rx.take() else {
      return Err(());
    };
    let Ok(value) = rx.blocking_recv() else {
      return Err(());
    };
    let data = serde_json::from_str::<T>(&value).unwrap();

    return Ok(data);
  }

  pub fn recv_void(&mut self) -> Result<(), ()> {
    let Some(rx) = self.rx.take() else {
      return Err(());
    };
    let Ok(_) = rx.blocking_recv() else {
      return Err(());
    };

    return Ok(());
  }
}
